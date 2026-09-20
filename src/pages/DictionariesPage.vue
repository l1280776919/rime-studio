<script setup lang="ts">
import { computed, ref } from "vue";
import { formatBytes, formatTime } from "../utils";
import { paginateItems } from "../utils/phrases";
import { useDictionaries } from "../composables/useDictionaries";
import DictionaryImportPreviewDialog from "../components/dictionaries/DictionaryImportPreviewDialog.vue";
import DictionaryUrlImportDialog from "../components/dictionaries/DictionaryUrlImportDialog.vue";
import OnlineDictionaryDialog from "../components/dictionaries/OnlineDictionaryDialog.vue";
import {
  Bottom,
  Collection,
  Delete,
  Download,
  Files,
  FolderOpened,
  InfoFilled,
  Link,
  MagicStick,
  Open,
  Refresh,
  Top,
  UploadFilled,
  Warning,
} from "@element-plus/icons-vue";
import type { DictionaryReference, DictInfo, RimeEnvironment } from "../types";

const _props = defineProps<{
  env?: RimeEnvironment;
}>();

const emit = defineEmits<{
  openPath: [command: "open_rime_user_dir"];
  deploy: [];
}>();

const {
  dictionaries,
  dictConfig,
  loading,
  importing,
  exportingDict,
  expandedDict,
  dictHealth,
  healthLoading,
  deletingDict,
  updatingReference,
  cleaningDict,
  importPreview,
  importUrl,
  importUrlSourceName,
  showImportPreviewDialog,
  showUrlImportDialog,
  showOnlineDictionaryDialog,
  onlineDictionaries,
  onlineCategories,
  categoryDictionaries,
  selectedOnlineCategory,
  onlineLoading,
  categoryLoading,
  onlineImporting,
  lmdgInstalling,
  lmdgResult,
  lmdgGrammarInstalling,
  lmdgGrammarUninstalling,
  lmdgGrammarResult,
  lmdgGrammarUninstallResult,
  lmdgDownloadProgress,
  toggleHealth,
  openFileLocation,
  referenceToDictInfo,
  dictNameToReference,
  chooseImportFile,
  importDictionary,
  loadOnlineDictionaries,
  loadCategoryDictionaries,
  installLmdgDictionaries,
  installLmdgGrammar,
  uninstallLmdgGrammar,
  previewOnlineDictionary,
  previewUrlDictionary,
  confirmDictionaryImport,
  exportDictionary,
  addDictionaryReference,
  removeDictionaryReference,
  moveReference,
  deleteDictionary,
  cleanDuplicateLines,
  selectOnlineCategory,
  loadAllStats,
  totalEntries,
  totalSize,
  enabledCount,
} = useDictionaries(emit);

const availablePage = ref(1);
const availablePageSize = ref(40);
const pagedAvailable = computed(() =>
  paginateItems(dictConfig.value?.available ?? [], availablePage.value, availablePageSize.value),
);
</script>

<template>
  <div class="dictionaries-hub-container">
    <!-- Bento Metrics Stage -->
    <div class="dict-bento-metrics">
      <div class="metric-card card-accent">
        <div class="metric-icon-box">
          <el-icon><Collection /></el-icon>
        </div>
        <div class="metric-body">
          <span class="metric-label">启用词库 / 总词库</span>
          <strong class="metric-value">{{ enabledCount }} <span class="metric-total">/ {{ dictionaries.length }}</span></strong>
        </div>
      </div>

      <div class="metric-card">
        <div class="metric-icon-box">
          <el-icon><InfoFilled /></el-icon>
        </div>
        <div class="metric-body">
          <span class="metric-label">收录总词条</span>
          <strong class="metric-value">{{ totalEntries.toLocaleString() }}</strong>
        </div>
      </div>

      <div class="metric-card">
        <div class="metric-icon-box">
          <el-icon><FolderOpened /></el-icon>
        </div>
        <div class="metric-body">
          <span class="metric-label">词库总容量</span>
          <strong class="metric-value">{{ formatBytes(totalSize) }}</strong>
        </div>
      </div>

      <div class="metric-card">
        <div class="metric-icon-box">
          <el-icon><Warning /></el-icon>
        </div>
        <div class="metric-body">
          <span class="metric-label">当前方案</span>
          <strong class="metric-value truncate" :title="dictConfig?.schema_name ?? dictConfig?.schema_id">
            {{ dictConfig?.schema_name ?? dictConfig?.schema_id ?? "未识别" }}
          </strong>
        </div>
      </div>

      <div class="metric-card">
        <div class="metric-icon-box">
          <el-icon><Files /></el-icon>
        </div>
        <div class="metric-body">
          <span class="metric-label">主词库文件</span>
          <strong class="metric-value truncate" :title="dictConfig?.main_dictionary ? `${dictConfig.main_dictionary}.dict.yaml` : '未配置'">
            {{ dictConfig?.main_dictionary ? `${dictConfig.main_dictionary}.dict.yaml` : "未配置" }}
          </strong>
        </div>
      </div>
    </div>

    <!-- Action Bar -->
    <div class="dict-action-bar panel">
      <div class="action-bar-left">
        <input
          ref="fileInput"
          type="file"
          accept=".bin,.scel,.txt,.dict.yaml,.yaml"
          style="display: none"
          @change="importDictionary"
        />
        <el-button
          type="primary"
          class="deploy-btn"
          :icon="UploadFilled"
          :loading="importing"
          @click="chooseImportFile"
        >
          导入词库文件
        </el-button>

        <el-button
          type="success"
          plain
          :icon="Download"
          @click="showOnlineDictionaryDialog = true"
        >
          社区在线词库市场
        </el-button>

        <el-button
          :icon="Link"
          :loading="importing"
          @click="showUrlImportDialog = true"
        >
          URL 在线导入
        </el-button>
      </div>

      <div class="action-bar-right">
        <el-button :icon="Refresh" :loading="loading" @click="loadAllStats">
          刷新统计
        </el-button>
        <el-button :icon="FolderOpened" @click="emit('openPath', 'open_rime_user_dir')">
          打开词库目录
        </el-button>
        <el-button type="primary" :icon="UploadFilled" @click="emit('deploy')">
          部署生效
        </el-button>
      </div>
    </div>

    <!-- Main Content Layout -->
    <div class="dict-workbench-grid">
      <div class="dict-main-column">
        <!-- Panel 1: Enabled Dictionaries in Current Schema -->
        <el-card class="panel dict-panel" shadow="never">
          <template #header>
            <div class="dict-panel-header">
              <div class="panel-heading-group">
                <div class="panel-icon-dot" />
                <h3 class="panel-heading-title">当前方案启用词库 (import_tables)</h3>
              </div>
              <div class="header-tags">
                <span class="tag-pill-accent">
                  主词库：{{ dictConfig?.main_dictionary ? `${dictConfig.main_dictionary}.dict.yaml` : "未识别" }}
                </span>
                <span class="count-capsule">{{ enabledCount }} 项启用</span>
              </div>
            </div>
          </template>

          <el-empty
            v-if="!loading && !dictConfig?.enabled.length && !dictConfig?.missing.length"
            description="当前方案尚未配置 import_tables 扩展词库"
            :image-size="64"
          />

          <el-table
            v-else
            v-loading="loading"
            :data="[...(dictConfig?.enabled ?? []), ...(dictConfig?.missing ?? [])]"
            stripe
            class="dict-clean-table"
            max-height="360"
          >
            <el-table-column label="优先级 / 词库引用" min-width="240">
              <template #default="{ row, $index }: { row: DictionaryReference; $index: number }">
                <div class="dict-ref-cell">
                  <span class="priority-badge" :class="{ 'is-top': $index === 0 }">
                    #{{ $index + 1 }}
                  </span>
                  <div class="ref-name-wrap">
                    <span class="dict-ref-name">{{ row.reference }}</span>
                    <small v-if="$index === 0" class="top-hint">优先级最高</small>
                  </div>
                  <span v-if="!row.exists" class="status-tag tag-missing">缺失文件</span>
                  <span v-else class="status-tag tag-enabled">生效中</span>
                </div>
              </template>
            </el-table-column>

            <el-table-column label="条目数" width="120" align="right">
              <template #default="{ row }: { row: DictionaryReference }">
                <span class="entry-num">{{ row.entry_count?.toLocaleString() ?? "—" }}</span>
              </template>
            </el-table-column>

            <el-table-column label="大小" width="100" align="right">
              <template #default="{ row }: { row: DictionaryReference }">
                <span class="size-num">{{ formatBytes(row.size_bytes) }}</span>
              </template>
            </el-table-column>

            <el-table-column label="排序与管理" width="220" align="center">
              <template #default="{ row, $index }: { row: DictionaryReference; $index: number }">
                <div class="row-action-btns">
                  <el-button
                    link
                    size="small"
                    :icon="Top"
                    :disabled="$index === 0"
                    title="上移优先级"
                    @click.stop="moveReference(row.reference, -1)"
                  >
                    上移
                  </el-button>
                  <el-button
                    link
                    size="small"
                    :icon="Bottom"
                    :disabled="$index >= enabledCount - 1"
                    title="下移优先级"
                    @click.stop="moveReference(row.reference, 1)"
                  >
                    下移
                  </el-button>
                  <el-button
                    v-if="row.exists"
                    link
                    size="small"
                    :icon="Download"
                    :loading="exportingDict === `${row.reference}.dict.yaml`"
                    title="导出词库"
                    @click.stop="exportDictionary(referenceToDictInfo(row))"
                  >
                    导出
                  </el-button>
                  <el-button
                    link
                    size="small"
                    type="danger"
                    :loading="updatingReference === row.reference"
                    title="从方案移除"
                    @click.stop="removeDictionaryReference(row.reference)"
                  >
                    移除
                  </el-button>
                </div>
              </template>
            </el-table-column>
          </el-table>
        </el-card>

        <!-- Panel 2: Available Dictionaries in User Dir -->
        <el-card class="panel dict-panel" shadow="never">
          <template #header>
            <div class="dict-panel-header">
              <div class="panel-heading-group">
                <div class="panel-icon-dot gray" />
                <h3 class="panel-heading-title">未启用 / 本地候选词库库</h3>
              </div>
              <span class="count-capsule">{{ dictConfig?.available.length ?? 0 }} 个本地词库</span>
            </div>
          </template>

          <el-empty
            v-if="!loading && !dictConfig?.available.length"
            description="暂无可加入的独立 .dict.yaml 词库文件"
            :image-size="64"
          >
            <p class="helper-text" style="font-size: 12px; color: var(--color-muted)">
              您可以点击上方「导入词库文件」或从「社区在线词库市场」获取词库。
            </p>
          </el-empty>

          <template v-else>
            <el-table
              v-loading="loading"
              :data="pagedAvailable.items"
              stripe
              class="dict-clean-table"
              highlight-current-row
              max-height="360"
              @row-click="toggleHealth"
            >
              <el-table-column label="词库文件名" min-width="260">
                <template #default="{ row }: { row: DictInfo }">
                  <div class="dict-file-cell">
                    <div class="dict-file-icon">
                      <el-icon><Collection /></el-icon>
                    </div>
                    <div class="dict-file-meta">
                      <span class="file-name">{{ row.name }}</span>
                      <small class="file-date">{{ formatTime(row.modified) }}</small>
                    </div>
                  </div>
                </template>
              </el-table-column>

              <el-table-column label="条目数" width="120" align="right">
                <template #default="{ row }: { row: DictInfo }">
                  <span class="entry-num">{{ row.entry_count.toLocaleString() }}</span>
                </template>
              </el-table-column>

              <el-table-column label="大小" width="100" align="right">
                <template #default="{ row }: { row: DictInfo }">
                  <span class="size-num">{{ formatBytes(row.size_bytes) }}</span>
                </template>
              </el-table-column>

              <el-table-column label="快捷操作" width="220" align="center">
                <template #default="{ row }: { row: DictInfo }">
                  <div class="row-action-btns">
                    <el-button
                      size="small"
                      type="success"
                      plain
                      :loading="updatingReference === row.name"
                      @click.stop="addDictionaryReference(dictNameToReference(row.name))"
                    >
                      加入方案
                    </el-button>
                    <el-button
                      link
                      size="small"
                      :icon="Open"
                      title="系统资源管理器定位"
                      @click.stop="openFileLocation(row)"
                    >
                      定位
                    </el-button>
                    <el-button
                      link
                      size="small"
                      :icon="Download"
                      :loading="exportingDict === row.name"
                      title="导出文本"
                      @click.stop="exportDictionary(row)"
                    >
                      导出
                    </el-button>
                    <el-button
                      link
                      size="small"
                      type="danger"
                      :icon="Delete"
                      :loading="deletingDict === row.name"
                      title="从磁盘删除"
                      @click.stop="deleteDictionary(row)"
                    />
                  </div>
                </template>
              </el-table-column>
            </el-table>

            <div
              v-if="(dictConfig?.available.length ?? 0) > availablePageSize"
              class="dict-pagination"
            >
              <el-pagination
                v-model:current-page="availablePage"
                v-model:page-size="availablePageSize"
                :page-sizes="[40, 80, 120]"
                :total="dictConfig?.available.length ?? 0"
                layout="total, sizes, prev, pager, next"
                small
              />
            </div>

            <!-- Health Inspection Drawer Strip -->
            <Transition name="el-fade-in-linear">
              <div v-if="expandedDict && dictHealth" class="health-inspect-banner">
                <div class="health-banner-top">
                  <div class="health-title-group">
                    <el-icon><MagicStick /></el-icon>
                    <strong>{{ expandedDict }} · 健康诊断报告</strong>
                  </div>

                  <el-button
                    type="warning"
                    size="small"
                    :icon="Delete"
                    :loading="cleaningDict === expandedDict"
                    :disabled="!dictHealth.duplicate_exact_lines"
                    @click.stop="cleanDuplicateLines(expandedDict)"
                  >
                    一键智能去重
                  </el-button>
                </div>

                <div class="health-metrics-row">
                  <div class="health-pill">
                    <span class="health-pill-label">分析词条</span>
                    <strong class="health-pill-val">{{ dictHealth.entries.toLocaleString() }}</strong>
                  </div>

                  <div class="health-pill" :class="{ warn: dictHealth.duplicate_exact_lines > 0 }">
                    <span class="health-pill-label">完全重复行</span>
                    <strong class="health-pill-val">{{ dictHealth.duplicate_exact_lines.toLocaleString() }}</strong>
                  </div>

                  <div class="health-pill" :class="{ warn: dictHealth.long_low_weight_entries > 0 }">
                    <span class="health-pill-label">长低权生僻项</span>
                    <strong class="health-pill-val">{{ dictHealth.long_low_weight_entries.toLocaleString() }}</strong>
                  </div>
                </div>
              </div>

              <div v-else-if="expandedDict && healthLoading" class="health-inspect-banner">
                <el-skeleton :rows="2" animated />
              </div>
            </Transition>
          </template>
        </el-card>
      </div>

      <!-- Right Column: Format Guide & Sogou Health -->
      <aside class="dict-side-column">
        <div class="panel side-card">
          <div class="side-card-title">
            <el-icon><InfoFilled /></el-icon>
            <strong>词库格式与生态规范</strong>
          </div>
          <p class="side-card-text">
            Rime 规范词库文件名必须以 <code>.dict.yaml</code> 结尾，文件内部包含 YAML 元数据头部与 Tab 分隔的数据行（词汇 → 编码 → 权重）。
          </p>

          <div class="format-badges-list">
            <span class="fmt-badge">搜狗用户备份 .bin</span>
            <span class="fmt-badge">搜狗细胞词库 .scel</span>
            <span class="fmt-badge">纯文本词表 .txt</span>
            <span class="fmt-badge">Rime 词库 .dict.yaml</span>
          </div>

          <div class="side-card-tip">
            💡 词库导入或修改顺序后，请点击右上角「部署生效」使 Rime 输入法更新编译索引。
          </div>
        </div>

        <!-- Sogou Health Card -->
        <div v-if="env?.sogou_health" class="panel side-card">
          <div class="side-card-title">
            <el-icon><MagicStick /></el-icon>
            <strong>搜狗扩展词库健康</strong>
          </div>

          <div class="sogou-health-list">
            <div class="sogou-health-item">
              <span>词条总量</span>
              <strong>{{ env.sogou_health.entries.toLocaleString() }}</strong>
            </div>
            <div class="sogou-health-item">
              <span>完全重复行</span>
              <strong :class="{ 'warn-text': env.sogou_health.duplicate_exact_lines > 0 }">
                {{ env.sogou_health.duplicate_exact_lines.toLocaleString() }}
              </strong>
            </div>
            <div class="sogou-health-item">
              <span>长低权重项</span>
              <strong :class="{ 'warn-text': env.sogou_health.long_low_weight_entries > 0 }">
                {{ env.sogou_health.long_low_weight_entries.toLocaleString() }}
              </strong>
            </div>
          </div>
        </div>
      </aside>
    </div>

    <!-- Modals -->
    <OnlineDictionaryDialog
      v-model="showOnlineDictionaryDialog"
      :online-dictionaries="onlineDictionaries"
      :online-categories="onlineCategories"
      :category-dictionaries="categoryDictionaries"
      :selected-category="selectedOnlineCategory"
      :online-loading="onlineLoading"
      :category-loading="categoryLoading"
      :local-loading="loading"
      :importing="importing"
      :online-importing="onlineImporting"
      :dict-installing="lmdgInstalling"
      :grammar-installing="lmdgGrammarInstalling"
      :grammar-uninstalling="lmdgGrammarUninstalling"
      :lmdg-progress="lmdgDownloadProgress"
      :lmdg-result="lmdgResult"
      :lmdg-grammar-result="lmdgGrammarResult"
      :lmdg-grammar-uninstall-result="lmdgGrammarUninstallResult"
      @refresh-local="loadAllStats"
      @refresh-online="loadOnlineDictionaries"
      @refresh-category="loadCategoryDictionaries"
      @select-category="selectOnlineCategory"
      @preview-dictionary="previewOnlineDictionary"
      @show-url-import="showUrlImportDialog = true"
      @install-dicts="installLmdgDictionaries"
      @install-grammar="installLmdgGrammar"
      @uninstall-grammar="uninstallLmdgGrammar"
    />

    <DictionaryUrlImportDialog
      v-model="showUrlImportDialog"
      v-model:import-url="importUrl"
      v-model:source-name="importUrlSourceName"
      :importing="importing"
      @preview="previewUrlDictionary"
    />

    <DictionaryImportPreviewDialog
      v-model="showImportPreviewDialog"
      :import-preview="importPreview"
      :importing="importing"
      @confirm="confirmDictionaryImport"
    />
  </div>
</template>

<style scoped>
.dictionaries-hub-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* Bento Metrics Grid */
.dict-bento-metrics {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 12px;
}

.metric-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-xs);
  transition: all var(--transition-fast);
}

.metric-card:hover {
  transform: translateY(-1px);
  border-color: var(--brand-300);
  box-shadow: var(--shadow-sm);
}

.metric-card.card-accent {
  background: linear-gradient(135deg, var(--brand-50, #eff6ff) 0%, var(--color-surface) 100%);
  border-color: var(--brand-200);
}

html[data-theme="dark"] .metric-card.card-accent {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.15) 0%, var(--color-surface) 100%);
  border-color: rgba(59, 130, 246, 0.3);
}

.metric-icon-box {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-sm);
  background: var(--color-surface-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--brand-600);
  font-size: 18px;
  flex-shrink: 0;
}

.card-accent .metric-icon-box {
  background: var(--brand-600);
  color: #fff;
}

.metric-body {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.metric-label {
  font-size: 11px;
  font-weight: 700;
  color: var(--ink-500);
}

.metric-value {
  font-size: 16px;
  font-weight: 800;
  color: var(--ink-900);
}

.metric-value.truncate {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.metric-total {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-muted);
}

/* Action Bar */
.dict-action-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  gap: 12px;
  flex-wrap: wrap;
}

.action-bar-left,
.action-bar-right {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.deploy-btn {
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.25);
}

/* Main Grid */
.dict-workbench-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 280px;
  gap: 16px;
  align-items: start;
}

.dict-main-column {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.dict-panel {
  padding: 0;
  overflow: hidden;
}

.dict-panel :deep(.el-card__header) {
  padding: 12px 16px;
  background: var(--color-surface-soft);
  border-bottom: 1px solid var(--color-line-soft);
}

.dict-panel :deep(.el-card__body) {
  padding: 0;
}

.dict-panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.panel-heading-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.panel-icon-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--brand-500);
  box-shadow: 0 0 6px rgba(37, 99, 235, 0.4);
}

.panel-icon-dot.gray {
  background: var(--ink-400);
  box-shadow: none;
}

.panel-heading-title {
  margin: 0;
  font-size: 13px;
  font-weight: 750;
  color: var(--ink-900);
}

.header-tags {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tag-pill-accent {
  font-size: 11px;
  color: var(--brand-600);
  background: var(--brand-50, #eff6ff);
  padding: 2px 8px;
  border-radius: var(--radius-full);
  font-family: var(--font-mono);
  border: 1px solid var(--brand-200);
}

.count-capsule {
  font-size: 11px;
  font-weight: 700;
  color: var(--ink-600);
  background: var(--color-surface);
  padding: 2px 8px;
  border-radius: var(--radius-full);
  border: 1px solid var(--color-line-soft);
}

/* Table Enhancements */
.dict-clean-table {
  font-size: 12px;
}

.dict-ref-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.priority-badge {
  font-size: 10px;
  font-weight: 800;
  padding: 1px 6px;
  border-radius: 4px;
  background: var(--color-surface-soft);
  color: var(--ink-500);
  border: 1px solid var(--color-line-soft);
}

.priority-badge.is-top {
  background: var(--brand-600);
  color: #fff;
  border-color: var(--brand-600);
}

.ref-name-wrap {
  display: flex;
  flex-direction: column;
}

.dict-ref-name {
  font-weight: 700;
  color: var(--ink-900);
}

.top-hint {
  font-size: 10px;
  color: var(--brand-600);
}

.status-tag {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: var(--radius-full);
}

.tag-enabled {
  background: rgba(16, 185, 129, 0.12);
  color: #059669;
}

.tag-missing {
  background: rgba(239, 68, 68, 0.12);
  color: #dc2626;
}

.dict-file-cell {
  display: flex;
  align-items: center;
  gap: 10px;
}

.dict-file-icon {
  color: var(--brand-500);
  font-size: 16px;
}

.dict-file-meta {
  display: flex;
  flex-direction: column;
}

.file-name {
  font-weight: 700;
  color: var(--ink-900);
}

.file-date {
  font-size: 10px;
  color: var(--color-muted);
}

.entry-num,
.size-num {
  font-family: var(--font-mono);
  color: var(--ink-700);
}

.row-action-btns {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
}

.dict-pagination {
  padding: 8px 16px;
  display: flex;
  justify-content: flex-end;
  background: var(--color-surface-soft);
  border-top: 1px solid var(--color-line-soft);
}

/* Health Inspection Banner */
.health-inspect-banner {
  padding: 14px 16px;
  background: var(--amber-50, #fffbeb);
  border-top: 1px solid #fef3c7;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

html[data-theme="dark"] .health-inspect-banner {
  background: rgba(245, 158, 11, 0.1);
  border-color: rgba(245, 158, 11, 0.25);
}

.health-banner-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.health-title-group {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 750;
  color: var(--ink-900);
}

.health-metrics-row {
  display: flex;
  gap: 12px;
}

.health-pill {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 10px;
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-xs);
  font-size: 11px;
}

.health-pill.warn {
  border-color: #f59e0b;
  color: #b45309;
}

.health-pill-val {
  font-weight: 800;
  color: var(--ink-900);
}

/* Side Column */
.dict-side-column {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.side-card {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.side-card-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 750;
  color: var(--ink-900);
}

.side-card-text {
  margin: 0;
  font-size: 11px;
  color: var(--ink-600);
  line-height: 1.5;
}

.format-badges-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.fmt-badge {
  font-size: 10px;
  font-family: var(--font-mono);
  background: var(--color-surface-soft);
  padding: 2px 6px;
  border-radius: 4px;
  border: 1px solid var(--color-line-soft);
  color: var(--ink-700);
}

.side-card-tip {
  font-size: 11px;
  color: var(--color-muted);
  line-height: 1.4;
  padding-top: 6px;
  border-top: 1px solid var(--color-line-soft);
}

.sogou-health-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.sogou-health-item {
  display: flex;
  justify-content: space-between;
  padding: 6px 10px;
  background: var(--color-surface-soft);
  border-radius: var(--radius-xs);
  font-size: 11px;
}

.warn-text {
  color: #ef4444;
}
</style>
