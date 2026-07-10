<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { invoke } from "@tauri-apps/api/core";
import { EditorView, basicSetup } from "codemirror";
import { yaml } from "@codemirror/lang-yaml";
import { oneDark } from "@codemirror/theme-one-dark";
import { EditorState } from "@codemirror/state";
import { Refresh, UploadFilled, MagicStick } from "@element-plus/icons-vue";
import { useErrorHandler } from "../composables/useErrorHandler";
import type { FileStatus, RimeEnvironment } from "../types";

const props = defineProps<{ env?: RimeEnvironment }>();
const emit = defineEmits<{ saved: []; deploy: [] }>();
const { withErrorHandling } = useErrorHandler();

const files = ref<FileStatus[]>([]);
const selectedFile = ref<FileStatus | null>(null);
const dirty = ref(false);
const saving = ref(false);
const loading = ref(false);

const editorContainer = ref<HTMLDivElement>();
let editorView: EditorView | null = null;
let originalContent = "";

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
const hasFiles = computed(() => files.value.length > 0);
const headerTitle = computed(() => {
  if (!selectedFile.value) return "配置编辑器";
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
    try {
      await ElMessageBox.confirm(
        `"${selectedFile.value.name}" 有未保存的修改，确定要切换文件吗？`,
        "未保存的修改",
        { confirmButtonText: "切换", cancelButtonText: "取消", type: "warning" },
      );
    } catch {
      return;
    }
  }

  selectedFile.value = file;
  await readFileContent(file);
}

async function handleSave() {
  if (!selectedFile.value) {
    ElMessage.warning("请先选择一个文件");
    return;
  }

  const content = editorView?.state.doc.toString() ?? "";
  saving.value = true;
  const result = await withErrorHandling(() =>
    invoke<void>("write_config_file_content", {
      filename: selectedFile.value!.name,
      content,
    }),
  );
  saving.value = false;

  if (result === undefined) {
    return;
  }

  originalContent = content;
  dirty.value = false;
  ElMessage.success(`已保存 ${selectedFile.value.name}`);
  emit("saved");
}

function handleDeploy() {
  emit("deploy");
}

async function handleRefresh() {
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

  ElMessage.success("文件列表已刷新");
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

onMounted(async () => {
  initEditor();
  await nextTick();
  await loadFiles();
});

onBeforeUnmount(() => {
  destroyEditor();
});
</script>

<template>
  <section class="content-grid config-editor-grid">
    <aside class="file-sidebar">
      <div class="file-sidebar-header">
        <h3>YAML 配置文件</h3>
        <span v-if="hasFiles" class="file-count">{{ files.length }} 个文件</span>
      </div>

      <div v-if="hasFiles" class="file-list">
        <button
          v-for="file in files"
          :key="file.name"
          class="file-list-item"
          :class="{ active: selectedFile?.name === file.name }"
          @click="selectFile(file)"
        >
          <span class="file-name">{{ file.name }}</span>
          <span
            v-if="selectedFile?.name === file.name && dirty"
            class="dirty-dot"
            aria-hidden="true"
          />
        </button>
      </div>

      <div v-else class="file-empty">
        <el-empty description="暂无 YAML 配置文件" />
      </div>

      <div class="file-sidebar-footer">
        <el-button :icon="Refresh" size="small" @click="handleRefresh">刷新</el-button>
      </div>
    </aside>

    <section class="editor-main">
      <header class="editor-header">
        <div class="editor-title">
          <h2>{{ headerTitle }}</h2>
          <p v-if="selectedFilePath">{{ selectedFilePath }}</p>
        </div>
        <div class="editor-actions">
          <el-button :icon="Refresh" :disabled="loading" @click="handleRefresh">刷新</el-button>
          <el-button
            type="primary"
            :icon="UploadFilled"
            :loading="saving"
            :disabled="!dirty || !selectedFile"
            @click="handleSave"
          >
            保存
          </el-button>
          <el-button :icon="MagicStick" @click="handleDeploy">部署</el-button>
        </div>
      </header>

      <div class="editor-body" tabindex="0" @keydown="handleKeydown">
        <div v-if="loading" class="editor-loading">
          <el-icon class="is-loading"><Refresh /></el-icon>
          <span>正在读取文件内容...</span>
        </div>

        <div v-if="!selectedFile && !loading" class="editor-placeholder">
          <el-empty description="选择左侧文件开始编辑" />
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
  grid-template-columns: 280px minmax(0, 1fr);
  gap: 0;
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

.file-sidebar {
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--color-surface-soft);
  border-right: 1px solid var(--color-line-soft);
}

.file-sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 16px;
  border-bottom: 1px solid var(--color-line-soft);
}

.file-sidebar-header h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 700;
  color: var(--ink-800);
}

.file-count {
  font-size: 12px;
  color: var(--ink-500);
  font-weight: 500;
}

.file-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 8px;
}

.file-list-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  width: 100%;
  padding: 10px 12px;
  margin-bottom: 4px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--ink-700);
  font-size: 13px;
  font-weight: 500;
  text-align: left;
  cursor: pointer;
  transition:
    background var(--transition-fast),
    color var(--transition-fast),
    box-shadow var(--transition-fast);
}

.file-list-item:hover {
  background: var(--color-accent-soft);
  color: var(--color-accent);
}

.file-list-item.active {
  background: linear-gradient(135deg, var(--brand-100), var(--brand-50));
  color: var(--brand-700);
  font-weight: 700;
  box-shadow: var(--shadow-xs);
}

.file-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--font-mono);
}

.dirty-dot {
  flex-shrink: 0;
  width: 6px;
  height: 6px;
  border-radius: var(--radius-full);
  background: var(--color-accent);
}

.file-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 0;
}

.file-sidebar-footer {
  display: flex;
  justify-content: flex-end;
  padding: 12px 16px;
  border-top: 1px solid var(--color-line-soft);
}

.editor-main {
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--color-surface);
}

.editor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 14px 20px;
  border-bottom: 1px solid var(--color-line-soft);
}

.editor-title {
  min-width: 0;
}

.editor-title h2 {
  margin: 0;
  font-size: 15px;
  font-weight: 700;
  color: var(--ink-800);
}

.editor-title p {
  margin: 2px 0 0;
  font-size: 12px;
  color: var(--ink-500);
  font-family: var(--font-mono);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.editor-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.editor-body {
  position: relative;
  flex: 1;
  min-height: 0;
  overflow: hidden;
  outline: none;
}

.editor-loading {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--ink-500);
  font-size: 13px;
  z-index: 2;
}

.editor-loading .el-icon {
  font-size: 24px;
}

.editor-placeholder {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1;
}

.codemirror-host {
  width: 100%;
  height: 100%;
  overflow: auto;
}

.codemirror-host.hidden {
  display: none;
}

.codemirror-host :deep(.cm-editor) {
  height: 100%;
  font-family: var(--font-mono);
  font-size: 13px;
}

.codemirror-host :deep(.cm-scroller) {
  overflow: auto;
}
</style>
