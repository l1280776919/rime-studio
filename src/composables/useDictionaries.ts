import { computed, onMounted, onUnmounted, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  DictHealth,
  DictionaryCleanResult,
  DictionaryConfig,
  DictInfo,
  DictionaryExportResult,
  DictionaryImportPreview,
  DictionaryImportResult,
  DictionaryReference,
  LmdgDownloadProgress,
  LmdgGrammarInstallResult,
  LmdgGrammarUninstallResult,
  LmdgInstallResult,
  OnlineDictionary,
  OnlineDictionaryCategory,
} from "../types";

type EmitFn = (event: "openPath", command: "open_rime_user_dir") => void;

export function useDictionaries(emit: EmitFn) {
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
      const [dicts, categories] = await Promise.all([
        invoke<OnlineDictionary[]>("list_online_dictionaries"),
        invoke<OnlineDictionaryCategory[]>("list_online_dictionary_categories"),
      ]);
      onlineDictionaries.value = dicts;
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

  return {
    // Refs
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
    fileInput,
    importPreview,
    importSourceName,
    importData,
    importKind,
    importOnlineId,
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

    // Functions
    loadDictionaries,
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

    // Computed
    totalEntries,
    totalSize,
    enabledCount,
  };
}
