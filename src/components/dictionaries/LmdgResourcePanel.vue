<script setup lang="ts">
import { computed } from "vue";
import { Collection, Delete, Download, Refresh } from "@element-plus/icons-vue";
import { formatBytes } from "../../utils";
import type {
  LmdgGrammarInstallResult,
  LmdgGrammarUninstallResult,
  LmdgInstallResult,
} from "../../types";

type LmdgDownloadProgress = {
  kind: "grammar" | "dicts";
  stage: string;
  downloaded_bytes: number;
  total_bytes?: number;
  percent?: number;
};

const props = defineProps<{
  dictInstalling: boolean;
  grammarInstalling: boolean;
  grammarUninstalling: boolean;
  loading: boolean;
  progress?: LmdgDownloadProgress;
  dictResult?: LmdgInstallResult;
  grammarResult?: LmdgGrammarInstallResult;
  grammarUninstallResult?: LmdgGrammarUninstallResult;
}>();

const emit = defineEmits<{
  installDicts: [];
  installGrammar: [];
  uninstallGrammar: [];
  refresh: [];
}>();

const progressPercentage = computed(() =>
  Math.max(0, Math.min(100, Math.round(props.progress?.percent ?? 0))),
);

const progressText = computed(() => {
  const progress = props.progress;
  if (!progress) return "";
  const downloaded = formatBytes(progress.downloaded_bytes);
  if (progress.total_bytes) {
    return `${progress.stage} · ${downloaded} / ${formatBytes(progress.total_bytes)}`;
  }
  return `${progress.stage} · 已下载 ${downloaded}`;
});

const showProgress = computed(() =>
  (props.grammarInstalling || props.dictInstalling) && Boolean(props.progress),
);
</script>

<template>
  <section class="lmdg-resource-panel">
    <div class="lmdg-resource-copy">
      <el-tag type="success" size="small">高级资源</el-tag>
      <div>
        <strong>万象语言模型 RIME-LMDG</strong>
        <small>
          下载 wanxiang-lts-zh-hans.gram，并为雾凇拼音写入 octagram 语法模型补丁。保留雾凇方案，只增强长句排序。
        </small>
      </div>
    </div>
    <div class="lmdg-resource-actions">
      <el-button
        type="success"
        :icon="Download"
        :loading="grammarInstalling"
        @click="emit('installGrammar')"
      >
        安装模型
      </el-button>
      <el-button
        type="warning"
        plain
        :icon="Delete"
        :loading="grammarUninstalling"
        @click="emit('uninstallGrammar')"
      >
        卸载模型
      </el-button>
      <el-button
        :icon="Collection"
        :loading="dictInstalling"
        @click="emit('installDicts')"
      >
        安装词库包
      </el-button>
      <el-button :icon="Refresh" :loading="loading" @click="emit('refresh')">
        刷新本地词库
      </el-button>
    </div>
    <div v-if="showProgress" class="lmdg-download-progress">
      <div>
        <span>{{ progressText }}</span>
        <strong v-if="progress?.percent !== undefined">
          {{ progressPercentage }}%
        </strong>
      </div>
      <el-progress
        :percentage="progressPercentage"
        :show-text="false"
        :indeterminate="progress?.percent === undefined"
      />
    </div>
    <div v-if="grammarResult" class="lmdg-resource-status">
      <span>{{ grammarResult.message }}</span>
      <small>{{ grammarResult.model_path }}</small>
      <small>{{ grammarResult.patch_path }}</small>
    </div>
    <div v-if="grammarUninstallResult" class="lmdg-resource-status">
      <span>{{ grammarUninstallResult.message }}</span>
      <small>{{ grammarUninstallResult.model_path }}</small>
      <small>{{ grammarUninstallResult.patch_path }}</small>
    </div>
    <div v-if="dictResult" class="lmdg-resource-status">
      <span>{{ dictResult.message }}</span>
      <small>{{ dictResult.target_dir }}</small>
    </div>
  </section>
</template>
