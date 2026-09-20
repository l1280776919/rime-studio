<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { ElMessage } from "element-plus";
import { api } from "../api";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  Check,
  Collection,
  Connection,
  FolderOpened,
  Link,
  Refresh,
  UploadFilled,
  Warning,
} from "@element-plus/icons-vue";
import pkg from "../../package.json";
import appLogo from "../assets/logo.png";
import type { AppUpdateInfo } from "../types";
import { useErrorHandler } from "../composables/useErrorHandler";

const { withErrorHandling } = useErrorHandler();
const checkingUpdate = ref(false);
const downloadingUpdate = ref(false);
const downloadStatus = ref("");
const updateInfo = ref<AppUpdateInfo>();
const updateCheckFailed = ref(false);

const releaseNotesPreview = computed(() => {
  const notes = updateInfo.value?.release_notes?.trim();
  if (!notes) return "";
  return notes.length > 600 ? `${notes.slice(0, 600)}...` : notes;
});

const updateState = computed(() => {
  if (updateCheckFailed.value) {
    return {
      tone: "error",
      title: "检查更新失败",
      detail: "暂时无法连接 GitHub Releases，请检查网络设置或代理后重试。",
      tagType: "danger" as const,
      tagText: "网络异常",
      actionText: "重试检查",
    };
  }
  if (!updateInfo.value) {
    return {
      tone: "idle",
      title: "检查软件更新",
      detail: "自动连接 GitHub Releases 获取最新发布版本与安装包。",
      tagType: "info" as const,
      tagText: "待检查",
      actionText: "检查更新",
    };
  }
  if (updateInfo.value.update_available) {
    return {
      tone: "available",
      title: `发现新版本 ${updateInfo.value.latest_version}`,
      detail: "已有更新版本可用，点击可自动下载并启动安装向导。",
      tagType: "warning" as const,
      tagText: "可升级",
      actionText: "一键升级",
    };
  }
  return {
    tone: "current",
    title: "当前已是最新正式版本",
    detail: "本机版本与 GitHub 最新 Release 完全一致，无需更新。",
    tagType: "success" as const,
    tagText: "最新版",
    actionText: "重新检查",
  };
});

function formatPublishedAt(value?: string) {
  if (!value) return "未知时间";
  return new Date(value).toLocaleString();
}

async function checkUpdate() {
  if (checkingUpdate.value) return;
  checkingUpdate.value = true;
  updateCheckFailed.value = false;
  try {
    const result = await withErrorHandling(() => api.checkAppUpdate());
    if (!result) {
      updateInfo.value = undefined;
      updateCheckFailed.value = true;
      return;
    }

    updateInfo.value = result;
    if (result.update_available) {
      ElMessage.success(`发现新版本 ${result.latest_version}`);
    } else {
      ElMessage.success("当前已是最新版本");
    }
  } finally {
    checkingUpdate.value = false;
  }
}

async function openReleasePage() {
  const url =
    updateInfo.value?.release_url ?? "https://github.com/l1280776919/rime-studio/releases";
  await openUrl(url);
}

async function handleUpdateAction() {
  if (!updateInfo.value || !updateInfo.value.update_available) {
    await checkUpdate();
    return;
  }

  downloadingUpdate.value = true;
  downloadStatus.value = "正在下载更新...";

  try {
    const result = await api.downloadAppUpdate();

    if (!result.success || !result.installer_path) {
      ElMessage.warning("下载失败，将跳转到发布页面手动下载");
      await openReleasePage();
      return;
    }

    downloadStatus.value = "正在启动安装程序...";
    await api.launchRimeInstaller(result.installer_path);
    ElMessage.success("安装程序已启动，请按提示完成安装");
    downloadStatus.value = "安装程序已启动";
  } catch (error) {
    ElMessage.warning(`自动更新失败: ${String(error)}，将跳转到发布页面`);
    await openReleasePage();
  } finally {
    downloadingUpdate.value = false;
  }
}

async function openLogDir() {
  const dir = await withErrorHandling(() => api.openAppLogDir());
  if (dir) {
    ElMessage.success(`已打开日志目录：${dir}`);
  }
}

onMounted(() => {
  void checkUpdate();
});
</script>

<template>
  <div class="about-studio-container">
    <!-- Brand Hero Showcase -->
    <div class="about-hero panel">
      <div class="brand-showcase">
        <div class="brand-avatar-box">
          <img :src="appLogo" alt="Rime Studio Logo" class="brand-logo-img" />
        </div>

        <div class="brand-meta">
          <div class="brand-badge-line">
            <span class="version-capsule">v{{ pkg.version }}</span>
            <span class="build-tag">Official Release</span>
          </div>
          <h1 class="brand-title">Rime Studio</h1>
          <p class="brand-tagline">
            专为中州韵 / 小狼毫 (Weasel) 打造的下一代现代化桌面级输入法控制工作台
          </p>
          <p class="brand-description">
            基于 Tauri 2 + Rust + Vue 3 深度构建。提供视网膜级实景候选窗预览、方案库调度、词库语料管理、智能无损 Patch 合并引擎与时光机自动快照备份。
          </p>

          <div class="brand-quick-actions">
            <el-button :icon="FolderOpened" @click="openLogDir">
              打开应用运行日志
            </el-button>
            <el-button :icon="Link" @click="openReleasePage">
              GitHub 源码仓库
            </el-button>
          </div>
        </div>
      </div>
    </div>

    <!-- Update Status Card -->
    <div class="about-update-card panel" :class="`is-${updateState.tone}`">
      <div class="update-header-bar">
        <div class="update-status-icon-wrap">
          <el-icon>
            <Warning v-if="updateInfo?.update_available" />
            <Check v-else-if="updateInfo && !updateCheckFailed" />
            <Refresh v-else />
          </el-icon>
        </div>

        <div class="update-status-meta">
          <div class="update-title-row">
            <strong>{{ updateState.title }}</strong>
            <el-tag :type="updateState.tagType" size="small" effect="light">
              {{ updateState.tagText }}
            </el-tag>
          </div>
          <p class="update-detail-desc">{{ updateState.detail }}</p>
        </div>

        <div class="update-action-btn-wrap">
          <el-button
            type="primary"
            :icon="downloadingUpdate ? Refresh : updateInfo?.update_available ? UploadFilled : Refresh"
            :loading="checkingUpdate || downloadingUpdate"
            :disabled="downloadingUpdate"
            @click="handleUpdateAction"
          >
            {{ downloadingUpdate ? downloadStatus : updateState.actionText }}
          </el-button>
        </div>
      </div>

      <!-- Version comparison pills -->
      <div class="update-version-pills">
        <div class="version-tile">
          <span class="v-label">本机安装版本</span>
          <strong class="v-val">v{{ updateInfo?.current_version ?? pkg.version }}</strong>
          <small class="v-sub">本地运行中</small>
        </div>

        <div class="version-tile latest-tile">
          <span class="v-label">GitHub 最新发布</span>
          <strong class="v-val">{{ updateInfo?.latest_version ?? "尚未获取" }}</strong>
          <small class="v-sub">
            {{ updateInfo ? `发布于 ${formatPublishedAt(updateInfo.published_at)}` : "点击右侧按钮获取" }}
          </small>
        </div>
      </div>

      <!-- Release Notes -->
      <div v-if="updateInfo && releaseNotesPreview" class="update-notes-container">
        <header class="notes-header">
          <span>Release Notes（发布说明）</span>
          <small>{{ updateInfo.release_name ?? updateInfo.latest_version }}</small>
        </header>
        <pre class="notes-pre">{{ releaseNotesPreview }}</pre>
      </div>
    </div>

    <!-- Ecosystem & Tech Stack Matrix -->
    <div class="about-ecosystem-grid">
      <!-- Links Bento -->
      <div class="panel links-panel">
        <h3 class="bento-title">社区生态与开源项目</h3>

        <div class="links-cards-grid">
          <a
            href="https://github.com/l1280776919/rime-studio"
            target="_blank"
            class="ecosystem-card"
          >
            <div class="eco-icon-box">
              <el-icon><Collection /></el-icon>
            </div>
            <div class="eco-meta">
              <strong>Rime Studio</strong>
              <span>本项目 GitHub 源码仓库与发版动态</span>
            </div>
            <el-icon class="eco-arrow"><Link /></el-icon>
          </a>

          <a
            href="https://github.com/l1280776919/rime-studio/issues"
            target="_blank"
            class="ecosystem-card"
          >
            <div class="eco-icon-box">
              <el-icon><Warning /></el-icon>
            </div>
            <div class="eco-meta">
              <strong>问题与建议反馈</strong>
              <span>提交 Bug 反馈、需求建议或技术交流</span>
            </div>
            <el-icon class="eco-arrow"><Link /></el-icon>
          </a>

          <a
            href="https://github.com/rime/home"
            target="_blank"
            class="ecosystem-card"
          >
            <div class="eco-icon-box">
              <el-icon><Connection /></el-icon>
            </div>
            <div class="eco-meta">
              <strong>Rime 中州韻</strong>
              <span>开源跨平台输入法核心引擎项目主页</span>
            </div>
            <el-icon class="eco-arrow"><Link /></el-icon>
          </a>

          <a
            href="https://github.com/iDvel/rime-ice"
            target="_blank"
            class="ecosystem-card"
          >
            <div class="eco-icon-box">
              <el-icon><Connection /></el-icon>
            </div>
            <div class="eco-meta">
              <strong>雾凇拼音 (rime-ice)</strong>
              <span>长期维护的精细化简体中文拼音词库方案</span>
            </div>
            <el-icon class="eco-arrow"><Link /></el-icon>
          </a>
        </div>
      </div>

      <!-- Architecture Stack Card -->
      <div class="panel tech-panel">
        <h3 class="bento-title">技术架构与系统环境</h3>

        <div class="tech-tiles-list">
          <div class="tech-tile">
            <span class="tech-label">客户端内核</span>
            <strong class="tech-val">Tauri 2.0 (Rust)</strong>
          </div>
          <div class="tech-tile">
            <span class="tech-label">前端视网膜渲染</span>
            <strong class="tech-val">Vue 3 + Vite 6 + TypeScript</strong>
          </div>
          <div class="tech-tile">
            <span class="tech-label">组件设计系统</span>
            <strong class="tech-val">Element Plus + Fluent Glass</strong>
          </div>
          <div class="tech-tile">
            <span class="tech-label">输入法目标平台</span>
            <strong class="tech-val">Weasel (小狼毫) for Windows</strong>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.about-studio-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* Brand Hero */
.about-hero {
  padding: 24px 28px;
  background: linear-gradient(135deg, var(--brand-50, #eff6ff) 0%, var(--color-surface) 60%);
}

html[data-theme="dark"] .about-hero {
  background: linear-gradient(135deg, rgba(37, 99, 235, 0.12) 0%, var(--color-surface) 60%);
}

.brand-showcase {
  display: flex;
  align-items: flex-start;
  gap: 24px;
}

.brand-avatar-box {
  width: 72px;
  height: 72px;
  border-radius: var(--radius-lg);
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  box-shadow: 0 8px 24px -4px rgba(37, 99, 235, 0.15);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px;
  flex-shrink: 0;
}

.brand-logo-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.brand-meta {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.brand-badge-line {
  display: flex;
  align-items: center;
  gap: 8px;
}

.version-capsule {
  font-size: 11px;
  font-weight: 800;
  color: var(--brand-600);
  background: var(--brand-50, #eff6ff);
  border: 1px solid var(--brand-200);
  padding: 1px 8px;
  border-radius: var(--radius-full);
}

.build-tag {
  font-size: 11px;
  color: var(--ink-500);
  font-weight: 600;
}

.brand-title {
  margin: 0;
  font-size: 24px;
  font-weight: 850;
  color: var(--ink-900);
  letter-spacing: -0.03em;
}

.brand-tagline {
  margin: 0;
  font-size: 14px;
  font-weight: 650;
  color: var(--ink-700);
}

.brand-description {
  margin: 4px 0 0;
  font-size: 12px;
  color: var(--color-muted);
  line-height: 1.6;
  max-width: 680px;
}

.brand-quick-actions {
  display: flex;
  gap: 10px;
  margin-top: 10px;
}

/* Update Card */
.about-update-card {
  padding: 18px 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.update-header-bar {
  display: flex;
  align-items: center;
  gap: 14px;
}

.update-status-icon-wrap {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-sm);
  background: var(--color-surface-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  color: var(--brand-600);
  flex-shrink: 0;
}

.is-current .update-status-icon-wrap {
  background: rgba(16, 185, 129, 0.12);
  color: #10b981;
}

.is-available .update-status-icon-wrap {
  background: rgba(245, 158, 11, 0.15);
  color: #f59e0b;
}

.update-status-meta {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.update-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.update-title-row strong {
  font-size: 14px;
  font-weight: 750;
  color: var(--ink-900);
}

.update-detail-desc {
  margin: 0;
  font-size: 12px;
  color: var(--color-muted);
}

.update-version-pills {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.version-tile {
  padding: 12px 14px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-sm);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.v-label {
  font-size: 11px;
  font-weight: 700;
  color: var(--ink-500);
}

.v-val {
  font-size: 16px;
  font-weight: 800;
  color: var(--ink-900);
}

.v-sub {
  font-size: 11px;
  color: var(--color-muted);
}

.update-notes-container {
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.notes-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: var(--color-surface-soft);
  border-bottom: 1px solid var(--color-line-soft);
  font-size: 12px;
  font-weight: 700;
  color: var(--ink-800);
}

.notes-pre {
  margin: 0;
  padding: 12px;
  font-size: 11px;
  font-family: var(--font-mono);
  background: var(--color-surface);
  line-height: 1.5;
  max-height: 200px;
  overflow-y: auto;
}

/* Ecosystem Grid */
.about-ecosystem-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 300px;
  gap: 16px;
  align-items: start;
}

.links-panel,
.tech-panel {
  padding: 18px 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.bento-title {
  margin: 0;
  font-size: 14px;
  font-weight: 750;
  color: var(--ink-900);
}

.links-cards-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.ecosystem-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 14px;
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-sm);
  text-decoration: none;
  transition: all var(--transition-fast);
}

.ecosystem-card:hover {
  transform: translateY(-1px);
  border-color: var(--brand-300);
  box-shadow: var(--shadow-xs);
}

.eco-icon-box {
  color: var(--brand-600);
  font-size: 18px;
  flex-shrink: 0;
}

.eco-meta {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.eco-meta strong {
  font-size: 12px;
  font-weight: 750;
  color: var(--ink-900);
}

.eco-meta span {
  font-size: 11px;
  color: var(--color-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.eco-arrow {
  color: var(--ink-400);
  font-size: 12px;
}

.tech-tiles-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tech-tile {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: var(--color-surface-soft);
  border-radius: var(--radius-xs);
}

.tech-label {
  font-size: 11px;
  font-weight: 650;
  color: var(--ink-600);
}

.tech-val {
  font-size: 11px;
  font-weight: 750;
  color: var(--ink-900);
  font-family: var(--font-mono);
}
</style>
