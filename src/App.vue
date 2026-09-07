<script setup lang="ts">
import { computed, defineAsyncComponent, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { useRoute, useRouter } from "vue-router";
import zhCn from "element-plus/es/locale/lang/zh-cn";
import { ElMessageBox } from "element-plus";
import AppSidebar from "./components/layout/AppSidebar.vue";
import AppTopbar from "./components/layout/AppTopbar.vue";
import AppStatusbar from "./components/layout/AppStatusbar.vue";
import { useTheme } from "./composables/useTheme";
import { useStudioStore } from "./stores/studio";
import { isPageKey, type PageKey } from "./navigation";

// ── Lazy page components ──────────────────────────
const AboutPage = defineAsyncComponent(() => import("./pages/AboutPage.vue"));
const AppearancePage = defineAsyncComponent(() => import("./pages/AppearancePage.vue"));
const BackupsPage = defineAsyncComponent(() => import("./pages/BackupsPage.vue"));
const ConfigEditorPage = defineAsyncComponent(() => import("./pages/ConfigEditorPage.vue"));
const DictionariesPage = defineAsyncComponent(() => import("./pages/DictionariesPage.vue"));
const OverviewPage = defineAsyncComponent(() => import("./pages/OverviewPage.vue"));
const PhrasesPage = defineAsyncComponent(() => import("./pages/PhrasesPage.vue"));
const QuickSettingsPage = defineAsyncComponent(() => import("./pages/QuickSettingsPage.vue"));
const SchemasPage = defineAsyncComponent(() => import("./pages/SchemasPage.vue"));

const { initTheme } = useTheme();
const studio = useStudioStore();
const {
  env,
  scanning,
  status,
  deploying,
  installingRecipe,
  log,
  backups,
  backingUp,
  restoringBackup,
  deletingBackup,
  hasDeployer,
} = storeToRefs(studio);

const route = useRoute();
const router = useRouter();
const editorDirty = ref(false);

const activePage = computed<PageKey>(() => {
  const name = String(route.name ?? "overview");
  return isPageKey(name) ? name : "overview";
});

async function navigateTo(key: string) {
  if (key === "configs") key = "editor";
  if (!isPageKey(key) || key === activePage.value) return;

  if (activePage.value === "editor" && editorDirty.value) {
    try {
      await ElMessageBox.confirm("配置编辑器中有未保存的修改，确定要离开吗？", "未保存的修改", {
        confirmButtonText: "放弃修改并离开",
        cancelButtonText: "继续编辑",
        type: "warning",
      });
    } catch {
      return;
    }
    editorDirty.value = false;
  }

  await router.push({ name: key });
}
const elapsedSeconds = ref(0);
let elapsedTimer: ReturnType<typeof setInterval> | undefined;

const pageTitle = computed(() => {
  const titles: Record<PageKey, string> = {
    overview: "Rime 配置控制台",
    quick: "快速设置",
    schemas: "方案管理",
    appearance: "主题配置",
    phrases: "短语管理",
    dictionaries: "词库管理",
    backups: "备份管理",
    editor: "配置中心",
    about: "关于",
  };
  return titles[activePage.value];
});

const pageDescription = computed(() => {
  const descriptions: Record<PageKey, string> = {
    overview: "管理方案、外观、词库与部署状态。",
    quick: "集中调整当前方案、候选数量、按键绑定，以及已安装的雾凇组件与 Lua 扩展。",
    schemas: "查看、启用、复制本机方案，浏览并一键安装社区方案与双拼键位图。",
    appearance: "调整小狼毫候选窗主题、字号、边距、颜色并测试打字效果。",
    phrases: "编辑自定义短语，支持添加、搜索、多格式导入与批量管理。",
    dictionaries: "浏览和管理 Rime 词库文件，查看条目统计、健康状态与在线导入。",
    backups: "查看、打开和恢复 Rime Studio 创建的配置备份与整包导出。",
    editor: "集中查看、定位、编辑和校验 Rime 关键配置文件。",
    about: "关于 Rime Studio 与相关开源项目。",
  };
  return descriptions[activePage.value];
});

let envRefreshTimer: ReturnType<typeof setTimeout> | undefined;

function refreshEnvironment() {
  if (envRefreshTimer) clearTimeout(envRefreshTimer);
  envRefreshTimer = setTimeout(() => {
    void studio.loadEnvironment();
  }, 300);
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
  void studio.loadEnvironment();
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
          @refresh="studio.loadEnvironment"
          @deploy="studio.deploy"
        />

        <div class="page-container">
          <Transition name="page" mode="out-in">
            <KeepAlive>
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
                @create-backup="studio.createManualBackup"
                @open-path="studio.openKnownPath"
                @install="studio.installRimeIce"
                @open-backup="studio.openBackupDir"
                @restore-backup="studio.restoreBackup"
                @delete-backup="studio.deleteBackupEntry"
              />

              <QuickSettingsPage
                v-else-if="activePage === 'quick'"
                key="quick"
                :env="env"
                :installing-recipe="installingRecipe"
                @saved="refreshEnvironment"
                @deploy="studio.deploy"
                @install="studio.installRimeIce"
              />

              <SchemasPage
                v-else-if="activePage === 'schemas'"
                key="schemas"
                :env="env"
                @saved="refreshEnvironment"
                @deploy="studio.deploy"
                @install="studio.installRimeIce"
              />

              <AppearancePage
                v-else-if="activePage === 'appearance'"
                key="appearance"
                :env="env"
                @saved="refreshEnvironment"
                @deploy="studio.deploy"
              />

              <PhrasesPage
                v-else-if="activePage === 'phrases'"
                key="phrases"
                :env="env"
                @saved="refreshEnvironment"
                @deploy="studio.deploy"
              />

              <DictionariesPage
                v-else-if="activePage === 'dictionaries'"
                key="dictionaries"
                :env="env"
                @open-path="studio.openKnownPath"
                @deploy="studio.deploy"
              />

              <BackupsPage
                v-else-if="activePage === 'backups'"
                key="backups"
                :backups="backups"
                :backing-up="backingUp"
                :restoring-backup="restoringBackup"
                :deleting-backup="deletingBackup"
                @create-backup="studio.createManualBackup"
                @open-backup="studio.openBackupDir"
                @restore-backup="studio.restoreBackup"
                @delete-backup="studio.deleteBackupEntry"
              />

              <ConfigEditorPage
                v-else-if="activePage === 'editor'"
                key="editor"
                :env="env"
                @saved="refreshEnvironment"
                @deploy="studio.deploy"
                @dirty-change="editorDirty = $event"
              />

              <AboutPage v-else-if="activePage === 'about'" key="about" />
            </KeepAlive>
          </Transition>
        </div>

        <AppStatusbar
          :status="status"
          :is-busy="isBusy"
          :elapsed-seconds="elapsedSeconds"
          :deploying="deploying"
          @cancel-deploy="studio.cancelDeploy"
        />
      </section>
    </main>
  </el-config-provider>
</template>
