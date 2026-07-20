<script setup lang="ts">
import { computed, defineAsyncComponent, onBeforeUnmount, onMounted, ref, watch } from "vue";
import zhCn from "element-plus/es/locale/lang/zh-cn";
import { useErrorHandler } from "./composables/useErrorHandler";
import { invoke } from "@tauri-apps/api/core";
import AppSidebar from "./components/layout/AppSidebar.vue";
import AppTopbar from "./components/layout/AppTopbar.vue";
import AppStatusbar from "./components/layout/AppStatusbar.vue";
import type { BackupEntry, RimeEnvironment } from "./types";
import { useTheme } from "./composables/useTheme";
import { useBackup } from "./composables/useBackup";
import { useDeploy } from "./composables/useDeploy";

// ── Lazy page components ──────────────────────────
const AboutPage = defineAsyncComponent(() => import("./pages/AboutPage.vue"));
const AppearancePage = defineAsyncComponent(() => import("./pages/AppearancePage.vue"));
const BackupsPage = defineAsyncComponent(() => import("./pages/BackupsPage.vue"));
const ConfigEditorPage = defineAsyncComponent(() => import("./pages/ConfigEditorPage.vue"));
const ConfigFilesPage = defineAsyncComponent(() => import("./pages/ConfigFilesPage.vue"));
const DictionariesPage = defineAsyncComponent(() => import("./pages/DictionariesPage.vue"));
const OverviewPage = defineAsyncComponent(() => import("./pages/OverviewPage.vue"));
const PhrasesPage = defineAsyncComponent(() => import("./pages/PhrasesPage.vue"));
const QuickSettingsPage = defineAsyncComponent(() => import("./pages/QuickSettingsPage.vue"));
const SchemasPage = defineAsyncComponent(() => import("./pages/SchemasPage.vue"));

// ── Composables ────────────────────────────────────
const { initTheme } = useTheme();
const {
  backups,
  backingUp,
  restoringBackup,
  deletingBackup,
  loadBackups,
  createManualBackup,
  openBackupDir,
  restoreBackup,
  deleteBackupEntry,
} = useBackup();
const { deploying, installingRecipe, log, deploy, installRimeIce } = useDeploy();
const { withErrorHandling } = useErrorHandler();

// ── Navigation ─────────────────────────────────────
type PageKey =
  | "overview"
  | "quick"
  | "schemas"
  | "configs"
  | "appearance"
  | "phrases"
  | "dictionaries"
  | "backups"
  | "editor"
  | "about";

const PAGE_KEYS: ReadonlySet<string> = new Set<PageKey>([
  "overview",
  "quick",
  "schemas",
  "configs",
  "appearance",
  "phrases",
  "dictionaries",
  "backups",
  "editor",
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
const scanning = ref(false);
const status = ref("启动中...");
const elapsedSeconds = ref(0);
let elapsedTimer: ReturnType<typeof setInterval> | undefined;

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
    editor: "配置编辑器",
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
    editor: "直接编辑 Rime YAML 配置文件，支持语法高亮和自动备份。",
    about: "关于 Rime Studio 与相关开源项目。",
  };
  return descriptions[activePage.value];
});

// ── Environment ────────────────────────────────────
async function loadEnvironment() {
  scanning.value = true;
  status.value = "正在扫描 Rime 配置...";

  const result = await withErrorHandling(async () => {
    env.value = await invoke<RimeEnvironment>("scan_rime_environment");
    await loadBackups();
    return true;
  });

  if (result !== undefined) {
    status.value = "扫描完成";
  }

  scanning.value = false;
}

let envRefreshTimer: ReturnType<typeof setTimeout> | undefined;

function refreshEnvironment() {
  if (envRefreshTimer) clearTimeout(envRefreshTimer);
  envRefreshTimer = setTimeout(() => {
    loadEnvironment();
  }, 300);
}

async function openKnownPath(command: "open_rime_user_dir" | "open_plum_dir") {
  await withErrorHandling(() => invoke(command));
}

// ── Wrapper handlers (status updates) ──────────────
async function handleDeploy() {
  status.value = "正在重新部署小狼毫...";
  try {
    await deploy();
    await loadEnvironment();
    status.value = "部署完成";
  } catch (error) {
    status.value = String(error);
  }
}

async function handleInstallRimeIce(recipe: string) {
  status.value = `正在安装 ${recipe}...`;
  try {
    await installRimeIce(recipe);
    await loadEnvironment();
    status.value = `${recipe} 安装完成`;
  } catch (error) {
    status.value = String(error);
  }
}

async function handleCreateBackup() {
  status.value = "正在创建配置备份...";
  try {
    const backup = await createManualBackup();
    if (backup) status.value = `已创建备份：${backup.name}`;
  } catch (error) {
    status.value = String(error);
  }
}

async function handleRestoreBackup(backup: BackupEntry) {
  status.value = `正在恢复备份：${backup.name}`;
  try {
    await restoreBackup(backup);
    await loadEnvironment();
    status.value = "备份已恢复";
  } catch (error) {
    status.value = String(error);
  }
}

// ── Busy / Elapsed timer ──────────────────────────
const isBusy = computed<boolean>(
  () =>
    !!(
      scanning.value ||
      deploying.value ||
      backingUp.value ||
      restoringBackup.value ||
      installingRecipe.value
    ),
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

// ── Lifecycle ──────────────────────────────────────
onMounted(() => {
  initTheme();
  loadEnvironment();
});

onBeforeUnmount(() => {
  if (envRefreshTimer) clearTimeout(envRefreshTimer);
});
</script>

<template>
  <el-config-provider :locale="zhCn">
    <main class="studio-shell">
      <AppSidebar :env="env" :active-page="activePage" @navigate="navigateTo" />

      <section class="workspace surface">
        <AppTopbar
          :page-title="pageTitle"
          :page-description="pageDescription"
          :scanning="scanning"
          :has-deployer="hasDeployer"
          :deploying="deploying"
          @refresh="loadEnvironment"
          @deploy="handleDeploy"
        />

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
              @create-backup="handleCreateBackup"
              @open-path="openKnownPath"
              @install="handleInstallRimeIce"
              @open-backup="openBackupDir"
              @restore-backup="handleRestoreBackup"
              @delete-backup="deleteBackupEntry"
            />

            <QuickSettingsPage
              v-else-if="activePage === 'quick'"
              key="quick"
              :env="env"
              :installing-recipe="installingRecipe"
              @saved="refreshEnvironment"
              @deploy="handleDeploy"
              @install="handleInstallRimeIce"
            />

            <SchemasPage
              v-else-if="activePage === 'schemas'"
              key="schemas"
              :env="env"
              @saved="refreshEnvironment"
              @deploy="handleDeploy"
            />

            <ConfigFilesPage
              v-else-if="activePage === 'configs'"
              key="configs"
              :env="env"
              :backing-up="backingUp"
              @refresh="loadEnvironment"
              @create-backup="handleCreateBackup"
            />

            <AppearancePage
              v-else-if="activePage === 'appearance'"
              key="appearance"
              :env="env"
              @saved="refreshEnvironment"
              @deploy="handleDeploy"
            />

            <PhrasesPage
              v-else-if="activePage === 'phrases'"
              key="phrases"
              :env="env"
              @saved="refreshEnvironment"
              @deploy="handleDeploy"
            />

            <DictionariesPage
              v-else-if="activePage === 'dictionaries'"
              key="dictionaries"
              :env="env"
              @open-path="openKnownPath"
              @deploy="handleDeploy"
            />

            <BackupsPage
              v-else-if="activePage === 'backups'"
              key="backups"
              :backups="backups"
              :backing-up="backingUp"
              :restoring-backup="restoringBackup"
              :deleting-backup="deletingBackup"
              @create-backup="handleCreateBackup"
              @open-backup="openBackupDir"
              @restore-backup="handleRestoreBackup"
              @delete-backup="deleteBackupEntry"
            />

            <ConfigEditorPage
              v-else-if="activePage === 'editor'"
              key="editor"
              :env="env"
              @saved="refreshEnvironment"
              @deploy="handleDeploy"
            />

            <AboutPage v-else-if="activePage === 'about'" key="about" />
          </Transition>
        </div>

        <AppStatusbar :status="status" :is-busy="isBusy" :elapsed-seconds="elapsedSeconds" />
      </section>
    </main>
  </el-config-provider>
</template>
