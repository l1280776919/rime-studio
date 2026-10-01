<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import type { DeployResult } from "../../types";
const props = defineProps<{ result?: DeployResult; busy: boolean; hasDeployer: boolean }>();
defineEmits<{ retry: []; navigate: [page: string] }>();
const dismissed = ref(false);
const showDetails = ref(false);
const severity = computed(() =>
  !props.result?.success ? "error" : props.result.hints?.length ? "warning" : "success",
);
watch(
  () => props.result,
  () => {
    dismissed.value = false;
    showDetails.value = false;
  },
);
async function copyDetails() {
  const result = props.result;
  if (!result) return;
  try {
    await navigator.clipboard.writeText(
      [result.message, ...(result.hints ?? []), result.log ?? ""].filter(Boolean).join("\n\n"),
    );
    ElMessage.success("已复制本次部署详情");
  } catch {
    ElMessage.warning("复制失败，可在详情中选中文本复制");
  }
}
</script>
<template>
  <section
    v-if="result && !dismissed"
    class="deployment-result"
    :class="severity"
    :role="result.success ? 'status' : 'alert'"
  >
    <div class="result-summary">
      <strong>{{
        !result.success ? "部署未完成" : result.hints?.length ? "部署完成，有提醒" : "部署完成"
      }}</strong>
      <span :title="result.message">{{ result.message }}</span>
    </div>
    <div class="result-actions">
      <el-button
        v-if="!result.success"
        size="small"
        type="primary"
        :disabled="busy || !hasDeployer"
        title="重试已保存到文件的配置"
        @click="$emit('retry')"
        >重试部署</el-button
      >
      <el-button
        v-if="!result.success || result.hints?.length"
        size="small"
        @click="$emit('navigate', 'quick')"
        >检查配置</el-button
      >
      <el-button v-if="!result.success" size="small" @click="$emit('navigate', 'backups')"
        >查看备份</el-button
      >
      <el-button size="small" @click="showDetails = true">查看详情</el-button>
      <el-button link aria-label="关闭本次部署结果" @click="dismissed = true">关闭</el-button>
    </div>
  </section>
  <el-dialog v-model="showDetails" title="本次部署详情" width="760px" append-to-body>
    <template v-if="result">
      <p>{{ result.message }}</p>
      <ul v-if="result.hints?.length">
        <li v-for="(hint, index) in result.hints" :key="index">{{ hint }}</li>
      </ul>
      <p class="helper-text">重试部署只使用已保存到文件的配置。备份恢复前会先创建安全快照。</p>
      <pre v-if="result.log" class="result-log">{{ result.log }}</pre>
      <el-empty v-else description="本次部署没有附带详细日志" :image-size="64" />
    </template>
    <template #footer
      ><el-button @click="copyDetails">复制详情</el-button
      ><el-button type="primary" @click="showDetails = false">完成</el-button></template
    >
  </el-dialog>
</template>
<style scoped>
.deployment-result {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  flex: 0 0 auto;
  gap: 10px;
  padding: 10px 14px;
  margin-bottom: 12px;
  border: 1px solid var(--el-color-success-light-5);
  border-left: 4px solid var(--el-color-success);
  border-radius: var(--radius-md);
  background: var(--color-surface);
}
.deployment-result.error {
  border-color: var(--el-color-danger-light-5);
  border-left-color: var(--el-color-danger);
}
.deployment-result.warning {
  border-color: var(--el-color-warning-light-5);
  border-left-color: var(--el-color-warning);
}
.result-summary {
  flex: 1;
  min-width: 180px;
  display: flex;
  flex-direction: column;
  gap: 3px;
  color: var(--ink-800);
  font-size: 12px;
}
.result-summary span {
  overflow-wrap: anywhere;
  color: var(--ink-600);
  max-height: 48px;
  overflow: auto;
}
.result-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px;
}
.result-actions .el-button {
  margin-left: 0;
}
.result-log {
  padding: 14px;
  max-height: 45vh;
  overflow: auto;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  background: var(--color-surface-soft);
  border-radius: var(--radius-md);
  color: var(--ink-800);
  font-size: 12px;
}
</style>
