<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { formatBytes, formatTime } from "../utils";
import DictionaryImportPreviewDialog from "../components/dictionaries/DictionaryImportPreviewDialog.vue";
import DictionaryUrlImportDialog from "../components/dictionaries/DictionaryUrlImportDialog.vue";
import OnlineDictionaryDialog from "../components/dictionaries/OnlineDictionaryDialog.vue";
import {
  Collection,
  Delete,
  Download,
  FolderOpened,
  InfoFilled,
  Open,
  Refresh,
  UploadFilled,
  Warning,
} from "@element-plus/icons-vue";
import type {
  DictHealth,
  DictionaryCleanResult,
  DictionaryConfig,
  DictInfo,
  DictionaryExportResult,
  DictionaryImportPreview,
  DictionaryImportResult,
  DictionaryReference,
  LmdgGrammarInstallResult,
  LmdgGrammarUninstallResult,
  LmdgInstallResult,
  OnlineDictionary,
  OnlineDictionaryCategory,
  RimeEnvironment,
} from "../types";

const props = defineProps<{
  env?: RimeEnvironment;
}>();

const emit = defineEmits<{
  openPath: [command: "open_rime_user_dir"];
  deploy: [];
}>();

const dictionaries = ref<DictInfo[]>([]);
const dictConfig = ref<DictionaryConfig>();
const loading = ref(false);
const importing = ref(false);
const exportingDict = ref<string>();
const expandedDict = ref<string | null>(null);
const dictHealth = ref<DictHealth | null>(null);
const healthLoading = ref(false);
const deletingDict = ref<string>();
const updatingReference = ref<string>();
const cleaningDict = ref<string>();
const fileInput = ref<HTMLInputElement>();
const importPreview = ref<DictionaryImportPreview>();
const importSourceName = ref("");
const importData = ref<number[]>([]);
const importKind = ref<"file" | "online" | "url">("file");
const importOnlineId = ref("");
const importUrl = ref("");
const importUrlSourceName = ref("");
const showImportPreviewDialog = ref(false);
const showUrlImportDialog = ref(false);
const showOnlineDictionaryDialog = ref(false);
const onlineDictionaries = ref<OnlineDictionary[]>([]);
const onlineCategories = ref<OnlineDictionaryCategory[]>([]);
const categoryDictionaries = ref<OnlineDictionary[]>([]);
const selectedOnlineCategory = ref("96");
const onlineLoading = ref(false);
const categoryLoading = ref(false);
const onlineImporting = ref<string>();
const lmdgInstalling = ref(false);
const lmdgResult = ref<LmdgInstallResult>();
const lmdgGrammarInstalling = ref(false);
const lmdgGrammarUninstalling = ref(false);
const lmdgGrammarResult = ref<LmdgGrammarInstallResult>();
const lmdgGrammarUninstallResult = ref<LmdgGrammarUninstallResult>();
const lmdgDownloadProgress = ref<LmdgDownloadProgress>();
let unlistenLmdgProgress: UnlistenFn | undefined;

type LmdgDownloadProgress = {
  kind: "grammar" | "dicts";
  stage: string;
  downloaded_bytes: number;
  total_bytes?: number;
  percent?: number;
};

async function loadDictionaries() {
  loading.value = true;
  try {
    const [dictList, config] = await Promise.all([
      invoke<DictInfo[]>("list_dictionaries"),
      invoke<DictionaryConfig>("get_dictionary_config"),
    ]);
    dictionaries.value = dictList;
    dictConfig.value = config;
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    loading.value = false;
  }
}

async function toggleHealth(dict: DictInfo) {
  if (expandedDict.value === dict.name) {
    expandedDict.value = null;
    dictHealth.value = null;
    return;
  }

  expandedDict.value = dict.name;
  healthLoading.value = true;
  dictHealth.value = null;
  try {
    dictHealth.value = await invoke<DictHealth>("get_dict_health", {
      dictName: dict.name,
    });
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    healthLoading.value = false;
  }
}

function openFileLocation(dict: DictInfo) {
  emit("openPath", "open_rime_user_dir");
  ElMessage.info(`词库文件: ${dict.name}`);
}

function referenceToDictInfo(reference: DictionaryReference): DictInfo {
  return {
    name: `${reference.reference}.dict.yaml`,
    path: reference.path ?? "",
    entry_count: reference.entry_count ?? 0,
    size_bytes: reference.size_bytes ?? 0,
  };
}

function dictNameToReference(name: string) {
  return name.replace(/\.dict\.yaml$/, "");
}

function chooseImportFile() {
  fileInput.value?.click();
}

async function importDictionary(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = "";
  if (!file) return;

  importing.value = true;
  try {
    const buffer = await file.arrayBuffer();
    importKind.value = "file";
    importSourceName.value = file.name;
    importData.value = Array.from(new Uint8Array(buffer));
    importOnlineId.value = "";
    importUrl.value = "";
    importUrlSourceName.value = "";
    importPreview.value = await invoke<DictionaryImportPreview>("preview_dictionary_import", {
      sourceName: file.name,
      data: importData.value,
    });
    showImportPreviewDialog.value = true;
  } catch (error) {
    importSourceName.value = "";
    importData.value = [];
    importPreview.value = undefined;
    ElMessage.error(String(error));
  } finally {
    importing.value = false;
  }
}

async function loadOnlineDictionaries() {
  onlineLoading.value = true;
  try {
    const [dictionaries, categories] = await Promise.all([
      invoke<OnlineDictionary[]>("list_online_dictionaries"),
      invoke<OnlineDictionaryCategory[]>("list_online_dictionary_categories"),
    ]);
    onlineDictionaries.value = dictionaries;
    onlineCategories.value = categories;
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    onlineLoading.value = false;
  }
}

async function loadCategoryDictionaries() {
  if (!selectedOnlineCategory.value) return;
  categoryLoading.value = true;
  try {
    categoryDictionaries.value = await invoke<OnlineDictionary[]>("list_online_dictionaries_by_category", {
      categoryId: selectedOnlineCategory.value,
    });
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    categoryLoading.value = false;
  }
}

async function installLmdgDictionaries() {
  lmdgInstalling.value = true;
  lmdgDownloadProgress.value = {
    kind: "dicts",
    stage: "准备下载万象词库包",
    downloaded_bytes: 0,
  };
  try {
    const result = await invoke<LmdgInstallResult>("install_lmdg_dicts");
    lmdgResult.value = result;
    await loadAllStats();
    ElMessage.success(result.message);
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    lmdgInstalling.value = false;
  }
}

async function installLmdgGrammar() {
  lmdgGrammarInstalling.value = true;
  lmdgGrammarUninstallResult.value = undefined;
  lmdgDownloadProgress.value = {
    kind: "grammar",
    stage: "准备下载万象语言模型",
    downloaded_bytes: 0,
  };
  try {
    const result = await invoke<LmdgGrammarInstallResult>("install_lmdg_grammar");
    lmdgGrammarResult.value = result;
    ElMessage.success(result.message);
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    lmdgGrammarInstalling.value = false;
  }
}

async function uninstallLmdgGrammar() {
  lmdgGrammarUninstalling.value = true;
  try {
    const result = await invoke<LmdgGrammarUninstallResult>("uninstall_lmdg_grammar");
    lmdgGrammarUninstallResult.value = result;
    lmdgGrammarResult.value = undefined;
    ElMessage.success(result.message);
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    lmdgGrammarUninstalling.value = false;
  }
}

async function previewOnlineDictionary(dict: OnlineDictionary) {
  onlineImporting.value = dict.id;
  try {
    importKind.value = "url";
    importOnlineId.value = "";
    importSourceName.value = dict.source_name;
    importData.value = [];
    importUrl.value = dict.detail_url;
    importUrlSourceName.value = dict.source_name;
    importPreview.value = await invoke<DictionaryImportPreview>("preview_dictionary_url_import", {
      url: dict.detail_url,
      sourceName: dict.source_name,
    });
    showImportPreviewDialog.value = true;
  } catch (error) {
    importPreview.value = undefined;
    ElMessage.error(String(error));
  } finally {
    onlineImporting.value = undefined;
  }
}

async function previewUrlDictionary() {
  const url = importUrl.value.trim();
  if (!url) {
    ElMessage.warning("请先填写在线词库地址");
    return;
  }

  importing.value = true;
  try {
    importKind.value = "url";
    importSourceName.value = "";
    importOnlineId.value = "";
    importData.value = [];
    importPreview.value = await invoke<DictionaryImportPreview>("preview_dictionary_url_import", {
      url,
      sourceName: importUrlSourceName.value.trim() || undefined,
    });
    showUrlImportDialog.value = false;
    showImportPreviewDialog.value = true;
  } catch (error) {
    importPreview.value = undefined;
    ElMessage.error(String(error));
  } finally {
    importing.value = false;
  }
}

async function confirmDictionaryImport(enableAfterImport = false) {
  if (!importPreview.value) return;

  importing.value = true;
  try {
    let result: DictionaryImportResult;
    if (importKind.value === "online") {
      result = await invoke<DictionaryImportResult>("import_online_dictionary", {
        id: importOnlineId.value,
      });
    } else if (importKind.value === "url") {
      result = await invoke<DictionaryImportResult>("import_dictionary_url", {
        url: importUrl.value.trim(),
        sourceName: importUrlSourceName.value.trim() || undefined,
      });
    } else {
      if (!importSourceName.value || !importData.value.length) return;
      result = await invoke<DictionaryImportResult>("import_dictionary", {
        sourceName: importSourceName.value,
        data: importData.value,
      });
    }
    await loadAllStats();
    ElMessage.success(
      `已导入 ${result.imported_entries.toLocaleString()} 条到 ${result.name}`
      + (result.skipped_entries ? `，跳过 ${result.skipped_entries.toLocaleString()} 条` : ""),
    );
    if (enableAfterImport) {
      await addDictionaryReference(result.reference);
    }
    showImportPreviewDialog.value = false;
    importSourceName.value = "";
    importData.value = [];
    importOnlineId.value = "";
    importUrl.value = "";
    importUrlSourceName.value = "";
    importPreview.value = undefined;
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    importing.value = false;
  }
}

async function exportDictionary(dict: DictInfo) {
  exportingDict.value = dict.name;
  try {
    const result = await invoke<DictionaryExportResult>("export_dictionary", {
      dictName: dict.name,
    });
    const blob = new Blob([result.contents], { type: "text/yaml;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = result.name;
    link.click();
    URL.revokeObjectURL(url);
    ElMessage.success("词库已导出");
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    exportingDict.value = undefined;
  }
}

async function addDictionaryReference(reference: string) {
  updatingReference.value = reference;
  try {
    dictConfig.value = await invoke<DictionaryConfig>("add_dictionary_to_current_schema", { reference });
    await loadAllStats();
    ElMessage.success("已加入当前方案，重新部署后生效");
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    updatingReference.value = undefined;
  }
}

async function removeDictionaryReference(reference: string) {
  updatingReference.value = reference;
  try {
    dictConfig.value = await invoke<DictionaryConfig>("remove_dictionary_from_current_schema", { reference });
    await loadAllStats();
    ElMessage.success("已从当前方案移除引用");
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    updatingReference.value = undefined;
  }
}

async function moveReference(reference: string, direction: -1 | 1) {
  if (!dictConfig.value) return;
  const imports = [
    ...dictConfig.value.enabled.map((entry) => entry.reference),
    ...dictConfig.value.missing.map((entry) => entry.reference),
  ];
  const index = imports.indexOf(reference);
  const nextIndex = index + direction;
  if (index < 0 || nextIndex < 0 || nextIndex >= imports.length) return;
  [imports[index], imports[nextIndex]] = [imports[nextIndex], imports[index]];

  updatingReference.value = reference;
  try {
    dictConfig.value = await invoke<DictionaryConfig>("save_dictionary_imports", { imports });
    await loadAllStats();
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    updatingReference.value = undefined;
  }
}

async function deleteDictionary(dict: DictInfo) {
  try {
    await ElMessageBox.confirm(
      `确定删除「${dict.name}」？此操作不可恢复。`,
      "删除词库",
      { confirmButtonText: "删除", cancelButtonText: "取消", type: "warning" },
    );
  } catch {
    return;
  }
  deletingDict.value = dict.name;
  try {
    await invoke("delete_dictionary", { dictName: dict.name });
    ElMessage.success("词库已删除");
    await loadAllStats();
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    deletingDict.value = undefined;
  }
}

async function cleanDuplicateLines(dictName: string) {
  if (!dictHealth.value?.duplicate_exact_lines) {
    ElMessage.info("这个词库没有重复词条");
    return;
  }

  try {
    await ElMessageBox.confirm(
      `将从「${dictName}」中移除 ${dictHealth.value.duplicate_exact_lines.toLocaleString()} 条完全重复的词条行。执行前会自动创建保存前备份。`,
      "清理重复词条",
      { confirmButtonText: "清理", cancelButtonText: "取消", type: "warning" },
    );
  } catch {
    return;
  }

  cleaningDict.value = dictName;
  try {
    const result = await invoke<DictionaryCleanResult>("clean_dictionary_duplicates", {
      dictName,
    });
    await loadAllStats();
    dictHealth.value = await invoke<DictHealth>("get_dict_health", { dictName });
    ElMessage.success(
      result.removed_duplicate_lines
        ? `已移除 ${result.removed_duplicate_lines.toLocaleString()} 条重复词条`
        : "未发现需要清理的重复词条",
    );
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    cleaningDict.value = undefined;
  }
}

const totalEntries = computed(() => dictionaries.value.reduce((s, d) => s + d.entry_count, 0));
const totalSize = computed(() => dictionaries.value.reduce((s, d) => s + d.size_bytes, 0));
const enabledCount = computed(() => (dictConfig.value?.enabled.length ?? 0) + (dictConfig.value?.missing.length ?? 0));
async function selectOnlineCategory(categoryId: string) {
  selectedOnlineCategory.value = categoryId;
  await loadCategoryDictionaries();
}

async function loadAllStats() {
  await loadDictionaries();
}

onMounted(async () => {
  unlistenLmdgProgress = await listen<LmdgDownloadProgress>("lmdg-download-progress", (event) => {
    lmdgDownloadProgress.value = event.payload;
  });
  await Promise.all([loadAllStats(), loadOnlineDictionaries()]);
  await loadCategoryDictionaries();
});

onUnmounted(() => {
  unlistenLmdgProgress?.();
});
</script>

<template>
  <section class="content-grid dictionaries-grid">
    <section class="main-column">
      <!-- Metric overview -->
      <div class="metric-grid">
        <div class="metric accent">
          <el-icon><Collection /></el-icon>
          <span>词库文件</span>
          <strong>{{ dictionaries.length }}</strong>
        </div>
        <div class="metric">
          <el-icon><InfoFilled /></el-icon>
          <span>总条目数</span>
          <strong>{{ totalEntries.toLocaleString() }}</strong>
        </div>
        <div class="metric">
          <el-icon><FolderOpened /></el-icon>
          <span>总大小</span>
          <strong>{{ formatBytes(totalSize) }}</strong>
        </div>
        <div class="metric">
          <el-icon><Warning /></el-icon>
          <span>当前方案</span>
          <strong>{{ dictConfig?.schema_name ?? dictConfig?.schema_id ?? "未识别" }}</strong>
        </div>
        <div class="metric">
          <el-icon><Collection /></el-icon>
          <span>启用词库</span>
          <strong>{{ enabledCount }}</strong>
        </div>
      </div>

      <!-- Toolbar -->
      <div class="dictionaries-toolbar">
        <input
          ref="fileInput"
          type="file"
          accept=".bin,.scel,.txt,.dict.yaml,.yaml"
          style="display: none"
          @change="importDictionary"
        />
        <el-button type="primary" :icon="UploadFilled" :loading="importing" @click="chooseImportFile">
          导入词库
        </el-button>
        <el-button type="success" :icon="Download" @click="showOnlineDictionaryDialog = true">
          在线词库
        </el-button>
        <el-button :icon="Download" :loading="importing" @click="showUrlImportDialog = true">
          URL 导入
        </el-button>
        <el-button :icon="Refresh" :loading="loading" @click="loadAllStats">刷新</el-button>
        <el-button :icon="UploadFilled" @click="emit('deploy')">重新部署</el-button>
        <el-button :icon="FolderOpened" @click="emit('openPath', 'open_rime_user_dir')">
          打开用户目录
        </el-button>
      </div>

      <el-card class="panel" shadow="never">
        <template #header>
          <div class="panel-title">
            <span>当前方案词库</span>
            <small>{{ dictConfig?.main_dictionary ? `${dictConfig.main_dictionary}.dict.yaml` : "未找到主词库" }}</small>
          </div>
        </template>
        <el-empty
          v-if="!loading && !dictConfig?.enabled.length && !dictConfig?.missing.length"
          description="当前方案还没有配置 import_tables"
          :image-size="80"
        />
        <el-table
          v-else
          :data="[...(dictConfig?.enabled ?? []), ...(dictConfig?.missing ?? [])]"
          v-loading="loading"
          stripe
          max-height="calc(50dvh - 180px)"
        >
          <el-table-column label="引用名" min-width="220">
            <template #default="{ row, $index }: { row: DictionaryReference; $index: number }">
              <div class="dict-name-cell">
                <el-icon><Collection /></el-icon>
                <span class="dict-name">{{ row.reference }}</span>
                <el-tag v-if="!row.exists" type="danger" size="small">缺失</el-tag>
                <el-tag v-else type="success" size="small">启用</el-tag>
              </div>
              <small v-if="$index === 0" class="helper-text">优先级最高</small>
            </template>
          </el-table-column>
          <el-table-column label="条目数" width="110" align="right">
            <template #default="{ row }: { row: DictionaryReference }">
              {{ row.entry_count?.toLocaleString() ?? "—" }}
            </template>
          </el-table-column>
          <el-table-column label="大小" width="100" align="right">
            <template #default="{ row }: { row: DictionaryReference }">
              {{ formatBytes(row.size_bytes) }}
            </template>
          </el-table-column>
          <el-table-column label="操作" width="260" align="center">
            <template #default="{ row, $index }: { row: DictionaryReference; $index: number }">
              <el-button link type="primary" :disabled="$index === 0" @click.stop="moveReference(row.reference, -1)">上移</el-button>
              <el-button link type="primary" :disabled="$index >= enabledCount - 1" @click.stop="moveReference(row.reference, 1)">下移</el-button>
              <el-button
                v-if="row.exists"
                link type="primary" :icon="Download"
                :loading="exportingDict === `${row.reference}.dict.yaml`"
                @click.stop="exportDictionary(referenceToDictInfo(row))"
              >
                导出
              </el-button>
              <el-button
                link type="danger"
                :loading="updatingReference === row.reference"
                @click.stop="removeDictionaryReference(row.reference)"
              >
                移除
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </el-card>

      <el-card class="panel" shadow="never">
        <template #header>
          <div class="panel-title">
            <span>未启用词库</span>
            <small>加入当前方案后，重新部署才会生效</small>
          </div>
        </template>
        <el-empty
          v-if="!loading && !dictConfig?.available.length"
          description="没有未启用词库"
          :image-size="80"
        >
          <p class="helper-text">
            还没有 .dict.yaml 词库文件。你可以安装 rime-ice 获取预置词库，或手动放置词库文件到用户目录。
          </p>
        </el-empty>

        <template v-else>
          <el-table
            :data="dictConfig?.available ?? []"
            v-loading="loading"
            stripe
            highlight-current-row
            max-height="calc(50dvh - 180px)"
            @row-click="toggleHealth"
          >
            <el-table-column label="文件名" min-width="240">
              <template #default="{ row }: { row: DictInfo }">
                <div class="dict-name-cell">
                  <el-icon><Collection /></el-icon>
                  <span class="dict-name">{{ row.name }}</span>
                </div>
              </template>
            </el-table-column>
            <el-table-column label="条目数" width="110" align="right">
              <template #default="{ row }: { row: DictInfo }">
                {{ row.entry_count.toLocaleString() }}
              </template>
            </el-table-column>
            <el-table-column label="大小" width="100" align="right">
              <template #default="{ row }: { row: DictInfo }">
                {{ formatBytes(row.size_bytes) }}
              </template>
            </el-table-column>
            <el-table-column label="修改时间" width="170">
              <template #default="{ row }: { row: DictInfo }">
                {{ formatTime(row.modified) }}
              </template>
            </el-table-column>
            <el-table-column label="操作" width="240" align="center" class-name="dict-actions-column">
              <template #default="{ row }: { row: DictInfo }">
                <div class="dict-row-actions">
                  <el-button
                    link type="success"
                    :loading="updatingReference === row.name"
                    @click.stop="addDictionaryReference(dictNameToReference(row.name))"
                  >
                    加入
                  </el-button>
                  <el-button link type="primary" :icon="Open" @click.stop="openFileLocation(row)">
                    定位
                  </el-button>
                  <el-button
                    link type="primary" :icon="Download"
                    :loading="exportingDict === row.name"
                    @click.stop="exportDictionary(row)"
                  >
                    导出
                  </el-button>
                  <el-button
                    link type="danger" :icon="Delete"
                    :loading="deletingDict === row.name"
                    @click.stop="deleteDictionary(row)"
                  >
                    删除
                  </el-button>
                </div>
              </template>
            </el-table-column>
          </el-table>

          <!-- Expandable health section -->
          <Transition name="el-fade-in-linear">
            <div v-if="expandedDict && dictHealth" class="dict-health-detail">
              <el-divider />
              <div class="panel-title" style="margin-bottom: 10px">
                <span>{{ expandedDict }} — 健康分析</span>
                <el-button
                  type="warning"
                  plain
                  size="small"
                  :icon="Delete"
                  :loading="cleaningDict === expandedDict"
                  :disabled="!dictHealth.duplicate_exact_lines"
                  @click.stop="cleanDuplicateLines(expandedDict)"
                >
                  一键去重
                </el-button>
              </div>
              <div class="health-list health-list-row">
                <div>
                  <span>总条目</span>
                  <strong>{{ dictHealth.entries.toLocaleString() }}</strong>
                </div>
                <div>
                  <span>重复行</span>
                  <strong :class="dictHealth.duplicate_exact_lines ? 'warn-text' : ''">
                    {{ dictHealth.duplicate_exact_lines.toLocaleString() }}
                  </strong>
                </div>
                <div>
                  <span>长低权重项</span>
                  <strong :class="dictHealth.long_low_weight_entries ? 'warn-text' : ''">
                    {{ dictHealth.long_low_weight_entries.toLocaleString() }}
                  </strong>
                </div>
              </div>
            </div>
            <div v-else-if="expandedDict && healthLoading" class="dict-health-detail">
              <el-divider />
              <el-skeleton :rows="2" animated />
            </div>
          </Transition>
        </template>
      </el-card>
    </section>

    <aside class="side-column">
      <!-- Quick help -->
      <el-card class="panel" shadow="never">
        <template #header>
          <span>格式说明</span>
        </template>
        <p class="helper-text">
          Rime 词库文件以 <code>.dict.yaml</code> 结尾。包含 YAML 头部和 Tab 分隔的数据行（词汇→编码→权重）。
          支持导入搜狗用户备份 <code>.bin</code>、搜狗细胞词库 <code>.scel</code>、Tab 分隔 <code>.txt</code> 和 Rime <code>.dict.yaml</code>。
          导入后点击重新部署生效。
        </p>
      </el-card>

      <!-- Sogou health from env -->
      <el-card v-if="env?.sogou_health" class="panel" shadow="never">
        <template #header>
          <span>搜狗词库健康</span>
        </template>
        <div class="health-list">
          <div>
            <span>条目</span>
            <strong>{{ env.sogou_health.entries.toLocaleString() }}</strong>
          </div>
          <div>
            <span>重复行</span>
            <strong :class="env.sogou_health.duplicate_exact_lines ? 'warn-text' : ''">
              {{ env.sogou_health.duplicate_exact_lines.toLocaleString() }}
            </strong>
          </div>
          <div>
            <span>长低权重项</span>
            <strong :class="env.sogou_health.long_low_weight_entries ? 'warn-text' : ''">
              {{ env.sogou_health.long_low_weight_entries.toLocaleString() }}
            </strong>
          </div>
        </div>
      </el-card>
    </aside>

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
  </section>
</template>
