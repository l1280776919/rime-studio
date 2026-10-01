<script setup lang="ts">
import { Check, RefreshLeft, UploadFilled } from "@element-plus/icons-vue";

defineProps<{
  ready: boolean;
  dirty: boolean;
  loading: boolean;
  saving: boolean;
  deploying: boolean;
  busy: boolean;
  hasDeployer: boolean;
}>();
defineEmits<{ save: []; deploy: []; reset: []; retry: [] }>();
</script>

<template>
  <div class="settings-save-bar" :class="{ dirty }">
    <div class="save-summary" role="status" aria-live="polite">
      <strong>{{
        deploying
          ? "正在部署"
          : saving
            ? "正在保存"
            : loading
              ? "正在读取配置"
              : !ready
                ? "配置读取失败"
                : dirty
                  ? "有未保存修改"
                  : "与已保存文件一致"
      }}</strong>
      <span>{{
        !ready && !loading
          ? "请刷新重试，读取成功后即可编辑。"
          : !hasDeployer
            ? "未检测到部署工具，可先保存，再到概览检查安装。"
            : "保存写入文件；部署后在输入法中生效。"
      }}</span>
    </div>
    <div class="save-actions">
      <el-button v-if="!ready && !loading" :disabled="busy" @click="$emit('retry')"
        >重新读取</el-button
      >
      <el-button
        :icon="RefreshLeft"
        :disabled="!ready || !dirty || loading || saving || busy"
        @click="$emit('reset')"
        >撤销未保存修改</el-button
      >
      <el-button
        :icon="Check"
        :loading="saving"
        :disabled="!ready || !dirty || loading || busy"
        title="仅保存到文件（Ctrl+S）"
        @click="$emit('save')"
        >保存 <kbd>Ctrl+S</kbd></el-button
      >
      <el-button
        type="primary"
        :icon="UploadFilled"
        :loading="deploying"
        :disabled="!ready || loading || saving || busy || !hasDeployer"
        :title="
          !hasDeployer ? '未检测到小狼毫部署工具，请在概览中检查安装' : '部署已保存的配置到输入法'
        "
        @click="$emit('deploy')"
        >{{ dirty ? "保存并部署" : "部署已保存配置" }}</el-button
      >
    </div>
  </div>
</template>

<style scoped>
.settings-save-bar {
  position: sticky;
  top: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 12px;
  padding: 12px 16px;
  border: 1px solid var(--color-line);
  border-radius: var(--radius-lg);
  background: var(--color-surface);
  box-shadow: var(--shadow-sm);
}
.settings-save-bar.dirty {
  border-color: var(--brand-400);
}
.save-summary {
  display: flex;
  flex-direction: column;
  gap: 4px;
  color: var(--ink-800);
  font-size: 13px;
}
.save-summary span {
  color: var(--ink-500);
  font-size: 12px;
}
.save-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.save-actions .el-button {
  margin-left: 0;
}
kbd {
  margin-left: 6px;
  color: var(--ink-500);
  font-size: 10px;
}
@media (max-width: 900px) {
  .settings-save-bar {
    position: static;
  }
  .save-actions {
    width: 100%;
  }
}
</style>
