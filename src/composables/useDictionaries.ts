import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { api } from "../api";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  DictHealth,
  DictionaryConfig,
  DictInfo,
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
  const importData = ref<Uint8Array>(new Uint8Array(0));
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
  type PreparedImport =
    { kind: "file"; name: string; data: number[] } | { kind: "url"; url: string; name?: string };
  let preparedImport: PreparedImport | undefined;
  let previewVersion = 0;
  let activePreviewSource: "file" | "url" | "online" | undefined;
  let confirmingImport = false;
  let healthVersion = 0;
  let categoryVersion = 0;
  let listVersion = 0;
  let disposed = false;
  let unlistenLmdgProgress: UnlistenFn | undefined;

  async function loadDictionaries() {
    const version = ++listVersion;
    loading.value = true;
    try {
      const [dictList, config] = await Promise.all([
        api.listDictionaries(),
        api.getDictionaryConfig(),
      ]);
      if (disposed || version !== listVersion) return;
      dictionaries.value = dictList;
      dictConfig.value = config;
    } catch (error) {
      ElMessage.error(String(error));
    } finally {
      if (version === listVersion) loading.value = false;
    }
  }

  async function toggleHealth(dict: DictInfo) {
    const version = ++healthVersion;
    if (expandedDict.value === dict.name) {
      expandedDict.value = null;
      dictHealth.value = null;
      healthLoading.value = false;
      return;
    }

    expandedDict.value = dict.name;
    healthLoading.value = true;
    dictHealth.value = null;
    try {
      const result = await api.getDictHealth(dict.name);
      if (!disposed && version === healthVersion) dictHealth.value = result;
    } catch (error) {
      ElMessage.error(String(error));
    } finally {
      if (version === healthVersion) healthLoading.value = false;
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

  function cancelPreview() {
    ++previewVersion;
    activePreviewSource = undefined;
    importing.value = false;
    onlineImporting.value = undefined;
    preparedImport = undefined;
    importPreview.value = undefined;
  }

  async function prepareImport(
    origin: "file" | "url" | "online",
    prepare: () => Promise<{ source: PreparedImport; preview: DictionaryImportPreview }>,
  ) {
    if (confirmingImport || disposed) return;
    const version = ++previewVersion;
    activePreviewSource = origin;
    preparedImport = undefined;
    importPreview.value = undefined;
    showImportPreviewDialog.value = false;
    importing.value = true;
    try {
      const { source, preview } = await prepare();
      if (disposed || version !== previewVersion) return;
      // Commit the preview together with the exact immutable source it describes.
      preparedImport = source;
      importPreview.value = preview;
      activePreviewSource = undefined;
      showUrlImportDialog.value = false;
      showImportPreviewDialog.value = true;
    } catch (error) {
      if (version === previewVersion) ElMessage.error(String(error));
    } finally {
      if (version === previewVersion) {
        importing.value = false;
        onlineImporting.value = undefined;
        activePreviewSource = undefined;
      }
    }
  }

  watch(
    showUrlImportDialog,
    (visible) => {
      if (!visible && activePreviewSource === "url") cancelPreview();
    },
    { flush: "sync" },
  );
  watch(
    showOnlineDictionaryDialog,
    (visible) => {
      if (!visible && activePreviewSource === "online") cancelPreview();
    },
    { flush: "sync" },
  );
  watch(
    showImportPreviewDialog,
    (visible) => {
      if (!visible && !confirmingImport && !activePreviewSource) {
        preparedImport = undefined;
        importPreview.value = undefined;
      }
    },
    { flush: "sync" },
  );

  async function importDictionary(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    input.value = "";
    if (!file) return;
    await prepareImport("file", async () => {
      const data = Array.from(new Uint8Array(await file.arrayBuffer()));
      const preview = await api.previewDictionaryImport(file.name, data);
      return { source: { kind: "file", name: file.name, data }, preview };
    });
  }

  async function loadOnlineDictionaries() {
    onlineLoading.value = true;
    try {
      const [dicts, categories] = await Promise.all([
        api.listOnlineDictionaries(),
        api.listOnlineDictionaryCategories(),
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
    const version = ++categoryVersion;
    const category = selectedOnlineCategory.value;
    categoryDictionaries.value = [];
    categoryLoading.value = !!category;
    if (!category) return;
    try {
      const result = await api.listOnlineDictionariesByCategory(category);
      if (!disposed && version === categoryVersion) categoryDictionaries.value = result;
    } catch (error) {
      if (version === categoryVersion) ElMessage.error(String(error));
    } finally {
      if (version === categoryVersion) categoryLoading.value = false;
    }
  }

  const resourceBusy = computed(
    () => lmdgInstalling.value || lmdgGrammarInstalling.value || lmdgGrammarUninstalling.value,
  );

  async function installLmdgDictionaries() {
    if (resourceBusy.value) return;
    lmdgInstalling.value = true;
    lmdgDownloadProgress.value = {
      kind: "dicts",
      stage: "准备下载万象词库包",
      downloaded_bytes: 0,
    };
    try {
      const result = await api.installLmdgDicts();
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
    if (resourceBusy.value) return;
    lmdgGrammarInstalling.value = true;
    lmdgGrammarUninstallResult.value = undefined;
    lmdgDownloadProgress.value = {
      kind: "grammar",
      stage: "准备下载万象语言模型",
      downloaded_bytes: 0,
    };
    try {
      const result = await api.installLmdgGrammar();
      lmdgGrammarResult.value = result;
      ElMessage.success(result.message);
    } catch (error) {
      ElMessage.error(String(error));
    } finally {
      lmdgGrammarInstalling.value = false;
    }
  }

  async function uninstallLmdgGrammar() {
    if (resourceBusy.value) return;
    lmdgGrammarUninstalling.value = true;
    try {
      const result = await api.uninstallLmdgGrammar();
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
    if (confirmingImport) return;
    onlineImporting.value = dict.id;
    const url = dict.detail_url;
    const name = dict.source_name;
    await prepareImport("online", async () => ({
      source: { kind: "url", url, name },
      preview: await api.previewDictionaryUrlImport(url, name),
    }));
  }

  async function previewUrlDictionary() {
    const url = importUrl.value.trim();
    const name = importUrlSourceName.value.trim() || undefined;
    if (!url) {
      ElMessage.warning("请先填写在线词库地址");
      return;
    }
    await prepareImport("url", async () => ({
      source: { kind: "url", url, name },
      preview: await api.previewDictionaryUrlImport(url, name),
    }));
  }

  async function confirmDictionaryImport(enableAfterImport = false) {
    if (!importPreview.value || !preparedImport || importing.value || confirmingImport) return;
    const source = preparedImport;
    confirmingImport = true;
    importing.value = true;
    try {
      let result: DictionaryImportResult;
      if (source.kind === "url") {
        result = await api.importDictionaryUrl(source.url, source.name);
      } else {
        result = await api.importDictionary(source.name, source.data);
      }
      await loadAllStats();
      ElMessage.success(
        `已导入 ${result.imported_entries.toLocaleString()} 条到 ${result.name}` +
          (result.skipped_entries ? `，跳过 ${result.skipped_entries.toLocaleString()} 条` : ""),
      );
      if (enableAfterImport && !(await addDictionaryReference(result.reference))) {
        ElMessage.warning("词库文件已导入，但未能加入当前方案。请在本地词库列表中重试加入。");
      }
      showImportPreviewDialog.value = false;
      importSourceName.value = "";
      importData.value = new Uint8Array(0);
      importOnlineId.value = "";
      importUrl.value = "";
      importUrlSourceName.value = "";
      importPreview.value = undefined;
      preparedImport = undefined;
    } catch (error) {
      ElMessage.error(String(error));
    } finally {
      importing.value = false;
      confirmingImport = false;
    }
  }

  async function exportDictionary(dict: DictInfo) {
    exportingDict.value = dict.name;
    try {
      const result = await api.exportDictionary(dict.name);
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
    if (updatingReference.value) return false;
    updatingReference.value = reference;
    try {
      dictConfig.value = await api.addDictionaryToCurrentSchema(reference);
      await loadAllStats();
      ElMessage.success("已加入当前方案，重新部署后生效");
      return true;
    } catch (error) {
      ElMessage.error(String(error));
      return false;
    } finally {
      updatingReference.value = undefined;
    }
  }

  async function removeDictionaryReference(reference: string) {
    if (updatingReference.value) return;
    updatingReference.value = reference;
    try {
      dictConfig.value = await api.removeDictionaryFromCurrentSchema(reference);
      await loadAllStats();
      ElMessage.success("已从当前方案移除引用");
    } catch (error) {
      ElMessage.error(String(error));
    } finally {
      updatingReference.value = undefined;
    }
  }

  async function moveReference(reference: string, direction: -1 | 1) {
    if (!dictConfig.value || updatingReference.value) return;
    const imports = [...dictConfig.value.imports];
    const index = imports.indexOf(reference);
    const nextIndex = index + direction;
    if (index < 0 || nextIndex < 0 || nextIndex >= imports.length) return;
    [imports[index], imports[nextIndex]] = [imports[nextIndex], imports[index]];

    updatingReference.value = reference;
    try {
      dictConfig.value = await api.saveDictionaryImports(imports);
      await loadAllStats();
    } catch (error) {
      ElMessage.error(String(error));
    } finally {
      updatingReference.value = undefined;
    }
  }

  async function deleteDictionary(dict: DictInfo) {
    if (deletingDict.value) return;
    deletingDict.value = dict.name;
    try {
      try {
        await ElMessageBox.confirm(`确定删除「${dict.name}」？此操作不可恢复。`, "删除词库", {
          confirmButtonText: "删除",
          cancelButtonText: "取消",
          type: "warning",
        });
      } catch {
        return;
      }
      await api.deleteDictionary(dict.name);
      if (expandedDict.value === dict.name) {
        ++healthVersion;
        expandedDict.value = null;
        dictHealth.value = null;
        healthLoading.value = false;
      }
      ElMessage.success("词库已删除");
      await loadAllStats();
    } catch (error) {
      ElMessage.error(String(error));
    } finally {
      deletingDict.value = undefined;
    }
  }

  async function cleanDuplicateLines(dictName: string) {
    if (cleaningDict.value || expandedDict.value !== dictName) return;
    if (!dictHealth.value?.duplicate_exact_lines) {
      ElMessage.info("这个词库没有重复词条");
      return;
    }

    cleaningDict.value = dictName;
    try {
      await ElMessageBox.confirm(
        `将从「${dictName}」中移除 ${dictHealth.value.duplicate_exact_lines.toLocaleString()} 条完全重复的词条行。执行前会自动创建保存前备份。`,
        "清理重复词条",
        { confirmButtonText: "清理", cancelButtonText: "取消", type: "warning" },
      );
    } catch {
      cleaningDict.value = undefined;
      return;
    }

    try {
      const result = await api.cleanDictionaryDuplicates(dictName);
      await loadAllStats();
      const version = healthVersion;
      const health = await api.getDictHealth(dictName);
      if (version === healthVersion && expandedDict.value === dictName) dictHealth.value = health;
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

  const orderedReferences = computed(() => {
    if (!dictConfig.value) return [];
    const byName = new Map(
      [...dictConfig.value.enabled, ...dictConfig.value.missing].map((reference) => [
        reference.reference,
        reference,
      ]),
    );
    return dictConfig.value.imports.flatMap((name) => {
      const reference = byName.get(name);
      return reference ? [reference] : [];
    });
  });

  const totalEntries = computed(() => dictionaries.value.reduce((s, d) => s + d.entry_count, 0));
  const totalSize = computed(() => dictionaries.value.reduce((s, d) => s + d.size_bytes, 0));
  const enabledCount = computed(
    () => (dictConfig.value?.enabled.length ?? 0) + (dictConfig.value?.missing.length ?? 0),
  );

  async function selectOnlineCategory(categoryId: string) {
    selectedOnlineCategory.value = categoryId;
    await loadCategoryDictionaries();
  }

  async function loadAllStats() {
    await loadDictionaries();
  }

  onMounted(() => {
    void listen<LmdgDownloadProgress>("lmdg-download-progress", (event) => {
      if (!disposed) lmdgDownloadProgress.value = event.payload;
    })
      .then((unlisten) => {
        if (disposed) unlisten();
        else unlistenLmdgProgress = unlisten;
      })
      .catch((error) => {
        if (!disposed) ElMessage.error(`下载进度监听失败: ${String(error)}`);
      });
    void Promise.all([loadAllStats(), loadOnlineDictionaries(), loadCategoryDictionaries()]);
  });

  onUnmounted(() => {
    disposed = true;
    ++healthVersion;
    ++categoryVersion;
    ++listVersion;
    ++previewVersion;
    unlistenLmdgProgress?.();
  });

  return {
    // Refs
    dictionaries,
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
