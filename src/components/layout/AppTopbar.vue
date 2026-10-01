<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import { Refresh, Search, SwitchButton, UploadFilled } from "@element-plus/icons-vue";
import TypingSandbox from "../common/TypingSandbox.vue";
import AppCommandPalette from "./AppCommandPalette.vue";

defineProps<{
  pageTitle: string;
  pageDescription: string;
  scanning: boolean;
  hasDeployer: boolean;
  deploying: boolean;
  restartingServer: boolean;
  busy: boolean;
}>();

const emit = defineEmits<{
  refresh: [];
  deploy: [];
  restartServer: [];
  navigate: [key: string];
  createBackup: [];
  previewTheme: [name: string];
}>();

const showPalette = ref(false);

function openCommandPalette() {
  showPalette.value = true;
}

function handleGlobalKeydown(e: KeyboardEvent) {
  if (
    !showPalette.value &&
    e.target instanceof window.Element &&
    e.target.closest(".el-dialog, .el-message-box")
  )
    return;
  if (!e.isComposing && !e.repeat && (e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
    e.preventDefault();
    showPalette.value = !showPalette.value;
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleGlobalKeydown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleGlobalKeydown);
});
</script>

<template>
  <header class="topbar">
    <!-- Clean integrated breadcrumb / title -->
    <div class="topbar-title-group">
      <div class="breadcrumb-pill">
        <span class="weasel-chip">Rime Studio</span>
        <span class="breadcrumb-slash">/</span>
        <h2 class="title-text">{{ pageTitle }}</h2>
      </div>
    </div>

    <!-- Center interactive Raycast-style command trigger -->
    <button
      type="button"
      class="command-pill"
      aria-label="打开快捷命令（Ctrl+K）"
      aria-haspopup="dialog"
      :aria-expanded="showPalette"
      title="快速搜索功能、直接跳转页面或切换主题 (Ctrl+K)"
      @click="openCommandPalette"
    >
      <el-icon :size="13" class="search-icon"><Search /></el-icon>
      <span class="command-placeholder">搜索页面、操作、配色…</span>
      <kbd class="shortcut-key">Ctrl+K</kbd>
    </button>

    <!-- Right toolbar action buttons -->
    <div class="toolbar-actions">
      <!-- Status pill -->
      <div
        class="engine-status-dock"
        :title="
          hasDeployer ? '已找到部署工具；运行状态请通过输入测试确认' : '请检查小狼毫安装后刷新'
        "
      >
        <span class="pulse-dot" :class="{ unavailable: !hasDeployer }" />
        <span class="engine-text">{{
          scanning ? "正在扫描" : hasDeployer ? "已检测到小狼毫" : "未检测到小狼毫"
        }}</span>
      </div>

      <TypingSandbox />

      <el-button
        :loading="scanning"
        :icon="Refresh"
        size="small"
        class="action-btn"
        title="刷新 Rime 状态"
        aria-label="刷新 Rime 状态"
        :disabled="scanning || busy"
        @click="$emit('refresh')"
      />

      <el-button
        :disabled="!hasDeployer || busy"
        :loading="restartingServer"
        :icon="SwitchButton"
        size="small"
        class="action-btn"
        title="重启小狼毫后台服务 (WeaselServer.exe)"
        @click="$emit('restartServer')"
      >
        重启服务
      </el-button>

      <el-button
        type="primary"
        :disabled="!hasDeployer || busy"
        :loading="deploying"
        :icon="UploadFilled"
        size="small"
        class="deploy-btn"
        @click="$emit('deploy')"
      >
        部署已保存配置
      </el-button>
    </div>

    <!-- Raycast Command Palette Modal -->
    <AppCommandPalette
      v-model:visible="showPalette"
      :busy="busy"
      :has-deployer="hasDeployer"
      @preview-theme="emit('previewTheme', $event)"
      @navigate="emit('navigate', $event)"
      @deploy="emit('deploy')"
      @restart-server="emit('restartServer')"
      @create-backup="emit('createBackup')"
    />
  </header>
</template>

<style scoped>
.topbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  flex: 0 0 auto;
  margin-bottom: 12px;
  padding: 4px 0 8px;
  gap: 16px;
  border-bottom: 1px solid var(--color-line-soft);
  min-height: 42px;
}

.topbar-title-group {
  display: flex;
  align-items: center;
  min-width: 0;
}

.breadcrumb-pill {
  display: flex;
  align-items: center;
  gap: 8px;
}

.weasel-chip {
  display: inline-flex;
  align-items: center;
  font-size: 10px;
  font-weight: 800;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--brand-700);
  background: var(--brand-50);
  border: 1px solid var(--brand-200);
  padding: 2px 8px;
  border-radius: var(--radius-full);
}

html[data-theme="dark"] .weasel-chip {
  color: var(--brand-300);
  background: rgba(37, 99, 235, 0.15);
  border-color: rgba(59, 130, 246, 0.3);
}

.breadcrumb-slash {
  color: var(--color-line);
  font-weight: 600;
  font-size: 13px;
}

.title-text {
  margin: 0;
  color: var(--ink-900);
  font-size: 15px;
  font-weight: 800;
  letter-spacing: -0.01em;
  line-height: 1.2;
}

/* Command Pill */
.command-pill {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 12px;
  background: var(--color-surface);
  border: 1px solid var(--color-line);
  border-radius: var(--radius-full);
  color: var(--ink-400);
  font-size: 12px;
  width: 260px;
  box-shadow: var(--shadow-xs);
  cursor: pointer;
  transition: all var(--transition-fast);
  outline: none;
}

.command-pill:hover {
  border-color: var(--brand-400);
  background: var(--color-surface-hover);
  color: var(--brand-600);
}

.search-icon {
  color: var(--ink-400);
}

.command-placeholder {
  flex: 1;
  text-align: left;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.shortcut-key {
  font-family: var(--font-mono);
  font-size: 10px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-line-soft);
  padding: 1px 5px;
  border-radius: 4px;
  color: var(--ink-500);
}

/* Actions */
.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.engine-status-dock {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-full);
  font-size: 11px;
  color: var(--ink-600);
  font-weight: 650;
}

.pulse-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--emerald-500, #10b981);
  box-shadow: 0 0 6px rgba(16, 185, 129, 0.6);
}

.action-btn {
  font-weight: 550;
  border-radius: var(--radius-sm);
}

.deploy-btn {
  font-weight: 650;
  border-radius: var(--radius-sm);
  background: linear-gradient(135deg, var(--brand-600), var(--brand-700));
  border: none;
  box-shadow: 0 2px 8px rgba(37, 99, 235, 0.28);
  transition: all var(--transition-fast);
}

.deploy-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 14px rgba(37, 99, 235, 0.4);
}
</style>

<style scoped>
.command-pill:focus-visible {
  outline: 2px solid var(--brand-500);
  outline-offset: 3px;
}
.pulse-dot.unavailable {
  background: var(--ink-400);
  box-shadow: none;
}
@media (max-width: 1250px) {
  .topbar {
    flex-wrap: wrap;
    gap: 10px;
  }
  .command-pill {
    width: 220px;
  }
  .engine-status-dock {
    display: none;
  }
}
@media (max-width: 900px) {
  .toolbar-actions {
    flex-wrap: wrap;
  }
  .command-pill {
    flex: 1;
    min-width: 150px;
  }
  .weasel-chip,
  .breadcrumb-slash {
    display: none;
  }
}
</style>
