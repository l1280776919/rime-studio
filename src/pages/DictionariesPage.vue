<script setup lang="ts">
import { computed, ref } from "vue";
import { formatBytes, formatTime } from "../utils";
import { paginateItems } from "../utils/phrases";
import { useConfigReload } from "../composables/useConfigReload";
import { useDictionaries } from "../composables/useDictionaries";
import DictionaryImportPreviewDialog from "../components/dictionaries/DictionaryImportPreviewDialog.vue";
import DictionaryUrlImportDialog from "../components/dictionaries/DictionaryUrlImportDialog.vue";
import OnlineDictionaryDialog from "../components/dictionaries/OnlineDictionaryDialog.vue";
import {
  Collection,
  Delete,
  Download,
  FolderOpened,
  Link,
  MagicStick,
  Open,
  Refresh,
  UploadFilled,
} from "@element-plus/icons-vue";
import type { DictionaryReference, DictInfo, RimeEnvironment } from "../types";

const props = defineProps<{
  env?: RimeEnvironment;
}>();

const emit = defineEmits<{
  openPath: [command: "open_rime_user_dir"];
  deploy: [];
}>();

const {
  dictionaries,
  fileInput,
  dictConfig,
  orderedReferences,
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
  grammarInstalled,
  grammarScanning,
  grammarScanError,
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
  reorderReference,
  deleteDictionary,
  cleanDuplicateLines,
  selectOnlineCategory,
  loadAllStats,
  totalEntries,
  totalSize,
  enabledCount,
} = useDictionaries(emit);

useConfigReload(() => props.env, loadAllStats);

const availablePage = ref(1);
const availablePageSize = ref(40);
const pagedAvailable = computed(() =>
  paginateItems(dictConfig.value?.available ?? [], availablePage.value, availablePageSize.value),
);

const draggedReference = ref<string>();
const dropTarget = ref<string>();
const dictionaryByReference = computed(
  () => new Map(dictionaries.value.map((dict) => [dictNameToReference(dict.name), dict])),
);
function referenceInfo(reference: string) {
  return dictionaryByReference.value.get(reference);
}
function clearDrag() {
  draggedReference.value = undefined;
  dropTarget.value = undefined;
}
function startDrag(event: DragEvent, reference: string) {
  if (updatingReference.value || loading.value) {
    event.preventDefault();
    return;
  }
  draggedReference.value = reference;
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData("text/plain", reference);
  }
}
/** Resolve the row from any cell so the full row is a drop target, not only the handle. */
function targetReference(event: DragEvent) {
  const target = event.target as HTMLElement | null;
  return target?.closest("tr")?.querySelector<HTMLElement>("[data-reference]")?.dataset.reference;
}
function onDragOver(event: DragEvent) {
  if (!draggedReference.value || updatingReference.value || loading.value) return;
  const target = targetReference(event);
  if (!target) return;
  event.preventDefault();
  dropTarget.value = target;
  if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
}
async function onDrop(event: DragEvent) {
  event.preventDefault();
  const source = draggedReference.value;
  const target = targetReference(event);
  clearDrag();
  if (source && target)
    await reorderReference(source, dictConfig.value?.imports.indexOf(target) ?? -1);
}
</script>

<template>
  <div class="dictionaries-hub-container">
    <header class="dictionary-heading">
      <div>
        <h2>管理你的词库</h2>
        <p>
          {{ dictConfig?.schema_name ?? dictConfig?.schema_id ?? "当前方案" }} ·
          {{ enabledCount }} 个启用 / {{ dictionaries.length }} 个本地词库 ·
          {{ totalEntries.toLocaleString() }} 条词条 · {{ formatBytes(totalSize) }}
        </p>
      </div>
    </header>

    <!-- Action Bar -->
    <div class="dict-action-bar panel">
      <div class="action-bar-left">
        <input
          :ref="(element) => (fileInput = element as HTMLInputElement | undefined)"
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

        <el-button type="success" plain :icon="Download" @click="showOnlineDictionaryDialog = true">
          在线词库
        </el-button>

        <el-button :icon="Link" :loading="importing" @click="showUrlImportDialog = true">
          URL 在线导入
        </el-button>
      </div>

      <div class="action-bar-right">
        <el-button :icon="Refresh" :loading="loading" @click="loadAllStats"> 刷新统计 </el-button>
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
                <h3 class="panel-heading-title">已启用词库</h3>
              </div>
              <div class="header-tags">
                <span class="tag-pill-accent">
                  主词库：{{
                    dictConfig?.main_dictionary
                      ? `${dictConfig.main_dictionary}.dict.yaml`
                      : "未识别"
                  }}
                </span>
                <span class="count-capsule">{{ enabledCount }} 项启用</span>
              </div>
            </div>
          </template>

          <el-empty
            v-if="!loading && !dictConfig?.enabled.length && !dictConfig?.missing.length"
            description="尚未启用扩展词库，可从下方加入词库"
            :image-size="64"
          />

          <p v-if="orderedReferences.length" class="order-help">
            拖动左侧手柄调整加载顺序，修改后请部署生效。
          </p>
          <el-table
            v-if="orderedReferences.length || loading"
            v-loading="loading"
            :data="orderedReferences"
            row-key="reference"
            :row-class-name="
              ({ row }: { row: DictionaryReference }) =>
                dropTarget === row.reference ? 'drop-target' : ''
            "
            stripe
            class="dict-clean-table"
            max-height="360"
            @dragover="onDragOver"
            @drop="onDrop"
            @dragend="clearDrag"
          >
            <el-table-column label="词库名称" min-width="240">
              <template #default="{ row, $index }: { row: DictionaryReference; $index: number }">
                <div class="dict-ref-cell" :data-reference="row.reference">
                  <button
                    class="drag-handle"
                    :draggable="!updatingReference && !loading"
                    :disabled="!!updatingReference || loading"
                    :aria-label="`拖动排序 ${row.reference}`"
                    title="拖动调整顺序"
                    @dragstart="startDrag($event, row.reference)"
                  >
                    ⠿
                  </button>
                  <span class="priority-badge" :class="{ 'is-top': $index === 0 }">
                    #{{ $index + 1 }}
                  </span>
                  <div class="ref-name-wrap">
                    <span class="dict-ref-name">{{
                      referenceInfo(row.reference)?.display_name || row.reference
                    }}</span>
                    <small class="file-date">{{ row.reference }}.dict.yaml</small>
                    <small v-if="$index === 0" class="top-hint">首位加载</small>
                  </div>
                  <span v-if="!row.exists" class="status-tag tag-missing">缺失文件</span>
                  <span v-else class="status-tag tag-enabled">已启用</span>
                </div>
              </template>
            </el-table-column>

            <el-table-column label="来源" min-width="180" show-overflow-tooltip>
              <template #default="{ row }: { row: DictionaryReference }">
                <span class="source-label">{{
                  referenceInfo(row.reference)?.source || "未记录"
                }}</span>
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

            <el-table-column label="管理" width="130" align="center">
              <template #default="{ row }: { row: DictionaryReference }">
                <div class="row-action-btns">
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
                    :disabled="!!updatingReference"
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
                <h3 class="panel-heading-title">未启用词库</h3>
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
              您可以点击上方「导入词库文件」或从「在线词库」获取词库。
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
              <el-table-column label="词库名称" min-width="260">
                <template #default="{ row }: { row: DictInfo }">
                  <div class="dict-file-cell">
                    <div class="dict-file-icon">
                      <el-icon><Collection /></el-icon>
                    </div>
                    <div class="dict-file-meta">
                      <span class="file-name">{{ row.display_name || row.name }}</span>
                      <small class="file-date">{{ row.name }}</small>
                      <small class="file-date">{{ formatTime(row.modified) }}</small>
                    </div>
                  </div>
                </template>
              </el-table-column>

              <el-table-column label="来源" min-width="180" show-overflow-tooltip>
                <template #default="{ row }: { row: DictInfo }">
                  <span class="source-label">{{ row.source || "未记录" }}</span>
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

              <el-table-column label="操作" width="180" align="center" fixed="right">
                <template #default="{ row }: { row: DictInfo }">
                  <!-- 常用操作直接展示，次要操作集中到菜单，避免窄表格中换行拥挤。 -->
                  <div class="available-row-actions" @click.stop>
                    <el-button
                      size="small"
                      type="primary"
                      plain
                      :loading="updatingReference === dictNameToReference(row.name)"
                      :disabled="!!updatingReference || !!deletingDict"
                      @click="addDictionaryReference(dictNameToReference(row.name))"
                      >加入方案</el-button
                    >
                    <el-dropdown trigger="click">
                      <el-button
                        link
                        size="small"
                        :aria-label="`${row.display_name || row.name} 的更多操作`"
                        :loading="exportingDict === row.name || deletingDict === row.name"
                        >更多</el-button
                      >
                      <template #dropdown>
                        <el-dropdown-menu>
                          <el-dropdown-item :icon="Open" @click="openFileLocation(row)"
                            >定位文件</el-dropdown-item
                          >
                          <el-dropdown-item
                            :icon="Download"
                            :disabled="!!exportingDict || !!deletingDict"
                            @click="exportDictionary(row)"
                            >导出词库</el-dropdown-item
                          >
                          <el-dropdown-item
                            divided
                            :icon="Delete"
                            :disabled="!!deletingDict || !!updatingReference || !!exportingDict"
                            @click="deleteDictionary(row)"
                            >从磁盘删除</el-dropdown-item
                          >
                        </el-dropdown-menu>
                      </template>
                    </el-dropdown>
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
                    <strong class="health-pill-val">{{
                      dictHealth.entries.toLocaleString()
                    }}</strong>
                  </div>

                  <div class="health-pill" :class="{ warn: dictHealth.duplicate_exact_lines > 0 }">
                    <span class="health-pill-label">完全重复行</span>
                    <strong class="health-pill-val">{{
                      dictHealth.duplicate_exact_lines.toLocaleString()
                    }}</strong>
                  </div>

                  <div
                    class="health-pill"
                    :class="{ warn: dictHealth.long_low_weight_entries > 0 }"
                  >
                    <span class="health-pill-label">长低权生僻项</span>
                    <strong class="health-pill-val">{{
                      dictHealth.long_low_weight_entries.toLocaleString()
                    }}</strong>
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
            Rime 词库文件名必须以 <code>.dict.yaml</code> 结尾，文件内部包含 YAML 元数据头部与 Tab
            分隔的数据行（词汇 → 编码 → 权重）。
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
      :grammar-installed="grammarInstalled"
      :grammar-scanning="grammarScanning"
      :grammar-scan-error="grammarScanError"
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
/* Keep the list primary; summary and helper content stay compact. */
.dictionary-heading h2 {
  margin: 0 0 8px;
  font-size: 23px;
  color: var(--ink-900);
}
.dictionary-heading p,
.order-help {
  font-size: 12px;
  color: var(--color-muted);
  margin: 0;
  line-height: 1.6;
}
.order-help {
  padding: 10px 16px;
}
.dictionaries-hub-container {
  max-width: 1400px;
  margin: 0 auto;
}
.dict-workbench-grid {
  grid-template-columns: minmax(0, 1fr);
}
.dict-side-column {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
}
.dict-action-bar {
  box-shadow: none;
  padding: 12px;
  flex-wrap: wrap;
}
.action-bar-left,
.action-bar-right {
  flex-wrap: wrap;
  gap: 8px;
}
.action-bar-left .el-button,
.action-bar-right .el-button {
  margin: 0;
}
.drag-handle {
  border: 0;
  background: transparent;
  cursor: grab;
  color: var(--color-muted);
  font-size: 22px;
  padding: 4px;
}
.drag-handle:active {
  cursor: grabbing;
}
.drag-handle:focus-visible {
  outline: 2px solid var(--color-accent);
}
.drag-handle:disabled {
  cursor: default;
  opacity: 0.4;
}
:deep(.drop-target td) {
  background: var(--color-surface-soft) !important;
  box-shadow: inset 0 2px var(--color-accent);
}
.source-label {
  display: block;
  font-size: 12px;
  color: var(--color-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1.5;
}
.ref-name-wrap,
.dict-file-meta {
  min-width: 0;
}
.dict-ref-name,
.file-name {
  font-weight: 600;
  overflow-wrap: anywhere;
}
.row-action-btns {
  flex-wrap: wrap;
}
:deep(.el-empty) {
  padding: 20px 0;
}
.available-row-actions {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  white-space: nowrap;
}
.available-row-actions .el-button {
  margin: 0;
}
</style>
