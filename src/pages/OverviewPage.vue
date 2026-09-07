<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import { api } from "../api";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  Check,
  Connection,
  Delete,
  Download,
  FolderOpened,
  Open,
  Refresh,
  RefreshLeft,
  Search,
  Setting,
  Warning,
} from "@element-plus/icons-vue";
import type {
  BackupEntry,
  FileStatus,
  RimeEnvironment,
  RimeSyncConfig,
  UserDictInfo,
  UserdbEntry,
  UserdbSnapshotInfo,
} from "../types";

const props = defineProps<{
  env?: RimeEnvironment;
  backups: BackupEntry[];
  log: string;
  scanning: boolean;
  backingUp: boolean;
  restoringBackup?: string;
  deletingBackup?: string;
  installingRecipe?: string;
}>();

const emit = defineEmits<{
  createBackup: [];
  openPath: [command: "open_rime_user_dir" | "open_plum_dir" | "open_sync_dir"];
  install: [recipe: string];
  openBackup: [backup: BackupEntry];
  restoreBackup: [backup: BackupEntry];
  deleteBackup: [backup: BackupEntry];
}>();

const toolsReady = computed(() => Boolean(props.env?.git_available && props.env?.bash_available));
const hasDeployer = computed(() => Boolean(props.env?.deployer_path));
const health = computed(() => props.env?.sogou_health);
const customFiles = computed(() => props.env?.custom_files ?? []);
const hasRimeIce = computed(() => {
  return customFiles.value.some(
    (file) =>
      file.exists &&
      ["rime_ice.schema.yaml", "rime_ice.dict.yaml", "rime_ice.custom.yaml"].includes(file.name),
  );
});
const foundFiles = computed(() => customFiles.value.filter((file) => file.exists).length);
const missingFiles = computed(() => customFiles.value.length - foundFiles.value);
const readinessItems = computed(() => [
  {
    label: "部署器",
    ready: hasDeployer.value,
    detail: props.env?.deployer_path ?? "未找到 WeaselDeployer.exe",
  },
  {
    label: "Git",
    ready: Boolean(props.env?.git_available),
    detail: props.env?.git_path ?? "未找到 Git",
  },
  {
    label: "Git Bash",
    ready: Boolean(props.env?.bash_available),
    detail: props.env?.bash_path ?? "未找到 Git Bash",
  },
]);
const installRecipes = [
  {
    name: "完整雾凇",
    recipe: "iDvel/rime-ice:others/recipes/full",
    description: "安装、修复或更新雾凇拼音完整方案",
  },
  {
    name: "词库更新",
    recipe: "iDvel/rime-ice:others/recipes/all_dicts",
    description: "仅同步 rime-ice 词库资源",
  },
];

import { backupLabel, backupKindLabel, backupKindType, formatBytes, formatTime } from "../utils";

const downloadingRime = ref(false);
const downloadStatus = ref("");
const downloadingGit = ref(false);
const gitDownloadStatus = ref("");

async function openRimeDownload() {
  await openUrl("https://rime.im/download/");
}

async function openGitDownload() {
  await openUrl("https://git-scm.com/download/win");
}

async function autoDownloadAndInstall() {
  downloadingRime.value = true;
  downloadStatus.value = "正在获取最新版本信息...";

  try {
    // Step 1: Download the installer
    downloadStatus.value = "正在下载小狼毫安装包...";
    const result = await api.downloadRimeInstaller();

    if (!result.success || !result.installer_path) {
      ElMessage.warning("自动下载失败，将跳转到官网下载");
      await openRimeDownload();
      return;
    }

    // Step 2: Launch the installer
    downloadStatus.value = "正在启动安装程序...";
    await api.launchRimeInstaller(result.installer_path);
    ElMessage.success("安装程序已启动，请按提示完成安装");
    downloadStatus.value = "安装程序已启动 — 完成后返回此页面点击刷新";
  } catch (error) {
    ElMessage.warning(`自动安装失败: ${String(error)}，将跳转到官网`);
    await openRimeDownload();
  } finally {
    downloadingRime.value = false;
  }
}

async function autoDownloadGitAndInstall() {
  downloadingGit.value = true;
  gitDownloadStatus.value = "正在获取最新版本信息...";

  try {
    gitDownloadStatus.value = "正在下载 Git for Windows 安装包...";
    const result = await api.downloadGitInstaller();

    if (!result.success || !result.installer_path) {
      ElMessage.warning("自动下载 Git 失败，将跳转到官网下载");
      await openGitDownload();
      return;
    }

    gitDownloadStatus.value = "正在启动 Git 安装程序...";
    await api.launchGitInstaller(result.installer_path);
    ElMessage.success("Git 安装程序已启动，请按提示完成安装");
    gitDownloadStatus.value = "安装程序已启动 — 完成后返回此页面点击刷新";
  } catch (error) {
    ElMessage.warning(`自动安装 Git 失败: ${String(error)}，将跳转到官网`);
    await openGitDownload();
  } finally {
    downloadingGit.value = false;
  }
}

// ── Userdb & Sync Management ──
const syncConfig = ref<RimeSyncConfig>();
const loadingSyncConfig = ref(false);
const syncingUserdb = ref(false);
const showSyncDialog = ref(false);
const savingSyncConfig = ref(false);
const syncForm = reactive({
  installation_id: "",
  sync_dir: "",
});

// Snapshot entries viewer
const showEntriesDialog = ref(false);
const currentSnapshot = ref<UserdbSnapshotInfo>();
const snapshotEntries = ref<UserdbEntry[]>([]);
const loadingEntries = ref(false);
const entriesSearchQuery = ref("");
const entriesPage = ref(1);
const entriesPageSize = ref(50);
const entriesTotal = ref(0);

async function loadSyncConfig() {
  loadingSyncConfig.value = true;
  try {
    syncConfig.value = await api.getSyncConfig();
    syncForm.installation_id = syncConfig.value.installation_id ?? "";
    syncForm.sync_dir = syncConfig.value.sync_dir ?? "";
  } catch (error) {
    console.error("加载同步配置失败:", error);
  } finally {
    loadingSyncConfig.value = false;
  }
}

function openSyncSettings() {
  if (syncConfig.value) {
    syncForm.installation_id = syncConfig.value.installation_id ?? "";
    syncForm.sync_dir = syncConfig.value.sync_dir ?? "";
  }
  showSyncDialog.value = true;
}

async function saveSyncSettings() {
  savingSyncConfig.value = true;
  try {
    await api.saveSyncConfig(
      syncForm.installation_id.trim() || undefined,
      syncForm.sync_dir.trim() || undefined,
    );
    ElMessage.success("同步配置已保存");
    showSyncDialog.value = false;
    await loadSyncConfig();
  } catch (error) {
    ElMessage.error(`保存失败: ${String(error)}`);
  } finally {
    savingSyncConfig.value = false;
  }
}

async function triggerSync() {
  syncingUserdb.value = true;
  try {
    const msg = await api.syncRime();
    ElMessage.success(msg);
    await loadSyncConfig();
  } catch (error) {
    ElMessage.error(`同步失败: ${String(error)}`);
  } finally {
    syncingUserdb.value = false;
  }
}

async function inspectSnapshotEntries(snapshot: UserdbSnapshotInfo) {
  currentSnapshot.value = snapshot;
  showEntriesDialog.value = true;
  entriesPage.value = 1;
  entriesSearchQuery.value = "";
  await fetchEntries();
}

async function fetchEntries() {
  if (!currentSnapshot.value) return;
  loadingEntries.value = true;
  try {
    const offset = (entriesPage.value - 1) * entriesPageSize.value;
    const result = await api.listUserdbEntries(
      currentSnapshot.value.name,
      entriesPageSize.value,
      offset,
      entriesSearchQuery.value || undefined,
    );
    snapshotEntries.value = result.entries;
    entriesTotal.value = result.total;
  } catch (error) {
    ElMessage.error(`读取词条失败: ${String(error)}`);
  } finally {
    loadingEntries.value = false;
  }
}

function onEntriesSearch() {
  entriesPage.value = 1;
  void fetchEntries();
}

function onEntriesPageChange(page: number) {
  entriesPage.value = page;
  void fetchEntries();
}

onMounted(() => {
  void loadSyncConfig();
});

watch(
  () => props.env,
  () => {
    void loadSyncConfig();
  },
);
</script>

<template>
  <div class="overview-page">
    <!-- Onboarding: Rime not installed -->
    <div v-if="!hasDeployer && env" class="onboarding-hero">
      <div class="onboarding-body">
        <div class="onboarding-icon">
          <el-icon><Download /></el-icon>
        </div>
        <h3>未检测到小狼毫输入法</h3>
        <p>Rime Studio 是小狼毫的配置工作台，需要先安装输入法本体才能使用完整功能。</p>
        <div class="onboarding-steps">
          <div class="onboard-step">
            <span class="step-num">1</span>
            <span>下载小狼毫安装包</span>
          </div>
          <div class="onboard-step">
            <span class="step-num">2</span>
            <span>运行安装程序，按提示完成安装</span>
          </div>
          <div class="onboard-step">
            <span class="step-num">3</span>
            <span>安装后回到此页面，点击「重新部署」激活</span>
          </div>
        </div>
        <div class="onboarding-actions">
          <el-button
            type="primary"
            size="large"
            :icon="Download"
            :loading="downloadingRime"
            @click="autoDownloadAndInstall"
          >
            {{ downloadingRime ? downloadStatus : "自动下载安装" }}
          </el-button>
          <el-button size="large" :disabled="downloadingRime" @click="openRimeDownload">
            手动去官网下载
          </el-button>
        </div>
        <p class="onboarding-hint">自动下载从 GitHub 获取最新版本。手动下载可自行选择版本。</p>
      </div>
    </div>

    <!-- Loading skeleton -->
    <el-skeleton v-if="!env" :rows="1" animated style="margin-bottom: 14px" />
    <!-- Rime installed: show status bar -->
    <section v-else-if="hasDeployer" class="compact-status">
      <div class="status-cell primary">
        <el-icon><Setting /></el-icon>
        <span>方案</span>
        <strong>{{ env?.active_schema ?? "未知" }}</strong>
      </div>
      <div class="status-cell">
        <span>主题</span>
        <strong>{{ env?.theme_name ?? "未知" }}</strong>
      </div>
      <div class="status-cell">
        <span>候选</span>
        <strong>{{ env?.page_size ?? "?" }} 项</strong>
      </div>
      <div class="status-cell">
        <span>字体</span>
        <strong>{{ env?.font_point ?? "?" }} / {{ env?.label_font_point ?? "?" }}</strong>
      </div>
      <div class="status-cell">
        <span>文件</span>
        <strong>{{ foundFiles }} / {{ customFiles.length }}</strong>
      </div>
      <div class="status-cell">
        <span>工具</span>
        <el-tag :type="toolsReady ? 'success' : 'warning'" effect="light" size="small">
          {{ toolsReady ? "就绪" : "缺失" }}
        </el-tag>
      </div>
    </section>

    <section class="content-grid overview-compact-grid">
      <section class="main-column">
        <el-card class="panel overview-panel compact-panel" shadow="never">
          <template #header>
            <div class="panel-title">
              <span>配置状态</span>
              <el-tag :type="missingFiles ? 'warning' : 'success'" effect="light">
                {{ missingFiles ? `${missingFiles} 个缺失` : "文件齐全" }}
              </el-tag>
            </div>
          </template>

          <el-skeleton v-if="!env && scanning" :rows="5" animated />
          <div v-else class="status-table">
            <div v-for="item in readinessItems" :key="item.label" class="status-row">
              <el-icon :class="item.ready ? 'ok-icon' : 'warn-icon'">
                <Check v-if="item.ready" />
                <Warning v-else />
              </el-icon>
              <span>{{ item.label }}</span>
              <code>{{ item.detail }}</code>
            </div>
          </div>
        </el-card>

        <el-card class="panel file-panel compact-panel" shadow="never">
          <template #header>
            <div class="panel-title">
              <span>受管理文件</span>
              <span class="panel-caption">{{ env?.build_dir ?? "build 目录待扫描" }}</span>
            </div>
          </template>
          <el-table :data="customFiles" stripe max-height="280">
            <el-table-column label="文件" min-width="280">
              <template #default="{ row }: { row: FileStatus }">
                <strong class="file-name">{{ row.name }}</strong>
                <span class="mono-path file-path">{{ row.path }}</span>
              </template>
            </el-table-column>
            <el-table-column label="状态" width="130">
              <template #default="{ row }: { row: FileStatus }">
                <el-tag :type="row.exists ? 'success' : 'warning'" effect="light">
                  {{ row.exists ? "存在" : "缺失" }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column label="大小" width="140">
              <template #default="{ row }: { row: FileStatus }">
                {{ formatBytes(row.size) }}
              </template>
            </el-table-column>
          </el-table>
        </el-card>

        <el-card class="panel compact-panel" shadow="never">
          <template #header>
            <div class="panel-title">
              <span>用户词库与同步</span>
              <div class="panel-header-actions">
                <el-button
                  type="primary"
                  size="small"
                  :icon="Refresh"
                  :loading="syncingUserdb"
                  @click="triggerSync"
                >
                  立即同步
                </el-button>
                <el-button size="small" :icon="Setting" @click="openSyncSettings">
                  同步设置
                </el-button>
                <el-button link type="primary" @click="emit('openPath', 'open_sync_dir')">
                  打开目录
                </el-button>
              </div>
            </div>
          </template>

          <!-- Sync Meta Summary -->
          <div class="sync-summary-bar">
            <div class="sync-summary-item">
              <span class="sync-summary-label">设备标识 (ID)</span>
              <el-tag size="small" effect="plain">{{
                syncConfig?.installation_id || "未设置"
              }}</el-tag>
            </div>
            <div class="sync-summary-item">
              <span class="sync-summary-label">同步路径</span>
              <code class="sync-summary-path" :title="syncConfig?.sync_dir">{{
                syncConfig?.sync_dir || "默认 sync/"
              }}</code>
            </div>
            <div class="sync-summary-item">
              <span class="sync-summary-label">上次同步</span>
              <span class="sync-summary-time">{{
                formatTime(syncConfig?.last_sync_time) || "未同步"
              }}</span>
            </div>
          </div>

          <!-- Section 1: Active User DBs -->
          <div class="sync-section-title">
            <span>活动词库 (*.userdb)</span>
            <small>当前输入法正在读写的动态词库</small>
          </div>
          <el-table
            :data="env?.user_dicts ?? []"
            stripe
            max-height="180"
            empty-text="暂无活动词库"
            size="small"
          >
            <el-table-column label="词库名称" min-width="180">
              <template #default="{ row }: { row: UserDictInfo }">
                <strong>{{ row.name }}</strong>
                <span class="mono-path file-path">{{ row.path }}</span>
              </template>
            </el-table-column>
            <el-table-column label="大小" width="110">
              <template #default="{ row }: { row: UserDictInfo }">
                {{ formatBytes(row.size_bytes) }}
              </template>
            </el-table-column>
            <el-table-column label="修改时间" width="160">
              <template #default="{ row }: { row: UserDictInfo }">
                {{ formatTime(row.modified) }}
              </template>
            </el-table-column>
          </el-table>

          <!-- Section 2: Sync Snapshot Files -->
          <div class="sync-section-title" style="margin-top: 14px">
            <span>同步快照文件 (*.userdb.txt)</span>
            <small>同步生成的文本快照，用于多设备合并与查看</small>
          </div>
          <el-table
            :data="syncConfig?.snapshot_files ?? []"
            stripe
            max-height="200"
            empty-text="尚未进行同步或无快照文件"
            size="small"
          >
            <el-table-column label="快照文件" min-width="180">
              <template #default="{ row }: { row: UserdbSnapshotInfo }">
                <strong>{{ row.name }}</strong>
              </template>
            </el-table-column>
            <el-table-column label="条目估算" width="110">
              <template #default="{ row }: { row: UserdbSnapshotInfo }">
                <el-tag size="small" type="info">{{ row.entry_count.toLocaleString() }} 条</el-tag>
              </template>
            </el-table-column>
            <el-table-column label="大小" width="100">
              <template #default="{ row }: { row: UserdbSnapshotInfo }">
                {{ formatBytes(row.file_size) }}
              </template>
            </el-table-column>
            <el-table-column label="更新时间" width="150">
              <template #default="{ row }: { row: UserdbSnapshotInfo }">
                {{ formatTime(row.modified) }}
              </template>
            </el-table-column>
            <el-table-column label="操作" width="90" fixed="right">
              <template #default="{ row }: { row: UserdbSnapshotInfo }">
                <el-button link type="primary" size="small" @click="inspectSnapshotEntries(row)">
                  浏览词条
                </el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-card>

        <el-card v-if="log" class="log-panel" shadow="never">
          <template #header>
            <span>安装日志</span>
          </template>
          <pre>{{ log }}</pre>
        </el-card>
      </section>

      <aside class="side-column">
        <el-card class="panel quick-panel compact-panel" shadow="never">
          <template #header>
            <span>快捷操作</span>
          </template>
          <div class="quick-actions">
            <el-button
              type="primary"
              plain
              :loading="backingUp"
              :icon="FolderOpened"
              @click="emit('createBackup')"
            >
              创建备份
            </el-button>
            <el-button :icon="Open" @click="emit('openPath', 'open_rime_user_dir')">
              打开用户目录
            </el-button>
          </div>
        </el-card>

        <el-card class="panel action-panel compact-panel" shadow="never">
          <template #header>
            <div class="panel-title">
              <span>{{ hasRimeIce ? "雾凇维护" : "安装 rime-ice" }}</span>
              <el-tag
                :type="!toolsReady ? 'warning' : hasRimeIce ? 'success' : 'info'"
                effect="light"
              >
                {{ !toolsReady ? "工具缺失" : hasRimeIce ? "已安装" : "可安装" }}
              </el-tag>
            </div>
          </template>

          <div v-if="!toolsReady" class="tool-install-callout">
            <div>
              <strong>需要先安装 Git for Windows</strong>
              <span>安装 rime-ice 需要 Git 和 Git Bash。完成安装后回到这里点击刷新。</span>
            </div>
            <div class="tool-install-actions">
              <el-button
                type="primary"
                :icon="Download"
                :loading="downloadingGit"
                @click="autoDownloadGitAndInstall"
              >
                {{ downloadingGit ? gitDownloadStatus : "自动下载安装 Git" }}
              </el-button>
              <el-button :disabled="downloadingGit" @click="openGitDownload"> 手动下载 </el-button>
            </div>
          </div>

          <div class="recipe-list">
            <button
              v-for="recipe in installRecipes"
              :key="recipe.recipe"
              class="recipe-card"
              :disabled="!toolsReady || Boolean(installingRecipe)"
              @click="emit('install', recipe.recipe)"
            >
              <span>
                <strong>
                  {{ hasRimeIce && recipe.recipe.includes("full") ? "更新/修复雾凇" : recipe.name }}
                </strong>
                <small>{{ recipe.description }}</small>
              </span>
              <el-icon v-if="installingRecipe === recipe.recipe" class="is-loading">
                <Refresh />
              </el-icon>
              <el-icon v-else><Download /></el-icon>
            </button>
          </div>
        </el-card>

        <el-card class="panel backup-panel compact-panel" shadow="never">
          <template #header>
            <div class="panel-title">
              <span>最近备份</span>
              <el-tag effect="light">{{ backups.length }}</el-tag>
            </div>
          </template>
          <el-empty v-if="!backups.length" :image-size="64" description="还没有备份" />
          <div v-else class="backup-list">
            <div v-for="backup in backups.slice(0, 4)" :key="backup.path" class="backup-item">
              <div class="backup-main">
                <span>
                  <strong>
                    <el-tag size="small" effect="light" :type="backupKindType(backup.kind)">
                      {{ backupKindLabel(backup.kind) }}
                    </el-tag>
                    {{ backupLabel(backup) }}
                  </strong>
                  <small>{{ formatTime(backup.modified) }} · {{ backup.files }} 个文件</small>
                </span>
                <div class="backup-actions">
                  <el-button link type="primary" :icon="Open" @click="emit('openBackup', backup)">
                    打开
                  </el-button>
                  <el-button
                    link
                    type="warning"
                    :icon="RefreshLeft"
                    :loading="restoringBackup === backup.name"
                    @click="emit('restoreBackup', backup)"
                  >
                    恢复
                  </el-button>
                  <el-button
                    link
                    type="danger"
                    :icon="Delete"
                    :loading="deletingBackup === backup.name"
                    @click="emit('deleteBackup', backup)"
                  >
                    删除
                  </el-button>
                </div>
              </div>
              <code>{{ backup.path }}</code>
            </div>
            <div v-if="backups.length > 4" class="backup-more">
              还有 {{ backups.length - 4 }} 个备份，到「备份管理」页查看全部
            </div>
          </div>
        </el-card>

        <el-card class="panel dictionary-panel compact-panel" shadow="never">
          <template #header>
            <span>词库健康</span>
          </template>
          <div v-if="health" class="health-list">
            <div>
              <span>条目</span>
              <strong>{{ health.entries.toLocaleString() }}</strong>
            </div>
            <div>
              <span>重复行</span>
              <strong :class="health.duplicate_exact_lines ? 'warn-text' : ''">{{
                health.duplicate_exact_lines.toLocaleString()
              }}</strong>
            </div>
            <div>
              <span>长低权重项</span>
              <strong :class="health.long_low_weight_entries ? 'warn-text' : ''">{{
                health.long_low_weight_entries.toLocaleString()
              }}</strong>
            </div>
          </div>
          <p v-else class="helper-text">暂无词库健康数据，安装雾凇后可查看。</p>
        </el-card>

        <el-card class="panel path-panel compact-panel" shadow="never">
          <template #header>
            <div class="panel-title">
              <span>Plum 目录</span>
              <el-button
                link
                type="primary"
                :icon="Open"
                @click="emit('openPath', 'open_plum_dir')"
              >
                打开
              </el-button>
            </div>
          </template>
          <div class="path-chip">
            <el-icon><Connection /></el-icon>
            <span>{{ env?.plum_dir ?? "未知" }}</span>
          </div>
        </el-card>
      </aside>
    </section>

    <!-- Sync Settings Dialog -->
    <el-dialog v-model="showSyncDialog" title="Rime 同步配置" width="520px">
      <el-form label-position="top">
        <el-form-item label="设备标识 (installation_id)">
          <el-input
            v-model="syncForm.installation_id"
            placeholder="例如: weasel-pc, laptop-home"
            maxlength="64"
          />
          <div class="field-tip">
            用于区分多台设备的词库快照，快照文件将存放于
            <code>&lt;sync_dir&gt;/&lt;id&gt;</code> 目录下。
          </div>
        </el-form-item>
        <el-form-item label="自定义同步目录 (sync_dir)">
          <el-input
            v-model="syncForm.sync_dir"
            placeholder="留空则使用默认用户目录下的 sync/ 文件夹"
          />
          <div class="field-tip">
            可设置为坚果云、OneDrive、Syncthing 等网盘的同步文件夹以实现多设备自动同步。
          </div>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showSyncDialog = false">取消</el-button>
        <el-button type="primary" :loading="savingSyncConfig" @click="saveSyncSettings">
          保存配置
        </el-button>
      </template>
    </el-dialog>

    <!-- Userdb Snapshot Entries Viewer Dialog -->
    <el-dialog
      v-model="showEntriesDialog"
      :title="`词条浏览 - ${currentSnapshot?.name ?? ''}`"
      width="780px"
      class="userdb-entries-dialog"
    >
      <div class="entries-viewer-header">
        <el-input
          v-model="entriesSearchQuery"
          placeholder="搜索词条或拼音编码..."
          clearable
          :prefix-icon="Search"
          style="max-width: 320px"
          @keyup.enter="onEntriesSearch"
          @clear="onEntriesSearch"
        />
        <el-button type="primary" plain :icon="Search" @click="onEntriesSearch"> 搜索 </el-button>
        <span class="entries-total-badge"> 共 {{ entriesTotal.toLocaleString() }} 条记录 </span>
      </div>

      <el-table
        v-loading="loadingEntries"
        :data="snapshotEntries"
        stripe
        height="380"
        empty-text="没有找到匹配词条"
        size="small"
      >
        <el-table-column prop="word" label="词条 / 文字" min-width="160" />
        <el-table-column prop="code" label="拼音编码" min-width="160">
          <template #default="{ row }: { row: UserdbEntry }">
            <code>{{ row.code }}</code>
          </template>
        </el-table-column>
        <el-table-column prop="count" label="词频 / 权重" width="130">
          <template #default="{ row }: { row: UserdbEntry }">
            <el-tag size="small" type="info">{{ row.count }}</el-tag>
          </template>
        </el-table-column>
      </el-table>

      <div class="entries-viewer-footer">
        <el-pagination
          background
          layout="prev, pager, next, total"
          :current-page="entriesPage"
          :page-size="entriesPageSize"
          :total="entriesTotal"
          @current-change="onEntriesPageChange"
        />
      </div>
    </el-dialog>
  </div>
</template>
