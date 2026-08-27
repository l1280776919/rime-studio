<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { invoke } from "@tauri-apps/api/core";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { EditorView, basicSetup } from "codemirror";
import { yaml } from "@codemirror/lang-yaml";
import { oneDark } from "@codemirror/theme-one-dark";
import { EditorState } from "@codemirror/state";
import {
  Document,
  FolderOpened,
  MagicStick,
  Refresh,
  Search,
  UploadFilled,
} from "@element-plus/icons-vue";
import { useErrorHandler } from "../composables/useErrorHandler";
import type { FileStatus, RimeEnvironment } from "../types";

const props = defineProps<{ env?: RimeEnvironment }>();
const emit = defineEmits<{ saved: []; deploy: []; dirtyChange: [dirty: boolean] }>();
const { withErrorHandling } = useErrorHandler();

const files = ref<FileStatus[]>([]);
const selectedFile = ref<FileStatus | null>(null);
const dirty = ref(false);
const saving = ref(false);
const loading = ref(false);
const fileSearch = ref("");
const sidebarTab = ref<"key" | "all">("key");

const editorContainer = ref<HTMLDivElement>();
let editorView: EditorView | null = null;
let originalContent = "";
let allowWindowClose = false;
let unlistenCloseRequested: UnlistenFn | undefined;

type KeyConfigFile = {
  name: string;
  label: string;
  group: "core" | "ui" | "data";
  role: string;
};

const KEY_CONFIG_FILES: KeyConfigFile[] = [
  {
    name: "default.custom.yaml",
    label: "默认方案配置",
    group: "core",
    role: "当前方案、候选数、按键绑定",
  },
  {
    name: "weasel.custom.yaml",
    label: "小狼毫外观配置",
    group: "ui",
    role: "主题、字号、候选窗方向",
  },
  {
    name: "rime_ice.custom.yaml",
    label: "雾凇组件配置",
    group: "ui",
    role: "Emoji、繁简、标点、全角开关",
  },
  {
    name: "custom_phrase.txt",
    label: "自定义短语",
    group: "data",
    role: "短语、编码、权重",
  },
  {
    name: "rime_ice.schema.yaml",
    label: "雾凇方案",
    group: "core",
    role: "雾凇输入方案入口",
  },
  {
    name: "rime_ice.dict.yaml",
    label: "雾凇主词库",
    group: "data",
    role: "雾凇基础词条",
  },
  {
    name: "sogou_ext.dict.yaml",
    label: "搜狗扩展词库",
    group: "data",
    role: "导入词库条目",
  },
];

const filesByName = computed(() => {
  return new Map((files.value ?? []).map((file) => [file.name, file]));
});

const keyFileRows = computed(() => {
  const customMap = new Map((props.env?.custom_files ?? []).map((f) => [f.name, f]));
  return KEY_CONFIG_FILES.map((meta) => ({
    ...meta,
    file: filesByName.value.get(meta.name) ?? customMap.get(meta.name),
  }));
});

const filteredAllFiles = computed(() => {
  const q = fileSearch.value.trim().toLowerCase();
  if (!q) return files.value;
  return files.value.filter((f) => f.name.toLowerCase().includes(q));
});

function getEditorExtensions() {
  return [
    basicSetup,
    yaml(),
    oneDark,
    EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        dirty.value = update.view.state.doc.toString() !== originalContent;
      }
    }),
  ];
}

const selectedFilePath = computed(() => selectedFile.value?.path ?? "");
const headerTitle = computed(() => {
  if (!selectedFile.value) return "配置中心 & YAML 编辑器";
  return `${selectedFile.value.name}${dirty.value ? " [已修改]" : ""}`;
});

async function loadFiles() {
  const result = await withErrorHandling(() => invoke<FileStatus[]>("list_yaml_config_files"));
  if (result !== undefined) {
    files.value = result;
  }
}

async function readFileContent(file: FileStatus) {
  loading.value = true;
  const content = await withErrorHandling(() =>
    invoke<string>("read_config_file_content", { filename: file.name }),
  );
  loading.value = false;

  if (content === undefined) {
    return;
  }

  originalContent = content;
  setEditorContent(content);
}

async function selectFile(file: FileStatus) {
  if (dirty.value && selectedFile.value && selectedFile.value.name !== file.name) {
    const confirmed = await confirmDiscard("切换文件", "切换");
    if (!confirmed) return;
  }

  selectedFile.value = file;
  await readFileContent(file);
}

async function handleSave(): Promise<boolean> {
  if (!selectedFile.value) {
    ElMessage.warning("请先选择一个文件");
    return false;
  }

  const content = editorView?.state.doc.toString() ?? "";
  saving.value = true;
  let result: boolean | undefined;
  try {
    result = await withErrorHandling(() =>
      invoke<boolean>("write_config_file_content", {
        filename: selectedFile.value!.name,
        content,
      }),
    );
  } finally {
    saving.value = false;
  }

  if (!result) return false;

  originalContent = content;
  dirty.value = false;
  ElMessage.success(`已保存 ${selectedFile.value.name}`);
  emit("saved");
  return true;
}

async function handleDeploy() {
  if (dirty.value) {
    try {
      await ElMessageBox.confirm(
        `"${selectedFile.value?.name ?? "当前文件"}" 有未保存的修改。保存成功后再部署？`,
        "保存并部署",
        {
          confirmButtonText: "保存并部署",
          cancelButtonText: "取消",
          type: "warning",
        },
      );
    } catch {
      return;
    }

    if (!(await handleSave())) return;
  }

  emit("deploy");
}

async function handleRefresh() {
  if (dirty.value && !(await confirmDiscard("刷新文件", "丢弃并刷新"))) {
    return;
  }

  const previousName = selectedFile.value?.name;
  await loadFiles();

  if (previousName && selectedFile.value?.name !== previousName) {
    const match = files.value.find((file) => file.name === previousName);
    if (match) {
      selectedFile.value = match;
    }
  }

  if (selectedFile.value) {
    await readFileContent(selectedFile.value);
  }

  ElMessage.success("配置文件已刷新");
}

async function openRimeUserDir() {
  await withErrorHandling(() => invoke("open_rime_user_dir"));
}

async function confirmDiscard(action: string, confirmButtonText: string): Promise<boolean> {
  try {
    await ElMessageBox.confirm(
      `"${selectedFile.value?.name ?? "当前文件"}" 有未保存的修改，确定要${action}吗？`,
      "未保存的修改",
      { confirmButtonText, cancelButtonText: "取消", type: "warning" },
    );
    return true;
  } catch {
    return false;
  }
}

function setEditorContent(content: string) {
  if (!editorView) return;

  editorView.setState(
    EditorState.create({
      doc: content,
      extensions: getEditorExtensions(),
    }),
  );
}

function handleKeydown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
    event.preventDefault();
    void handleSave();
  }
}

function initEditor() {
  if (!editorContainer.value) return;

  editorView = new EditorView({
    state: EditorState.create({
      doc: "",
      extensions: getEditorExtensions(),
    }),
    parent: editorContainer.value,
  });
}

function destroyEditor() {
  if (editorView) {
    editorView.destroy();
    editorView = null;
  }
}

watch(
  () => props.env?.user_dir,
  async (newDir, oldDir) => {
    if (newDir && newDir !== oldDir) {
      await loadFiles();
    }
  },
);

watch(
  dirty,
  (value) => {
    emit("dirtyChange", value);
  },
  { immediate: true },
);

onMounted(async () => {
  initEditor();
  await nextTick();
  await loadFiles();

  // Select default.custom.yaml if present
  if (!selectedFile.value && files.value.length > 0) {
    const def = files.value.find((f) => f.name === "default.custom.yaml") ?? files.value[0];
    await selectFile(def);
  }

  unlistenCloseRequested = await getCurrentWindow().onCloseRequested(async (event) => {
    if (!dirty.value || allowWindowClose) return;

    event.preventDefault();
    const confirmed = await confirmDiscard("关闭 Rime Studio", "放弃修改并关闭");
    if (!confirmed) return;

    allowWindowClose = true;
    await getCurrentWindow().close();
  });
});

onBeforeUnmount(() => {
  unlistenCloseRequested?.();
  destroyEditor();
});
</script>

<template>
  <section class="content-grid config-editor-grid">
    <aside class="file-sidebar">
      <div class="file-sidebar-header">
        <div class="header-title-box">
          <h3>配置中心</h3>
          <span class="file-count">{{ files.length }} 个文件</span>
        </div>
        <div class="header-actions">
          <el-button
            :icon="FolderOpened"
            circle
            size="small"
            title="在资源管理器中打开 Rime 目录"
            @click="openRimeUserDir"
          />
          <el-button :icon="Refresh" circle size="small" title="刷新文件" @click="handleRefresh" />
        </div>
      </div>

      <!-- Tab Switch: Key vs All -->
      <div class="sidebar-tabs">
        <el-radio-group v-model="sidebarTab" size="small" style="width: 100%">
          <el-radio-button value="key" style="width: 50%">核心配置</el-radio-button>
          <el-radio-button value="all" style="width: 50%">全部文件</el-radio-button>
        </el-radio-group>
      </div>

      <!-- Key Config Files List -->
      <div v-if="sidebarTab === 'key'" class="file-list key-files-list">
        <div
          v-for="item in keyFileRows"
          :key="item.name"
          class="key-file-card"
          :class="{
            active: selectedFile?.name === item.name,
            missing: !item.file?.exists,
          }"
          @click="item.file?.exists ? selectFile(item.file) : null"
        >
          <div class="card-top">
            <span class="card-label">{{ item.label }}</span>
            <el-tag size="small" :type="item.file?.exists ? 'success' : 'info'">
              {{ item.file?.exists ? "已生成" : "未生成" }}
            </el-tag>
          </div>
          <div class="card-name">{{ item.name }}</div>
          <div class="card-role">{{ item.role }}</div>
        </div>
      </div>

      <!-- All YAML Files List -->
      <div v-else class="all-files-wrapper">
        <div class="search-input-box">
          <el-input
            v-model="fileSearch"
            size="small"
            placeholder="搜索配置文件..."
            :prefix-icon="Search"
            clearable
          />
        </div>

        <div v-if="filteredAllFiles.length > 0" class="file-list">
          <button
            v-for="file in filteredAllFiles"
            :key="file.name"
            class="file-list-item"
            :class="{ active: selectedFile?.name === file.name }"
            @click="selectFile(file)"
          >
            <div class="file-item-left">
              <el-icon><Document /></el-icon>
              <span class="file-name">{{ file.name }}</span>
            </div>
            <span
              v-if="selectedFile?.name === file.name && dirty"
              class="dirty-dot"
              aria-hidden="true"
            />
          </button>
        </div>

        <div v-else class="file-empty">
          <el-empty description="未找到匹配文件" />
        </div>
      </div>
    </aside>

    <section class="editor-main">
      <header class="editor-header">
        <div class="editor-title">
          <h2>{{ headerTitle }}</h2>
          <p v-if="selectedFilePath">{{ selectedFilePath }}</p>
        </div>
        <div class="editor-actions">
          <el-button :icon="Refresh" :disabled="loading" @click="handleRefresh">重新读取</el-button>
          <el-button
            type="primary"
            :icon="UploadFilled"
            :loading="saving"
            :disabled="!dirty || !selectedFile"
            @click="handleSave"
          >
            保存 (Ctrl+S)
          </el-button>
          <el-button type="success" plain :icon="MagicStick" @click="handleDeploy">
            保存并重新部署
          </el-button>
        </div>
      </header>

      <div class="editor-body" tabindex="0" @keydown="handleKeydown">
        <div v-if="loading" class="editor-loading">
          <el-icon class="is-loading"><Refresh /></el-icon>
          <span>正在读取文件内容...</span>
        </div>

        <div v-if="!selectedFile && !loading" class="editor-placeholder">
          <el-empty description="选择左侧配置文件开始编辑" />
        </div>

        <div
          ref="editorContainer"
          class="codemirror-host"
          :class="{ hidden: !selectedFile || loading }"
        />
      </div>
    </section>
  </section>
</template>

<style scoped>
.config-editor-grid {
  display: grid;
  grid-template-columns: 320px minmax(0, 1fr);
  gap: 0;
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

.file-sidebar {
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--color-surface-soft, #f8fafc);
  border-right: 1px solid var(--color-line-soft, #edf2f7);
}

.file-sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  border-bottom: 1px solid var(--color-line-soft, #edf2f7);
}

.header-title-box h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--ink-900, #0f172a);
}

.file-count {
  font-size: 11px;
  color: var(--color-muted, #64748b);
}

.header-actions {
  display: flex;
  gap: 6px;
}

.sidebar-tabs {
  padding: 10px 14px;
  border-bottom: 1px solid var(--color-line-soft, #edf2f7);
}

.sidebar-tabs :deep(.el-radio-button__inner) {
  width: 100%;
}

.file-list {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.key-files-list {
  gap: 8px;
}

.key-file-card {
  background: var(--color-surface, #ffffff);
  border: 1px solid var(--color-line, #e2e8f0);
  border-radius: var(--radius-sm, 8px);
  padding: 10px 12px;
  cursor: pointer;
  transition: all 0.15s ease;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.key-file-card:hover {
  border-color: var(--brand-400, #60a5fa);
  transform: translateY(-1px);
}

.key-file-card.active {
  background: var(--brand-50, #eff6ff);
  border-color: var(--brand-500, #3b82f6);
}

.key-file-card.missing {
  opacity: 0.6;
  cursor: not-allowed;
}

.card-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--ink-800, #1e293b);
}

.card-name {
  font-size: 11px;
  font-family: var(--font-mono, monospace);
  color: var(--color-muted, #64748b);
}

.card-role {
  font-size: 11px;
  color: var(--ink-500, #64748b);
  line-height: 1.3;
}

.all-files-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.search-input-box {
  padding: 8px 12px;
  border-bottom: 1px solid var(--color-line-soft, #edf2f7);
}

.file-list-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border: none;
  background: transparent;
  border-radius: var(--radius-xs, 6px);
  cursor: pointer;
  text-align: left;
  transition: all 0.12s ease;
}

.file-list-item:hover {
  background: var(--color-surface-hover, #eef3f9);
}

.file-list-item.active {
  background: var(--brand-50, #eff6ff);
  color: var(--brand-700, #1d4ed8);
  font-weight: 600;
}

.file-item-left {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dirty-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--amber-500, #f59e0b);
}

.editor-main {
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--color-surface, #ffffff);
}

.editor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  border-bottom: 1px solid var(--color-line-soft, #edf2f7);
}

.editor-title h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--ink-900, #0f172a);
}

.editor-title p {
  margin: 2px 0 0 0;
  font-size: 12px;
  font-family: var(--font-mono, monospace);
  color: var(--color-muted, #64748b);
}

.editor-actions {
  display: flex;
  gap: 8px;
}

.editor-body {
  flex: 1;
  position: relative;
  min-height: 0;
  outline: none;
  background: #282c34;
}

.codemirror-host {
  height: 100%;
  overflow: auto;
}

.codemirror-host.hidden {
  display: none;
}

.codemirror-host :deep(.cm-editor) {
  height: 100%;
  font-family: var(--font-mono, monospace);
  font-size: 14px;
}

.editor-loading,
.editor-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #94a3b8;
  gap: 12px;
}
</style>
