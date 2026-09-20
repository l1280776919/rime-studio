<script setup lang="ts">
import {
  Brush,
  Collection,
  Document,
  EditPen,
  Files,
  FolderOpened,
  InfoFilled,
  MagicStick,
  Monitor,
  Moon,
  Sunny,
} from "@element-plus/icons-vue";
import type { Component } from "vue";
import type { RimeEnvironment } from "../../types";
import { useTheme } from "../../composables/useTheme";
import appLogo from "../../assets/logo.png";

defineProps<{
  env?: RimeEnvironment;
  activePage: string;
}>();

const emit = defineEmits<{
  navigate: [key: string];
}>();

const { isDark, toggleTheme } = useTheme();

interface NavItem {
  key: string;
  label: string;
  icon: Component;
  badge?: string;
}

interface NavGroup {
  name: string;
  items: NavItem[];
}

const navGroups: NavGroup[] = [
  {
    name: "常用控制",
    items: [
      { key: "overview", label: "概览与状态", icon: Monitor },
      { key: "quick", label: "快速设置", icon: MagicStick },
      { key: "appearance", label: "候选窗主题", icon: Brush },
    ],
  },
  {
    name: "方案与资产",
    items: [
      { key: "schemas", label: "方案管理", icon: Files },
      { key: "dictionaries", label: "词库管理", icon: Collection },
      { key: "phrases", label: "自定义短语", icon: EditPen },
    ],
  },
  {
    name: "系统与维护",
    items: [
      { key: "editor", label: "配置中心", icon: Document },
      { key: "backups", label: "快照与备份", icon: FolderOpened },
      { key: "about", label: "关于应用", icon: InfoFilled },
    ],
  },
];
</script>

<template>
  <aside class="sidebar">
    <!-- Brand Header -->
    <div class="brand">
      <div class="brand-info">
        <img :src="appLogo" alt="Rime Studio" class="brand-mark" />
        <div class="brand-text">
          <span class="brand-title">Rime Studio</span>
          <span class="brand-subtitle">小狼毫配置工作台</span>
        </div>
      </div>
      <button
        type="button"
        class="theme-btn"
        :title="isDark ? '切换至浅色模式' : '切换至深色模式'"
        @click="toggleTheme"
      >
        <el-icon :size="15">
          <Sunny v-if="isDark" />
          <Moon v-else />
        </el-icon>
      </button>
    </div>

    <!-- Categorized Navigation -->
    <div class="nav-container custom-scrollbar">
      <div v-for="group in navGroups" :key="group.name" class="nav-group">
        <span class="nav-group-title">{{ group.name }}</span>
        <div class="nav-group-items">
          <button
            v-for="item in group.items"
            :key="item.key"
            type="button"
            class="nav-pill"
            :class="{ active: activePage === item.key }"
            @click="emit('navigate', item.key)"
          >
            <el-icon class="nav-icon"><component :is="item.icon" /></el-icon>
            <span class="nav-label">{{ item.label }}</span>
            <span v-if="item.badge" class="nav-badge">{{ item.badge }}</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Bottom Status Dock -->
    <div class="sidebar-dock">
      <div class="dock-header">
        <div class="status-indicator">
          <span class="pulse-dot" :class="{ online: Boolean(env?.user_dir) }"></span>
          <span class="status-text">{{ env?.user_dir ? "引擎就绪" : "等待连接" }}</span>
        </div>
        <span v-if="env?.active_schema" class="schema-tag">{{ env.active_schema }}</span>
      </div>
      <div class="dock-path" :title="env?.user_dir ?? '未连接'">
        {{ env?.user_dir ?? "%APPDATA%\\Rime" }}
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  overflow: hidden;
  padding: 16px 12px 14px;
  background: var(--color-sidebar-bg);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border-right: 1px solid var(--color-line-soft);
  user-select: none;
}

/* Brand */
.brand {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px 16px;
  border-bottom: 1px solid var(--color-line-soft);
  margin-bottom: 12px;
}

.brand-info {
  display: flex;
  align-items: center;
  gap: 10px;
}

.brand-mark {
  width: 34px;
  height: 34px;
  border-radius: var(--radius-md);
  object-fit: contain;
  box-shadow: 0 4px 12px rgba(15, 23, 42, 0.08);
  transition: transform var(--transition-spring);
}

.brand-mark:hover {
  transform: scale(1.08) rotate(-3deg);
}

.brand-text {
  display: flex;
  flex-direction: column;
}

.brand-title {
  color: var(--ink-900);
  font-size: 15px;
  font-weight: 800;
  letter-spacing: -0.02em;
  line-height: 1.2;
}

.brand-subtitle {
  color: var(--ink-500);
  font-size: 11px;
  font-weight: 500;
}

.theme-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: var(--radius-full);
  border: 1px solid var(--color-line-soft);
  background: var(--color-surface);
  color: var(--ink-600);
  transition: all var(--transition-fast);
}

.theme-btn:hover {
  background: var(--color-surface-hover);
  color: var(--color-accent);
  border-color: var(--brand-200);
  transform: scale(1.06);
}

/* Nav Container */
.nav-container {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding-right: 2px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.nav-group {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.nav-group-title {
  padding: 4px 8px 2px;
  font-size: 10px;
  font-weight: 700;
  color: var(--ink-400);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.nav-group-items {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.nav-pill {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 8px 10px;
  border-radius: var(--radius-md);
  border: 1px solid transparent;
  background: transparent;
  color: var(--ink-700);
  font-size: 13px;
  font-weight: 550;
  text-align: left;
  transition: all var(--transition-fast);
  cursor: pointer;
}

.nav-pill:hover {
  background: var(--color-surface-hover);
  color: var(--ink-900);
}

.nav-pill.active {
  background: linear-gradient(135deg, var(--brand-50), rgba(219, 234, 254, 0.6));
  color: var(--brand-700);
  border-color: rgba(59, 130, 246, 0.25);
  font-weight: 650;
  box-shadow: 0 1px 3px rgba(37, 99, 235, 0.06);
}

html[data-theme="dark"] .nav-pill.active {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.22), rgba(30, 64, 175, 0.28));
  color: var(--brand-300);
  border-color: rgba(59, 130, 246, 0.35);
  box-shadow: 0 0 12px -2px rgba(59, 130, 246, 0.2);
}

.nav-icon {
  font-size: 16px;
  color: var(--ink-500);
  transition: transform var(--transition-fast), color var(--transition-fast);
}

.nav-pill:hover .nav-icon {
  color: var(--ink-800);
  transform: scale(1.08);
}

.nav-pill.active .nav-icon {
  color: var(--brand-600);
  transform: scale(1.1);
}

html[data-theme="dark"] .nav-pill.active .nav-icon {
  color: var(--brand-400);
}

.nav-label {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.nav-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: var(--radius-full);
  background: var(--brand-100);
  color: var(--brand-800);
}

/* Bottom Status Dock */
.sidebar-dock {
  margin-top: auto;
  padding: 10px 12px;
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-xs);
  display: flex;
  flex-direction: column;
  gap: 5px;
  transition: border-color var(--transition-base), box-shadow var(--transition-base);
}

.sidebar-dock:hover {
  border-color: var(--brand-200);
  box-shadow: var(--shadow-sm);
}

.dock-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
}

.pulse-dot {
  width: 7px;
  height: 7px;
  border-radius: var(--radius-full);
  background: var(--amber-500);
  box-shadow: 0 0 6px rgba(245, 158, 11, 0.4);
  flex-shrink: 0;
}

.pulse-dot.online {
  background: var(--emerald-500);
  box-shadow: 0 0 6px rgba(16, 185, 129, 0.5);
}

.status-text {
  font-size: 11px;
  font-weight: 650;
  color: var(--ink-700);
}

.schema-tag {
  font-size: 10px;
  font-weight: 600;
  color: var(--brand-700);
  background: var(--brand-50);
  padding: 1px 6px;
  border-radius: 4px;
  max-width: 80px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

html[data-theme="dark"] .schema-tag {
  color: var(--brand-300);
  background: rgba(37, 99, 235, 0.2);
}

.dock-path {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--ink-400);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
