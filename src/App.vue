<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { invoke } from "@tauri-apps/api/core";
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
  Refresh,
  Sunny,
  UploadFilled,
} from "@element-plus/icons-vue";
import AboutPage from "./pages/AboutPage.vue";
import AppearancePage from "./pages/AppearancePage.vue";
import BackupsPage from "./pages/BackupsPage.vue";
import ConfigFilesPage from "./pages/ConfigFilesPage.vue";
import DictionariesPage from "./pages/DictionariesPage.vue";
import OverviewPage from "./pages/OverviewPage.vue";
import PhrasesPage from "./pages/PhrasesPage.vue";
import QuickSettingsPage from "./pages/QuickSettingsPage.vue";
import SchemasPage from "./pages/SchemasPage.vue";
import type {
  BackupEntry,
  DeployResult,
  InstallResult,
  RestoreResult,
  RimeEnvironment,
} from "./types";

type PageKey = "overview" | "quick" | "schemas" | "configs" | "appearance" | "phrases" | "dictionaries" | "backups" | "about";

const PAGE_KEYS: ReadonlySet<string> = new Set<PageKey>([
  "overview",
  "quick",
  "schemas",
  "configs",
  "appearance",
  "phrases",
  "dictionaries",
  "backups",
  "about",
]);

function isPageKey(value: string): value is PageKey {
  return PAGE_KEYS.has(value);
}

function navigateTo(key: string) {
  if (isPageKey(key)) {
    activePage.value = key;
  }
}

const activePage = ref<PageKey>("overview");
const env = ref<RimeEnvironment>();
const backups = ref<BackupEntry[]>([]);
const status = ref("启动中...");
const log = ref("");
const scanning = ref(false);
const deploying = ref(false);
const backingUp = ref(false);
const restoringBackup = ref<string>();
const deletingBackup = ref<string>();
const installingRecipe = ref<string>();
const elapsedSeconds = ref(0);
let elapsedTimer: ReturnType<typeof setInterval> | undefined;

const DARK_THEME_KEY = "rime-studio-theme";
const isDark = ref(false);

function applyTheme(dark: boolean) {
  if (dark) {
    document.documentElement.dataset.theme = "dark";
  } else {
    delete document.documentElement.dataset.theme;
  }
  localStorage.setItem(DARK_THEME_KEY, dark ? "dark" : "light");
}

function toggleTheme() {
  isDark.value = !isDark.value;
  applyTheme(isDark.value);
}

function initTheme() {
  const stored = localStorage.getItem(DARK_THEME_KEY);
  if (stored) {
    isDark.value = stored === "dark";
  } else {
    isDark.value = window.matchMedia("(prefers-color-scheme: dark)").matches;
  }
  applyTheme(isDark.value);
}

const hasDeployer = computed(() => Boolean(env.value?.deployer_path));
const pageTitle = computed(() => {
  const titles: Record<PageKey, string> = {
    overview: "Rime 配置控制台",
    quick: "快速设置",
    schemas: "方案管理",
    configs: "配置文件",
    appearance: "主题配置",
    phrases: "短语管理",
    dictionaries: "词库管理",
    backups: "备份管理",
    about: "关于",
  };
  return titles[activePage.value];
});
const pageDescription = computed(() => {
  const descriptions: Record<PageKey, string> = {
    overview: "管理方案、外观、词库与部署状态。",
    quick: "集中调整雾凇方案、候选数量、按键和候选窗行为。",
    schemas: "查看、启用、复制和定位本机 Rime 输入方案。",
    configs: "集中查看、定位和备份 Rime 配置文件。",
    appearance: "调整小狼毫候选窗主题、字号、边距和颜色。",
    phrases: "编辑自定义短语，支持添加、搜索、导入和批量管理。",
    dictionaries: "浏览和管理 Rime 词库文件，查看条目统计与健康状态。",
    backups: "查看、打开和恢复 Rime Studio 创建的配置备份。",
    about: "关于 Rime Studio 与相关开源项目。",
  };
  return descriptions[activePage.value];
});

async function loadBackups() {
  backups.value = await invoke<BackupEntry[]>("list_backups");
}

async function loadEnvironment() {
  scanning.value = true;
  status.value = "正在扫描 Rime 配置...";

  try {
    env.value = await invoke<RimeEnvironment>("scan_rime_environment");
    await loadBackups();
    status.value = "扫描完成";
  } catch (error) {
    status.value = String(error);
    ElMessage.error(String(error));
  } finally {
    scanning.value = false;
  }
}

async function createManualBackup() {
  backingUp.value = true;
  status.value = "正在创建配置备份...";

  try {
    const backup = await invoke<BackupEntry>("create_backup");
    await loadBackups();
    status.value = `已创建备份：${backup.name}`;
    ElMessage.success("备份已创建");
  } catch (error) {
    status.value = String(error);
    ElMessage.error(String(error));
  } finally {
    backingUp.value = false;
  }
}

async function openKnownPath(command: "open_rime_user_dir" | "open_plum_dir") {
  try {
    await invoke(command);
  } catch (error) {
    ElMessage.error(String(error));
  }
}

async function openBackupDir(backup: BackupEntry) {
  try {
    await invoke("open_backup_dir", { backupName: backup.name });
  } catch (error) {
    ElMessage.error(String(error));
  }
}

async function restoreBackup(backup: BackupEntry) {
  try {
    await ElMessageBox.confirm(
      `将恢复备份 ${backup.name} 中的 ${backup.files} 个文件。恢复前会先为当前配置创建一份安全备份。`,
      "恢复备份",
      {
        confirmButtonText: "恢复",
        cancelButtonText: "取消",
        type: "warning",
      },
    );
  } catch {
    return;
  }

  restoringBackup.value = backup.name;
  status.value = `正在恢复备份：${backup.name}`;

  try {
    const result = await invoke<RestoreResult>("restore_backup", { backupName: backup.name });
    await loadEnvironment();
    status.value = `已恢复 ${result.restored_files} 个文件，恢复前备份：${result.safety_backup_dir}`;
    ElMessage.success("备份已恢复");
  } catch (error) {
    status.value = String(error);
    ElMessage.error(String(error));
  } finally {
    restoringBackup.value = undefined;
  }
}

async function deleteBackupEntry(backup: BackupEntry) {
  try {
    await ElMessageBox.confirm(
      `确定删除备份 ${backup.name}（${backup.files} 个文件）？此操作不可恢复。`,
      "删除备份",
      { confirmButtonText: "删除", cancelButtonText: "取消", type: "warning" },
    );
  } catch {
    return;
  }

  deletingBackup.value = backup.name;
  try {
    await invoke("delete_backup", { backupName: backup.name });
    await loadBackups();
    ElMessage.success("备份已删除");
  } catch (error) {
    ElMessage.error(String(error));
  } finally {
    deletingBackup.value = undefined;
  }
}

async function deploy() {
  deploying.value = true;
  status.value = "正在重新部署小狼毫...";

  try {
    const result = await invoke<DeployResult>("deploy_rime");
    await loadEnvironment();
    status.value = result.message;
    ElMessage({
      type: result.success ? "success" : "warning",
      message: result.message,
    });
  } catch (error) {
    status.value = String(error);
    ElMessage.error(String(error));
  } finally {
    deploying.value = false;
  }
}

async function installRimeIce(recipe: string) {
  installingRecipe.value = recipe;
  status.value = `正在安装 ${recipe}...`;
  log.value = "正在准备安装器...";

  try {
    const result = await invoke<InstallResult>("install_rime_ice", { recipe });
    await loadEnvironment();
    status.value = result.success
      ? `已安装 ${result.recipe}`
      : `安装失败：${result.recipe}`;
    log.value = result.log;
    ElMessage({
      type: result.success ? "success" : "error",
      message: result.success ? "rime-ice 安装完成" : "rime-ice 安装失败",
    });
  } catch (error) {
    status.value = String(error);
    log.value = String(error);
    ElMessage.error(String(error));
  } finally {
    installingRecipe.value = undefined;
  }
}

const isBusy = computed(
  () => scanning.value || deploying.value || backingUp.value || restoringBackup.value || installingRecipe.value,
);

function startElapsedTimer() {
  stopElapsedTimer();
  elapsedSeconds.value = 0;
  elapsedTimer = setInterval(() => {
    elapsedSeconds.value++;
  }, 1000);
}

function stopElapsedTimer() {
  if (elapsedTimer !== undefined) {
    clearInterval(elapsedTimer);
    elapsedTimer = undefined;
  }
}

watch(isBusy, (busy) => {
  if (busy) {
    startElapsedTimer();
  } else {
    stopElapsedTimer();
    elapsedSeconds.value = 0;
  }
});

function formatElapsed(seconds: number) {
  if (seconds < 60) return `${seconds}s`;
  const min = Math.floor(seconds / 60);
  const sec = seconds % 60;
  return `${min}m ${sec}s`;
}

onMounted(() => {
  initTheme();
  loadEnvironment();
});
</script>

<template>
  <el-config-provider>
    <main class="studio-shell">
      <aside class="sidebar">
        <div class="brand">
          <div class="brand-mark">R</div>
          <div>
            <h1>Rime Studio</h1>
            <p>小狼毫配置工作台</p>
          </div>
        </div>

        <el-menu class="nav-menu" :default-active="activePage" @select="navigateTo">
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

      <section class="workspace surface">
        <header class="topbar">
          <div>
            <span class="eyebrow">Windows Weasel</span>
            <h2>{{ pageTitle }}</h2>
            <p>{{ pageDescription }}</p>
          </div>
          <div class="toolbar-actions">
            <el-button :loading="scanning" :icon="Refresh" @click="loadEnvironment">
              刷新
            </el-button>
            <el-button
              type="primary"
              :disabled="!hasDeployer"
              :loading="deploying"
              :icon="UploadFilled"
              @click="deploy"
            >
              重新部署
            </el-button>
          </div>
        </header>

        <div class="page-container">
          <Transition name="page" mode="out-in">
            <OverviewPage
              v-if="activePage === 'overview'"
              key="overview"
              :env="env"
              :backups="backups"
              :log="log"
              :scanning="scanning"
              :backing-up="backingUp"
              :restoring-backup="restoringBackup"
              :installing-recipe="installingRecipe"
              :deleting-backup="deletingBackup"
              @create-backup="createManualBackup"
              @open-path="openKnownPath"
              @install="installRimeIce"
              @open-backup="openBackupDir"
              @restore-backup="restoreBackup"
              @delete-backup="deleteBackupEntry"
            />

            <QuickSettingsPage
              v-else-if="activePage === 'quick'"
              key="quick"
              :env="env"
              :installing-recipe="installingRecipe"
              @saved="loadEnvironment"
              @deploy="deploy"
              @install="installRimeIce"
            />

            <SchemasPage
              v-else-if="activePage === 'schemas'"
              key="schemas"
              :env="env"
              @saved="loadEnvironment"
              @deploy="deploy"
            />

            <ConfigFilesPage
              v-else-if="activePage === 'configs'"
              key="configs"
              :env="env"
              :backing-up="backingUp"
              @refresh="loadEnvironment"
              @create-backup="createManualBackup"
            />

            <AppearancePage
              v-else-if="activePage === 'appearance'"
              key="appearance"
              :env="env"
              @saved="loadEnvironment"
              @deploy="deploy"
            />

            <PhrasesPage
              v-else-if="activePage === 'phrases'"
              key="phrases"
              :env="env"
              @saved="loadEnvironment"
              @deploy="deploy"
            />

            <DictionariesPage
              v-else-if="activePage === 'dictionaries'"
              key="dictionaries"
              :env="env"
              @open-path="openKnownPath"
            />

            <BackupsPage
              v-else-if="activePage === 'backups'"
              key="backups"
              :backups="backups"
              :backing-up="backingUp"
              :restoring-backup="restoringBackup"
              :deleting-backup="deletingBackup"
              @create-backup="createManualBackup"
              @open-backup="openBackupDir"
              @restore-backup="restoreBackup"
              @delete-backup="deleteBackupEntry"
            />

            <AboutPage
              v-else-if="activePage === 'about'"
              key="about"
            />

          </Transition>
        </div>

        <footer class="statusbar" :class="{ busy: isBusy }">
          <span>{{ status }}<template v-if="isBusy && elapsedSeconds"> (已用时 {{ formatElapsed(elapsedSeconds) }})</template></span>
        </footer>
      </section>
    </main>
  </el-config-provider>
</template>
