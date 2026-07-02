<script setup lang="ts">
import { computed } from "vue";
import {
  Brush,
  Check,
  Connection,
  Download,
  FolderOpened,
  Open,
  Refresh,
  RefreshLeft,
  Setting,
  Warning,
} from "@element-plus/icons-vue";
import type { BackupEntry, FileStatus, RimeEnvironment } from "../types";

const props = defineProps<{
  env?: RimeEnvironment;
  backups: BackupEntry[];
  log: string;
  scanning: boolean;
  backingUp: boolean;
  restoringBackup?: string;
  installingRecipe?: string;
}>();

const emit = defineEmits<{
  createBackup: [];
  openPath: [command: "open_rime_user_dir" | "open_plum_dir"];
  install: [recipe: string];
  openBackup: [backup: BackupEntry];
  restoreBackup: [backup: BackupEntry];
}>();

const toolsReady = computed(() => Boolean(props.env?.git_available && props.env?.bash_available));
const hasDeployer = computed(() => Boolean(props.env?.deployer_path));
const health = computed(() => props.env?.sogou_health);
const customFiles = computed(() => props.env?.custom_files ?? []);
const foundFiles = computed(() => customFiles.value.filter((file) => file.exists).length);
const missingFiles = computed(() => customFiles.value.length - foundFiles.value);
const fileCompletion = computed(() => {
  if (!customFiles.value.length) return 0;
  return Math.round((foundFiles.value / customFiles.value.length) * 100);
});
const readinessItems = computed(() => [
  {
    label: "部署器",
    ready: hasDeployer.value,
    detail: props.env?.deployer_path ?? "未找到 WeaselDeployer.exe",
  },
  {
    label: "Git",
    ready: Boolean(props.env?.git_available),
    detail: props.env?.git_path ?? "未找到 Git",
  },
  {
    label: "Git Bash",
    ready: Boolean(props.env?.bash_available),
    detail: props.env?.bash_path ?? "未找到 Git Bash",
  },
]);
const installRecipes = [
  {
    name: "完整配置",
    recipe: "iDvel/rime-ice:others/recipes/full",
    description: "安装或更新雾凇拼音完整方案",
  },
  {
    name: "词库更新",
    recipe: "iDvel/rime-ice:others/recipes/all_dicts",
    description: "仅同步 rime-ice 词库资源",
  },
];

function formatBackupTime(value?: number) {
  if (!value) return "未知时间";
  return new Intl.DateTimeFormat("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(value * 1000));
}

function formatBytes(value?: number) {
  if (value === undefined) return "未找到";
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  return `${(value / 1024 / 1024).toFixed(1)} MB`;
}
</script>

<template>
  <div class="overview-page">
    <el-alert
      v-if="!hasDeployer && env"
    class="setup-alert"
    type="warning"
    show-icon
    title="未找到 WeaselDeployer.exe"
    description="当前只能扫描和安装配置，重新部署需要先安装小狼毫或补充部署器路径。"
    :closable="false"
  />

  <section class="compact-status">
    <div class="status-cell primary">
      <el-icon><Setting /></el-icon>
      <span>方案</span>
      <strong>{{ env?.active_schema ?? "未知" }}</strong>
    </div>
    <div class="status-cell">
      <span>主题</span>
      <strong>{{ env?.theme_name ?? "未知" }}</strong>
    </div>
    <div class="status-cell">
      <span>候选</span>
      <strong>{{ env?.page_size ?? "?" }} 项</strong>
    </div>
    <div class="status-cell">
      <span>字体</span>
      <strong>{{ env?.font_point ?? "?" }} / {{ env?.label_font_point ?? "?" }}</strong>
    </div>
    <div class="status-cell">
      <span>文件</span>
      <strong>{{ foundFiles }} / {{ customFiles.length }}</strong>
    </div>
    <div class="status-cell">
      <span>工具</span>
      <el-tag :type="toolsReady ? 'success' : 'warning'" effect="light" size="small">
        {{ toolsReady ? "就绪" : "缺失" }}
      </el-tag>
    </div>
  </section>

  <section class="content-grid overview-compact-grid">
    <section class="main-column">
      <el-card class="panel overview-panel compact-panel" shadow="never">
        <template #header>
          <div class="panel-title">
            <span>配置状态</span>
            <el-tag :type="missingFiles ? 'warning' : 'success'" effect="light">
              {{ missingFiles ? `${missingFiles} 个缺失` : "文件齐全" }}
            </el-tag>
          </div>
        </template>

        <el-skeleton v-if="!env && scanning" :rows="5" animated />
        <div v-else class="status-table">
          <div v-for="item in readinessItems" :key="item.label" class="status-row">
            <el-icon :class="item.ready ? 'ok-icon' : 'warn-icon'">
              <Check v-if="item.ready" />
              <Warning v-else />
            </el-icon>
            <span>{{ item.label }}</span>
            <code>{{ item.detail }}</code>
          </div>
        </div>
      </el-card>

      <el-card class="panel file-panel compact-panel" shadow="never">
        <template #header>
          <div class="panel-title">
            <span>受管理文件</span>
            <span class="panel-caption">{{ env?.build_dir ?? "build 目录待扫描" }}</span>
          </div>
        </template>
        <el-table :data="customFiles" stripe>
          <el-table-column label="文件" min-width="280">
            <template #default="{ row }: { row: FileStatus }">
              <strong class="file-name">{{ row.name }}</strong>
              <span class="mono-path file-path">{{ row.path }}</span>
            </template>
          </el-table-column>
          <el-table-column label="状态" width="130">
            <template #default="{ row }: { row: FileStatus }">
              <el-tag :type="row.exists ? 'success' : 'warning'" effect="light">
                {{ row.exists ? "Found" : "Missing" }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="大小" width="140">
            <template #default="{ row }: { row: FileStatus }">
              {{ formatBytes(row.size) }}
            </template>
          </el-table-column>
        </el-table>
      </el-card>

      <el-card v-if="log" class="log-panel" shadow="never">
        <template #header>
          <span>安装日志</span>
        </template>
        <pre>{{ log }}</pre>
      </el-card>
    </section>

    <aside class="side-column">
      <el-card class="panel quick-panel compact-panel" shadow="never">
        <template #header>
          <span>快捷操作</span>
        </template>
        <div class="quick-actions">
          <el-button
            type="primary"
            plain
            :loading="backingUp"
            :icon="FolderOpened"
            @click="emit('createBackup')"
          >
            创建备份
          </el-button>
          <el-button :icon="Open" @click="emit('openPath', 'open_rime_user_dir')">
            打开用户目录
          </el-button>
        </div>
      </el-card>

      <el-card class="panel action-panel compact-panel" shadow="never">
        <template #header>
          <div class="panel-title">
            <span>安装 rime-ice</span>
            <el-tag :type="toolsReady ? 'success' : 'warning'" effect="light">
              {{ toolsReady ? "可安装" : "工具缺失" }}
            </el-tag>
          </div>
        </template>

        <div class="recipe-list">
          <button
            v-for="recipe in installRecipes"
            :key="recipe.recipe"
            class="recipe-card"
            :disabled="!toolsReady || Boolean(installingRecipe)"
            @click="emit('install', recipe.recipe)"
          >
            <span>
              <strong>{{ recipe.name }}</strong>
              <small>{{ recipe.description }}</small>
            </span>
            <el-icon v-if="installingRecipe === recipe.recipe" class="is-loading">
              <Refresh />
            </el-icon>
            <el-icon v-else><Download /></el-icon>
          </button>
        </div>
      </el-card>

      <el-card class="panel backup-panel compact-panel" shadow="never">
        <template #header>
          <div class="panel-title">
            <span>最近备份</span>
            <el-tag effect="light">{{ backups.length }}</el-tag>
          </div>
        </template>
        <el-empty v-if="!backups.length" :image-size="64" description="还没有备份" />
        <div v-else class="backup-list">
          <div v-for="backup in backups.slice(0, 4)" :key="backup.path" class="backup-item">
            <div class="backup-main">
              <span>
                <strong>{{ backup.name.replace("backup-rime-studio-", "") }}</strong>
                <small>{{ formatBackupTime(backup.modified) }} · {{ backup.files }} 个文件</small>
              </span>
              <div class="backup-actions">
                <el-button link type="primary" :icon="Open" @click="emit('openBackup', backup)">
                  打开
                </el-button>
                <el-button
                  link
                  type="warning"
                  :icon="RefreshLeft"
                  :loading="restoringBackup === backup.name"
                  @click="emit('restoreBackup', backup)"
                >
                  恢复
                </el-button>
              </div>
            </div>
            <code>{{ backup.path }}</code>
          </div>
          <div v-if="backups.length > 4" class="backup-more">
            还有 {{ backups.length - 4 }} 个备份，到「备份管理」页查看全部
          </div>
        </div>
      </el-card>

      <el-card class="panel dictionary-panel compact-panel" shadow="never">
        <template #header>
          <span>词库健康</span>
        </template>
        <div class="health-list">
          <div>
            <span>条目</span>
            <strong>{{ health?.entries ?? 0 }}</strong>
          </div>
          <div>
            <span>重复行</span>
            <strong>{{ health?.duplicate_exact_lines ?? 0 }}</strong>
          </div>
          <div>
            <span>长低权重项</span>
            <strong>{{ health?.long_low_weight_entries ?? 0 }}</strong>
          </div>
        </div>
      </el-card>

      <el-card class="panel path-panel compact-panel" shadow="never">
        <template #header>
          <div class="panel-title">
            <span>Plum 目录</span>
            <el-button link type="primary" :icon="Open" @click="emit('openPath', 'open_plum_dir')">
              打开
            </el-button>
          </div>
        </template>
        <div class="path-chip">
          <el-icon><Connection /></el-icon>
          <span>{{ env?.plum_dir ?? "未知" }}</span>
        </div>
      </el-card>
    </aside>
  </section>
  </div>
</template>
