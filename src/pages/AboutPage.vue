<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { ElMessage } from "element-plus";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  Check,
  Collection,
  Connection,
  Link,
  Refresh,
  UploadFilled,
  Warning,
} from "@element-plus/icons-vue";
import pkg from "../../package.json";
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
      detail: "暂时无法连接 GitHub Releases，请检查网络后重试。",
      tagType: "danger" as const,
      tagText: "检查失败",
      actionText: "重试",
    };
  }
  if (!updateInfo.value) {
    return {
      tone: "idle",
      title: "检查更新",
      detail: "从 GitHub Releases 获取最新版本。",
      tagType: "info" as const,
      tagText: "待检查",
      actionText: "检查更新",
    };
  }
  if (updateInfo.value.update_available) {
    return {
      tone: "available",
      title: `发现新版本 ${updateInfo.value.latest_version}`,
      detail: "点击更新获取最新安装包。",
      tagType: "warning" as const,
      tagText: "可更新",
      actionText: "更新",
    };
  }
  return {
    tone: "current",
    title: "当前已是最新版本",
    detail: "本机版本与 GitHub 最新正式发布一致。",
    tagType: "success" as const,
    tagText: "已是最新",
    actionText: "重新检查",
  };
});

function formatPublishedAt(value?: string) {
  if (!value) return "未知";
  return new Date(value).toLocaleString();
}

async function checkUpdate() {
  if (checkingUpdate.value) return;
  checkingUpdate.value = true;
  updateCheckFailed.value = false;
  try {
    const result = await withErrorHandling(() => invoke<AppUpdateInfo>("check_app_update"));
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
    const result = await invoke<{ success: boolean; installer_path?: string; message: string }>(
      "download_app_update",
    );

    if (!result.success || !result.installer_path) {
      ElMessage.warning("下载失败，将跳转到发布页面");
      await openReleasePage();
      return;
    }

    downloadStatus.value = "正在启动安装程序...";
    await invoke("launch_rime_installer", { path: result.installer_path });
    ElMessage.success("安装程序已启动，请按提示完成安装");
    downloadStatus.value = "安装程序已启动";
  } catch (error) {
    ElMessage.warning(`自动更新失败: ${String(error)}，将跳转到发布页面`);
    await openReleasePage();
  } finally {
    downloadingUpdate.value = false;
  }
}

onMounted(() => {
  checkUpdate();
});
</script>

<template>
  <section class="content-grid about-grid">
    <section class="main-column">
      <el-card class="panel" shadow="never">
        <template #header>
          <div class="panel-title">
            <span>关于 Rime Studio</span>
          </div>
        </template>

        <div class="about-intro">
          <div class="brand-mark" style="margin-bottom: 16px">R</div>
          <h3>Rime Studio</h3>
          <p>小狼毫输入法配置工作台 v{{ pkg.version }}</p>
          <p class="helper-text">
            基于 Tauri 2 + Vue 3 + Rust 构建的桌面应用，提供图形化界面来管理 Rime
            输入法的外观主题、自定义短语、词库和配置备份。
          </p>
        </div>
      </el-card>

      <el-card class="panel" shadow="never">
        <template #header>
          <div class="panel-title">
            <span>应用更新</span>
          </div>
        </template>

        <div class="update-panel" :class="`is-${updateState.tone}`">
          <div class="update-status-card">
            <div class="update-status-icon">
              <el-icon>
                <Warning v-if="updateInfo?.update_available" />
                <Check v-else-if="updateInfo" />
                <Refresh v-else />
              </el-icon>
            </div>
            <div class="update-status-copy">
              <div>
                <strong>{{ updateState.title }}</strong>
                <el-tag :type="updateState.tagType" effect="light">
                  {{ updateState.tagText }}
                </el-tag>
              </div>
              <p>{{ updateState.detail }}</p>
            </div>
            <div class="update-action-col">
              <el-button
                type="primary"
                :icon="
                  downloadingUpdate
                    ? Refresh
                    : updateInfo?.update_available
                      ? UploadFilled
                      : Refresh
                "
                :loading="checkingUpdate || downloadingUpdate"
                :disabled="downloadingUpdate"
                @click="handleUpdateAction"
              >
                {{ downloadingUpdate ? downloadStatus : updateState.actionText }}
              </el-button>
            </div>
          </div>

          <div class="update-version-grid">
            <div class="update-version-tile">
              <span>当前版本</span>
              <strong>v{{ updateInfo?.current_version ?? pkg.version }}</strong>
              <small>已安装在本机</small>
            </div>
            <div class="update-version-tile latest">
              <span>GitHub 最新版本</span>
              <strong>{{ updateInfo?.latest_version ?? "尚未检查" }}</strong>
              <small>{{
                updateInfo
                  ? `发布于 ${formatPublishedAt(updateInfo.published_at)}`
                  : "点击检查后显示"
              }}</small>
            </div>
          </div>

          <div v-if="updateInfo" class="update-release-detail">
            <section v-if="releaseNotesPreview" class="update-notes">
              <header>
                <span>Release Notes</span>
                <small>{{ updateInfo.release_name ?? updateInfo.latest_version }}</small>
              </header>
              <pre>{{ releaseNotesPreview }}</pre>
            </section>
          </div>
        </div>
      </el-card>
    </section>

    <aside class="side-column">
      <el-card class="panel" shadow="never">
        <template #header>
          <span>相关链接</span>
        </template>
        <div class="about-links">
          <a
            href="https://github.com/l1280776919/rime-studio"
            target="_blank"
            class="about-link-card"
          >
            <el-icon><Collection /></el-icon>
            <span>
              <strong>Rime Studio</strong>
              <small>本项目 — 小狼毫配置工作台</small>
            </span>
            <el-icon class="link-arrow"><Link /></el-icon>
          </a>

          <a href="https://github.com/rime/home" target="_blank" class="about-link-card">
            <el-icon><Connection /></el-icon>
            <span>
              <strong>Rime 中州韻</strong>
              <small>输入法引擎 — 全平台开源输入法框架</small>
            </span>
            <el-icon class="link-arrow"><Link /></el-icon>
          </a>

          <a href="https://github.com/iDvel/rime-ice" target="_blank" class="about-link-card">
            <el-icon><Connection /></el-icon>
            <span>
              <strong>雾凇拼音</strong>
              <small>rime-ice — 长期维护的简体中文词库配置</small>
            </span>
            <el-icon class="link-arrow"><Link /></el-icon>
          </a>
        </div>
      </el-card>

      <el-card class="panel" shadow="never">
        <template #header>
          <span>技术栈</span>
        </template>
        <div class="tech-stack">
          <div><span>桌面框架</span><strong>Tauri 2</strong></div>
          <div><span>前端</span><strong>Vue 3 + Element Plus</strong></div>
          <div><span>后端</span><strong>Rust</strong></div>
          <div><span>平台</span><strong>Windows</strong></div>
        </div>
      </el-card>
    </aside>
  </section>
</template>
