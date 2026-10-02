<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { ElMessage } from "element-plus";
import { api } from "../../api";
import { useErrorHandler } from "../../composables/useErrorHandler";
import {
  formatSafeDiagnostics,
  type DiagnosticReport,
  type ConfigDiagnostic,
} from "../../utils/diagnostics";

const router = useRouter();
const { withErrorHandling } = useErrorHandler();
const visible = ref(false);
const loading = ref(false);
const report = ref<DiagnosticReport>();
const safeReport = computed(() => (report.value ? formatSafeDiagnostics(report.value) : ""));

async function inspect() {
  if (loading.value) return;
  visible.value = true;
  loading.value = true;
  try {
    report.value = await withErrorHandling(() => api.getDiagnosticReport());
  } finally {
    loading.value = false;
  }
}
async function copy() {
  try {
    await navigator.clipboard.writeText(safeReport.value);
    ElMessage.success("已复制脱敏诊断信息，可粘贴到 issue");
  } catch {
    ElMessage.warning("复制失败，请选中下方报告手动复制");
  }
}
async function openIssue(issue: ConfigDiagnostic) {
  await router.push({ path: "/editor", query: { file: issue.filename, line: issue.line ?? 1 } });
  visible.value = false;
}
</script>

<template>
  <el-button size="small" :loading="loading" @click="inspect">环境诊断</el-button>
  <el-dialog
    v-model="visible"
    top="5vh"
    title="环境诊断与反馈"
    width="min(820px, 94vw)"
    append-to-body
  >
    <div v-loading="loading" class="diagnostic-content">
      <template v-if="report">
        <h3>小狼毫识别来源</h3>
        <p>按手动指定、注册表、安装目录、开始菜单顺序，使用第一个可用部署器。</p>
        <el-table :data="report.candidates" max-height="240">
          <el-table-column prop="source" label="来源" width="130" />
          <el-table-column prop="path" label="本机路径" min-width="220" show-overflow-tooltip />
          <el-table-column prop="reason" label="结果" min-width="160" />
        </el-table>
        <h3>配置检查</h3>
        <p>
          已检查 {{ report.files_checked }} 个文件，发现
          {{ report.issues.length }}
          个诊断项。检查覆盖用户目录和小狼毫共享目录中的方案及词库依赖，不执行 Lua。
        </p>
        <ul class="diagnostic-issues">
          <li v-for="(issue, index) in report.issues" :key="index">
            <strong>{{ issue.filename }}</strong>
            <span v-if="issue.line"> · 第 {{ issue.line }} 行，第 {{ issue.column }} 列</span>
            <p>{{ issue.message }}</p>
            <el-button
              v-if="issue.code === 'yaml_parse' && issue.editable"
              size="small"
              @click="openIssue(issue)"
              >定位到编辑器</el-button
            >
          </li>
        </ul>
        <h3>可复制的脱敏报告</h3>
        <pre class="diagnostic-report">{{ safeReport }}</pre>
      </template>
      <el-empty v-else-if="!loading" description="诊断未完成，请重试" />
    </div>
    <template #footer>
      <el-button :disabled="loading" @click="inspect">重新检查</el-button>
      <el-button type="primary" :disabled="loading || !report" @click="copy"
        >复制脱敏报告</el-button
      >
    </template>
  </el-dialog>
</template>

<style scoped>
.diagnostic-content {
  max-height: 65vh;
  overflow: auto;
}
.diagnostic-issues {
  padding-left: 20px;
}
.diagnostic-issues li {
  margin-bottom: 14px;
  overflow-wrap: anywhere;
}
.diagnostic-report {
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  max-height: 240px;
  overflow: auto;
  padding: 12px;
  background: var(--color-surface-soft);
}
</style>
