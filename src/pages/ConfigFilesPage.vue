<script setup lang="ts">
import { computed } from "vue";
import { ElMessage } from "element-plus";
import { useErrorHandler } from "../composables/useErrorHandler";
import { invoke } from "@tauri-apps/api/core";
import {
  Document,
  FolderOpened,
  Refresh,
  UploadFilled,
} from "@element-plus/icons-vue";
import type { FileStatus, RimeEnvironment } from "../types";

const props = defineProps<{
  env?: RimeEnvironment;
  backingUp?: boolean;
}>();

const emit = defineEmits<{
  refresh: [];
  createBackup: [];
}>();

const { withErrorHandling } = useErrorHandler();

type ConfigFileMeta = {
  name: string;
  label: string;
  group: "core" | "ui" | "data";
  role: string;
};

const CONFIG_FILES: ConfigFileMeta[] = [
  { name: "default.custom.yaml", label: "默认方案配置", group: "core", role: "当前方案、候选数、按键绑定" },
  { name: "weasel.custom.yaml", label: "小狼毫外观配置", group: "ui", role: "主题、字号、候选窗方向" },
  { name: "rime_ice.custom.yaml", label: "雾凇组件配置", group: "ui", role: "Emoji、繁简、标点、全角开关" },
  { name: "custom_phrase.txt", label: "自定义短语", group: "data", role: "短语、编码、权重" },
  { name: "rime_ice.schema.yaml", label: "雾凇方案", group: "core", role: "雾凇输入方案入口" },
  { name: "rime_ice.dict.yaml", label: "雾凇主词库", group: "data", role: "雾凇基础词条" },
  { name: "rime_ice_ext.dict.yaml", label: "雾凇扩展词库", group: "data", role: "扩展词条" },
  { name: "sogou_ext.dict.yaml", label: "搜狗扩展词库", group: "data", role: "导入词库条目" },
];

const filesByName = computed(() => {
  return new Map((props.env?.custom_files ?? []).map((file) => [file.name, file]));
});

const rows = computed(() => {
  return CONFIG_FILES.map((meta) => ({
    ...meta,
    file: filesByName.value.get(meta.name),
  }));
});

const existingCount = computed(() => rows.value.filter((row) => row.file?.exists).length);
const missingCount = computed(() => rows.value.length - existingCount.value);
const groups = computed(() => [
  { key: "core", title: "核心配置", rows: rows.value.filter((row) => row.group === "core") },
  { key: "ui", title: "外观与组件", rows: rows.value.filter((row) => row.group === "ui") },
  { key: "data", title: "短语与词库", rows: rows.value.filter((row) => row.group === "data") },
]);

function formatBytes(size?: number) {
  if (size === undefined) return "无文件";
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
  return `${(size / 1024 / 1024).toFixed(1)} MB`;
}

function formatModified(modified?: number) {
  if (!modified) return "从未生成";
  return new Date(modified * 1000).toLocaleString();
}

async function openConfigFile(file: FileStatus | undefined) {
  if (!file?.exists) {
    ElMessage.warning("这个文件还没有生成");
    return;
  }

  await withErrorHandling(() => invoke("open_config_file", { name: file.name }));
}

async function openRimeDir() {
  await withErrorHandling(() => invoke("open_rime_user_dir"));
}
</script>

<template>
  <section class="content-grid config-files-grid">
    <section class="main-column">
      <div class="config-files-hero panel">
        <div>
          <span>Rime 用户目录</span>
          <strong>{{ env?.user_dir ?? "等待扫描" }}</strong>
          <small>集中查看 Rime Studio 会读写的关键配置文件。</small>
        </div>
        <div class="config-files-actions">
          <el-button :icon="Refresh" @click="emit('refresh')">刷新</el-button>
          <el-button :icon="FolderOpened" @click="openRimeDir">打开目录</el-button>
          <el-button type="primary" plain :icon="UploadFilled" :loading="backingUp" @click="emit('createBackup')">
            手动备份
          </el-button>
        </div>
      </div>

      <section v-for="group in groups" :key="group.key" class="config-file-section">
        <header>
          <h3>{{ group.title }}</h3>
          <span>{{ group.rows.filter((row) => row.file?.exists).length }} / {{ group.rows.length }}</span>
        </header>

        <div class="config-file-list">
          <article
            v-for="row in group.rows"
            :key="row.name"
            class="config-file-row"
            :class="{ missing: !row.file?.exists }"
          >
            <div class="config-file-main">
              <el-icon><Document /></el-icon>
              <div>
                <strong>{{ row.label }}</strong>
                <code>{{ row.name }}</code>
                <p>{{ row.role }}</p>
              </div>
            </div>

            <div class="config-file-meta">
              <span :class="{ ok: row.file?.exists }">{{ row.file?.exists ? "存在" : "未生成" }}</span>
              <small>{{ formatBytes(row.file?.size) }}</small>
              <small>{{ formatModified(row.file?.modified) }}</small>
            </div>

            <el-button
              link
              :icon="FolderOpened"
              :disabled="!row.file?.exists"
              @click="openConfigFile(row.file)"
            >
              定位
            </el-button>
          </article>
        </div>
      </section>
    </section>

    <aside class="side-column">
      <el-card class="panel config-files-summary" shadow="never">
        <template #header>
          <span>文件状态</span>
        </template>
        <div class="health-list">
          <div>
            <span>已生成</span>
            <strong>{{ existingCount }}</strong>
          </div>
          <div>
            <span>未生成</span>
            <strong>{{ missingCount }}</strong>
          </div>
          <div>
            <span>用户目录</span>
            <strong>{{ env?.user_dir ? "已连接" : "等待扫描" }}</strong>
          </div>
        </div>
      </el-card>

      <el-card class="panel" shadow="never">
        <template #header>
          <span>使用建议</span>
        </template>
        <p class="config-files-note">
          普通保存会自动创建保存前备份。需要标记重要节点时，可以额外手动备份。
        </p>
      </el-card>
    </aside>
  </section>
</template>
