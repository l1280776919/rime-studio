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
import type { RimeEnvironment } from "../../types";
import { useTheme } from "../../composables/useTheme";

defineProps<{
  env?: RimeEnvironment;
  activePage: string;
}>();

const emit = defineEmits<{
  navigate: [key: string];
}>();

const { isDark, toggleTheme } = useTheme();
</script>

<template>
  <aside class="sidebar">
    <div class="brand">
      <div class="brand-mark">R</div>
      <div>
        <h1>Rime Studio</h1>
        <p>小狼毫配置工作台</p>
      </div>
    </div>

    <el-menu class="nav-menu" :default-active="activePage" @select="(key: string) => emit('navigate', key)">
      <el-menu-item index="overview">
        <el-icon><Monitor /></el-icon>
        <span>概览</span>
      </el-menu-item>
      <el-menu-item index="quick">
        <el-icon><MagicStick /></el-icon>
        <span>快速设置</span>
      </el-menu-item>
      <el-menu-item index="schemas">
        <el-icon><Files /></el-icon>
        <span>方案</span>
      </el-menu-item>
      <el-menu-item index="configs">
        <el-icon><Document /></el-icon>
        <span>配置</span>
      </el-menu-item>
      <el-menu-item index="appearance">
        <el-icon><Brush /></el-icon>
        <span>主题</span>
      </el-menu-item>
      <el-menu-item index="phrases">
        <el-icon><EditPen /></el-icon>
        <span>短语</span>
      </el-menu-item>
      <el-menu-item index="dictionaries">
        <el-icon><Collection /></el-icon>
        <span>词库</span>
      </el-menu-item>
      <el-menu-item index="backups">
        <el-icon><FolderOpened /></el-icon>
        <span>备份</span>
      </el-menu-item>
      <el-menu-item index="about">
        <el-icon><InfoFilled /></el-icon>
        <span>关于</span>
      </el-menu-item>
    </el-menu>

    <div class="theme-toggle">
      <el-button
        :icon="isDark ? Sunny : Moon"
        circle
        size="small"
        @click="toggleTheme"
      />
      <span>{{ isDark ? "深色模式" : "浅色模式" }}</span>
    </div>

    <div class="sidebar-card">
      <span>用户目录</span>
      <strong>{{ env?.user_dir ? "已连接" : "等待扫描" }}</strong>
      <p>{{ env?.user_dir ?? "启动后自动读取 APPDATA 下的 Rime 目录" }}</p>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
  padding: 24px 16px 18px;
  background: #f8fafc;
  border-right: 1px solid var(--color-line-soft);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.6),
    1px 0 0 rgba(255, 255, 255, 0.3),
    2px 0 12px rgba(15, 23, 42, 0.04);
}

/* Brand */
.brand {
  display: flex;
  align-items: center;
  gap: 12px;
  margin: 0 4px 28px;
}

.brand-mark {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  flex-shrink: 0;
  background: linear-gradient(145deg, #60a5fa, #2563eb);
  border: 1px solid rgba(255, 255, 255, 0.4);
  border-radius: var(--radius-md);
  box-shadow:
    0 8px 20px rgba(37, 99, 235, 0.25),
    0 2px 6px rgba(37, 99, 235, 0.12),
    inset 0 1px 0 rgba(255, 255, 255, 0.25);
  color: #ffffff;
  font-size: 19px;
  font-weight: 850;
  letter-spacing: -0.02em;
  transition: transform var(--transition-spring), box-shadow var(--transition-spring);
}

.brand-mark:hover {
  transform: scale(1.06);
  box-shadow:
    0 10px 24px rgba(37, 99, 235, 0.3),
    0 3px 8px rgba(37, 99, 235, 0.18),
    inset 0 1px 0 rgba(255, 255, 255, 0.3);
}

.brand h1,
.brand p {
  margin: 0;
}

.brand h1 {
  color: var(--ink-900);
  font-size: 18px;
  font-weight: 800;
  letter-spacing: -0.01em;
  line-height: 1.2;
}

.brand p {
  color: var(--ink-500);
  font-size: 12px;
  margin-top: 2px;
  font-weight: 500;
}

/* Nav Menu */
.nav-menu {
  --el-menu-active-color: var(--color-accent);
  --el-menu-bg-color: transparent;
  --el-menu-hover-bg-color: var(--color-accent-soft);
  --el-menu-text-color: var(--ink-600);
  --el-menu-item-height: 44px;
  border-right: 0 !important;
  flex: 1;
  overflow-y: auto;
}

.nav-menu .el-menu-item {
  border-radius: var(--radius-sm);
  margin: 3px 0;
  height: 44px;
  line-height: 44px;
  font-weight: 560;
  transition:
    background var(--transition-fast),
    color var(--transition-fast),
    transform var(--transition-fast);
}

.nav-menu .el-menu-item:hover {
  background: var(--color-accent-soft);
  color: var(--color-accent);
}

.nav-menu .el-menu-item.is-active {
  background: linear-gradient(135deg, #dbeafe, #eff6ff);
  color: var(--brand-700);
  font-weight: 700;
  box-shadow: var(--shadow-xs);
}

.nav-menu .el-menu-item .el-icon {
  font-size: 18px;
  transition: transform var(--transition-fast);
}

.nav-menu .el-menu-item.is-active .el-icon {
  transform: scale(1.1);
}

/* Theme toggle */
.theme-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 12px;
  padding: 0 4px;
  color: var(--ink-500);
  font-size: 12px;
  font-weight: 500;
}

/* Sidebar info card */
.sidebar-card {
  margin-top: auto;
  padding: 14px 16px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.9), rgba(243, 246, 251, 0.95));
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-md);
  transition: border-color var(--transition-base), box-shadow var(--transition-base);
}

.sidebar-card:hover {
  border-color: var(--brand-200);
  box-shadow: var(--shadow-sm);
}

.sidebar-card span {
  display: block;
  color: var(--ink-500);
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.sidebar-card strong {
  display: block;
  margin-top: 4px;
  color: var(--ink-800);
  font-size: 15px;
  font-weight: 700;
}

.sidebar-card p {
  margin: 6px 0 0;
  color: var(--ink-500);
  font-family: var(--font-mono);
  font-size: 11px;
  line-height: 1.4;
  overflow-wrap: anywhere;
}
</style>
