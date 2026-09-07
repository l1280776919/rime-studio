<script setup lang="ts">
import { onMounted, ref } from "vue";
import { ElMessage } from "element-plus";
import { api } from "../../api";
import { Edit, Refresh } from "@element-plus/icons-vue";
import type { LuaPluginInfo } from "../../types";
import { useErrorHandler } from "../../composables/useErrorHandler";

const emit = defineEmits<{
  change: [];
  deploy: [];
}>();

const loading = ref(false);
const toggling = ref<string | null>(null);
const plugins = ref<LuaPluginInfo[]>([]);
const editDialogVisible = ref(false);
const editingPlugin = ref<LuaPluginInfo | null>(null);
const scriptContent = ref("");
const savingScript = ref(false);

const { withErrorHandling } = useErrorHandler();

async function loadPlugins() {
  loading.value = true;
  try {
    const list = await withErrorHandling(() => api.listLuaPlugins());
    if (list) {
      plugins.value = list;
    }
  } finally {
    loading.value = false;
  }
}

async function handleToggle(plugin: LuaPluginInfo, enabled: boolean) {
  toggling.value = plugin.id;
  try {
    const result = await withErrorHandling(() => api.toggleLuaPlugin(plugin.id, enabled));
    if (result) {
      plugins.value = result;
      ElMessage.success(`${plugin.name} 已${enabled ? "启用" : "禁用"}，点击右上角部署即可生效`);
      emit("change");
    }
  } finally {
    toggling.value = null;
  }
}

async function openScriptEditor(plugin: LuaPluginInfo) {
  editingPlugin.value = plugin;
  editDialogVisible.value = true;
  const content = await withErrorHandling(() => api.getLuaScriptContent(plugin.id));
  if (content !== undefined) {
    scriptContent.value = content;
  }
}

async function saveScript() {
  if (!editingPlugin.value) return;
  savingScript.value = true;
  try {
    await withErrorHandling(() =>
      api.saveLuaScriptContent(editingPlugin.value!.id, scriptContent.value),
    );
    ElMessage.success("Lua 脚本已保存");
    editDialogVisible.value = false;
  } finally {
    savingScript.value = false;
  }
}

onMounted(() => {
  loadPlugins();
});

defineExpose({
  loadPlugins,
});
</script>

<template>
  <div class="lua-plugin-manager panel">
    <div class="panel-header">
      <div>
        <h2>Lua 扩展脚本管理</h2>
        <p>启用或微调小狼毫 Lua 动态脚本扩展，增强输入法实时运算与动态功能。</p>
      </div>
      <el-button :icon="Refresh" circle size="small" :loading="loading" @click="loadPlugins" />
    </div>

    <div v-loading="loading" class="plugin-cards-grid">
      <div
        v-for="plugin in plugins"
        :key="plugin.id"
        class="plugin-card"
        :class="{ active: plugin.enabled }"
      >
        <div class="card-header">
          <div class="plugin-title-row">
            <span class="plugin-name">{{ plugin.name }}</span>
            <el-tag size="small" :type="plugin.enabled ? 'success' : 'info'">
              {{ plugin.enabled ? "已启用" : "未启用" }}
            </el-tag>
          </div>
          <el-switch
            :model-value="plugin.enabled"
            :loading="toggling === plugin.id"
            @change="(val: boolean | string | number) => handleToggle(plugin, Boolean(val))"
          />
        </div>

        <p class="plugin-desc">{{ plugin.description }}</p>

        <div class="plugin-preview-box">
          <span class="preview-label">触发规则：</span>
          <code>{{ plugin.trigger_preview }}</code>
        </div>

        <div class="card-footer">
          <span class="file-name">{{ plugin.file_name }}</span>
          <el-button
            link
            type="primary"
            size="small"
            :icon="Edit"
            @click="openScriptEditor(plugin)"
          >
            编辑脚本
          </el-button>
        </div>
      </div>
    </div>

    <!-- Script Editor Dialog -->
    <el-dialog
      v-model="editDialogVisible"
      :title="`编辑 Lua 脚本 - ${editingPlugin?.name} (${editingPlugin?.file_name})`"
      width="640px"
      append-to-body
    >
      <div class="script-editor-body">
        <el-input
          v-model="scriptContent"
          type="textarea"
          :rows="14"
          placeholder="-- Lua Script"
          class="code-textarea"
        />
      </div>
      <template #footer>
        <el-button @click="editDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="savingScript" @click="saveScript"> 保存脚本 </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.lua-plugin-manager {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.panel-header h2 {
  font-size: 16px;
  font-weight: 600;
  color: var(--ink-900, #0f172a);
  margin: 0 0 4px 0;
}

.panel-header p {
  font-size: 13px;
  color: var(--color-muted, #64748b);
  margin: 0;
}

.plugin-cards-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.plugin-card {
  background: var(--color-surface, #ffffff);
  border: 1px solid var(--color-line, #e2e8f0);
  border-radius: var(--radius-md, 12px);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  transition: all 0.2s ease;
}

.plugin-card.active {
  border-color: var(--brand-400, #60a5fa);
  box-shadow: var(--shadow-sm, 0 1px 3px rgba(0, 0, 0, 0.05));
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.plugin-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.plugin-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--ink-800, #1e293b);
}

.plugin-desc {
  font-size: 12px;
  color: var(--color-muted, #64748b);
  margin: 0;
  line-height: 1.5;
  flex: 1;
}

.plugin-preview-box {
  background: var(--color-surface-soft, #f8fafc);
  padding: 6px 10px;
  border-radius: var(--radius-xs, 6px);
  font-size: 12px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.preview-label {
  color: var(--color-muted, #64748b);
}

.plugin-preview-box code {
  font-family: var(--font-mono, monospace);
  color: var(--brand-700, #1d4ed8);
  font-weight: 500;
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-top: 1px solid var(--color-line-soft, #edf2f7);
  padding-top: 8px;
  margin-top: 4px;
}

.file-name {
  font-size: 11px;
  font-family: var(--font-mono, monospace);
  color: var(--color-muted, #64748b);
}

.script-editor-body {
  margin-bottom: 8px;
}

.code-textarea :deep(textarea) {
  font-family: var(--font-mono, monospace);
  font-size: 13px;
  line-height: 1.5;
  background: var(--ink-900, #0f172a);
  color: #f8fafc;
}
</style>
