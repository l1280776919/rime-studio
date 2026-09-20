<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { ElMessage } from "element-plus";
import { api } from "../api";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  Close,
  Download,
  FolderOpened,
  MagicStick,
  Refresh,
  Setting,
  UploadFilled,
} from "@element-plus/icons-vue";
import type {
  BackupEntry,
  FileStatus,
  QuickSettingsConfig,
  RimeEnvironment,
  RimeIceSettings,
  RimeSyncConfig,
  SchemaInfo,
  UserDictInfo,
  UserdbEntry,
  UserdbSnapshotInfo,
} from "../types";
import { backupLabel, formatBytes, formatTime } from "../utils";

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
  deploy: [];
  saved: [];
  navigate: [key: string];
}>();

const hasDeployer = computed(() => Boolean(props.env?.deployer_path));
const customFiles = computed(() => props.env?.custom_files ?? []);
const hasRimeIce = computed(() => {
  return customFiles.value.some(
    (file) =>
      file.exists &&
      ["rime_ice.schema.yaml", "rime_ice.dict.yaml", "rime_ice.custom.yaml"].includes(file.name),
  );
});
const foundFiles = computed(() => customFiles.value.filter((file) => file.exists).length);

// ── Quick Settings & Ice Settings State in Cockpit ──
const loadingQuick = ref(false);
const deployingCockpit = ref(false);
const schemas = ref<SchemaInfo[]>([]);

const quickConfig = reactive<QuickSettingsConfig>({
  schema_id: "",
  page_size: 7,
  switch_key: "shift",
  paging_keys: "comma_period",
  navigation_keys: "up_down",
  horizontal: true,
  inline_preedit: true,
});

const iceSettings = reactive<RimeIceSettings>({
  emoji: true,
  traditionalization: false,
  ascii_punct: false,
  full_shape: false,
  search_single_char: false,
  fuzzy_pinyin: false,
  traditional_preset: "s2t.json",
  fuzzy_pairs: [],
});

const activeSchema = computed(() => {
  return (
    schemas.value.find((s) => s.id === quickConfig.schema_id) ??
    schemas.value.find((s) => s.is_active)
  );
});

async function loadCockpitSettings() {
  loadingQuick.value = true;
  try {
    const [cfg, schemaList, iceCfg] = await Promise.all([
      api.getQuickSettings(),
      api.listSchemas(),
      api.getRimeIceSettings().catch(() => null),
    ]);
    if (cfg) Object.assign(quickConfig, cfg);
    if (schemaList) schemas.value = schemaList;
    if (iceCfg) Object.assign(iceSettings, iceCfg);
  } catch (err) {
    console.error("加载指挥舱配置失败:", err);
  } finally {
    loadingQuick.value = false;
  }
}

async function quickSaveAndDeploy() {
  deployingCockpit.value = true;
  try {
    await Promise.all([
      api.saveQuickSettings({ ...quickConfig }),
      hasRimeIce.value ? api.saveRimeIceSettings({ ...iceSettings }) : Promise.resolve(null),
    ]);
    emit("saved");
    emit("deploy");
    ElMessage.success("配置已保存，已发送小狼毫重新部署指令！");
  } catch (err) {
    ElMessage.error(`保存失败: ${String(err)}`);
  } finally {
    deployingCockpit.value = false;
  }
}

async function onTileToggle() {
  try {
    if (hasRimeIce.value) {
      await api.saveRimeIceSettings({ ...iceSettings });
    }
    await api.saveQuickSettings({ ...quickConfig });
    emit("saved");
    ElMessage.success("特性已更新 (点击「一键部署」可立即对小狼毫生效)");
  } catch (err) {
    ElMessage.error(`更新特性失败: ${String(err)}`);
  }
}

// ── Interactive Live Typing & Candidate Stage ──
const typingQuery = ref("");
const mockDict: Record<string, string[]> = {
  ni: ["你", "泥", "拟", "呢", "尼"],
  hao: ["好", "号", "毫", "豪", "浩"],
  nihao: ["你好", "拟好", "泥壕"],
  rime: ["Rime", "中州韵", "小狼毫", "鼠须管"],
  wusong: ["雾凇", "雾凇拼音", "物颂"],
  zhongwen: ["中文", "众文", "中温"],
  ceshi: ["测试", "侧室", "策士"],
  dazi: ["打字", "搭子", "大字"],
  weasel: ["小狼毫", "黄鼠狼", "输入法"],
};

const stageCandidates = computed(() => {
  const q = typingQuery.value.trim().toLowerCase();
  const count = Math.max(3, Math.min(quickConfig.page_size || 7, 12));
  if (!q) {
    const list = [
      iceSettings.emoji ? "我们 😊" : "我们",
      "文明",
      "蜗牛",
      "握手",
      "卧室",
      "莴苣",
      "沃土",
      "乌鸦",
      "舞会",
      "污染",
    ];
    return list.slice(0, count);
  }
  const found = mockDict[q];
  if (found) {
    const res = [...found];
    if (iceSettings.emoji && res.length > 0) {
      res[0] = `${res[0]} ✨`;
    }
    return res.slice(0, count);
  }
  return [q, `${q}的`, `${q}了`, `${q}在`, `${q}是`].slice(0, count);
});

function setSampleQuery(q: string) {
  typingQuery.value = q;
}

// ── Bottom Telemetry & Drawer ──
const showTelemetryDrawer = ref(false);
const telemetryTab = ref<"files" | "userdb" | "backups">("files");

// ── Userdb & Sync Management ──
const syncConfig = ref<RimeSyncConfig>();
const syncingUserdb = ref(false);
const showSyncDialog = ref(false);
const savingSyncConfig = ref(false);
const syncForm = reactive({ installation_id: "", sync_dir: "" });

const showEntriesDialog = ref(false);
const currentSnapshot = ref<UserdbSnapshotInfo>();
const snapshotEntries = ref<UserdbEntry[]>([]);
const loadingEntries = ref(false);
const entriesSearchQuery = ref("");
const entriesPage = ref(1);
const entriesPageSize = ref(50);
const entriesTotal = ref(0);

async function loadSyncConfig() {
  try {
    syncConfig.value = await api.getSyncConfig();
    syncForm.installation_id = syncConfig.value.installation_id ?? "";
    syncForm.sync_dir = syncConfig.value.sync_dir ?? "";
  } catch (error) {
    console.error("加载同步配置失败:", error);
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

// ── Onboarding / Git Install ──
const downloadingRime = ref(false);
const downloadStatus = ref("");
async function autoDownloadAndInstall() {
  downloadingRime.value = true;
  downloadStatus.value = "正在获取安装包...";
  try {
    const result = await api.downloadRimeInstaller();
    if (!result.success || !result.installer_path) {
      await openUrl("https://rime.im/download/");
      return;
    }
    await api.launchRimeInstaller(result.installer_path);
    ElMessage.success("安装程序已启动");
  } catch (error) {
    ElMessage.warning(`安装启动失败: ${String(error)}`);
  } finally {
    downloadingRime.value = false;
  }
}

onMounted(() => {
  void loadCockpitSettings();
  void loadSyncConfig();
});

watch(
  () => props.env,
  () => {
    void loadCockpitSettings();
    void loadSyncConfig();
  },
);
</script>

<template>
  <div class="cockpit-container custom-scrollbar">
    <!-- 0. Onboarding Hero (Only when Weasel not detected) -->
    <div v-if="!hasDeployer && env" class="onboarding-hero">
      <div class="onboarding-body">
        <div class="onboarding-icon">
          <el-icon><Download /></el-icon>
        </div>
        <h3>未检测到小狼毫输入法 (Weasel)</h3>
        <p>Rime Studio 是小狼毫的桌面配置工作台，需要先安装输入法本体才能使用完整功能。</p>
        <div class="onboarding-actions">
          <el-button
            type="primary"
            size="large"
            :icon="Download"
            :loading="downloadingRime"
            @click="autoDownloadAndInstall"
          >
            {{ downloadingRime ? downloadStatus : "自动下载并运行安装程序" }}
          </el-button>
          <el-button size="large" @click="openUrl('https://rime.im/download/')">
            官网手动下载
          </el-button>
        </div>
      </div>
    </div>

    <template v-else-if="hasDeployer">
      <!-- 1. Master Cockpit Hero Banner (输入法总控指挥舱) -->
      <section class="cockpit-hero">
        <div class="cockpit-hero-left">
          <div class="cockpit-avatar">
            <span>❄️</span>
          </div>
          <div class="cockpit-hero-info">
            <div class="cockpit-title-row">
              <h2 class="cockpit-schema-name">
                {{ activeSchema?.name || env?.active_schema || "雾凇拼音" }}
              </h2>
              <span class="cockpit-status-pill">
                <span class="pulse-dot online"></span>
                Weasel 引擎就绪
              </span>
            </div>
            <div class="cockpit-meta-row">
              <span>{{ quickConfig.schema_id || "rime_ice" }}</span>
              <span>·</span>
              <span
                >{{ quickConfig.page_size }} 项候选 ({{
                  quickConfig.horizontal ? "横排" : "竖排"
                }})</span
              >
              <span>·</span>
              <span>主题: {{ env?.theme_name ?? "明快蓝调" }}</span>
              <span>·</span>
              <span>配置完整: {{ foundFiles }} / {{ customFiles.length }}</span>
            </div>
          </div>
        </div>

        <div class="cockpit-hero-actions">
          <el-button
            :icon="Refresh"
            :loading="scanning"
            circle
            title="重新扫描环境"
            @click="loadCockpitSettings"
          />
          <el-button
            :icon="FolderOpened"
            circle
            title="打开当前 Rime 用户目录"
            @click="emit('openPath', 'open_rime_user_dir')"
          />
          <el-button
            :icon="MagicStick"
            circle
            title="创建配置快照备份"
            :loading="backingUp"
            @click="emit('createBackup')"
          />
          <button
            type="button"
            class="deploy-giant-btn el-button el-button--primary"
            :disabled="deployingCockpit"
            @click="quickSaveAndDeploy"
          >
            <el-icon class="mr-1"><UploadFilled /></el-icon>
            {{ deployingCockpit ? "正在部署..." : "一键部署生效" }}
          </button>
        </div>
      </section>

      <!-- 2. Interactive Live Candidate & Typing Stage (中央打字与候选窗画板) -->
      <section class="cockpit-stage">
        <div class="stage-top-bar">
          <div class="stage-title">
            <span>🎯</span>
            <span>实时候选窗与输入手感仿真 (Live Interactive Stage)</span>
          </div>
          <div class="sample-chips-row">
            <span class="sample-chips-label">快速试词：</span>
            <button
              v-for="sample in ['nihao', 'rime', 'wusong', 'ceshi']"
              :key="sample"
              type="button"
              class="sample-chip"
              @click="setSampleQuery(sample)"
            >
              {{ sample }}
            </button>
          </div>
        </div>

        <!-- Interactive Typing Input -->
        <div class="stage-interactive-input-wrap">
          <input
            v-model="typingQuery"
            type="text"
            class="stage-interactive-input"
            placeholder="在此键入拼音测试打字体验 (例如输入: nihao, rime, wusong, ceshi)..."
          />
          <button
            v-if="typingQuery"
            type="button"
            class="stage-clear-btn"
            @click="typingQuery = ''"
          >
            <el-icon><Close /></el-icon>
          </button>
          <span v-else class="stage-input-hint">键入即刻联动下方候选窗</span>
        </div>

        <!-- Floating Realistic Candidate Window Box -->
        <div class="stage-weasel-arena">
          <div class="weasel-realistic-box">
            <div v-if="!quickConfig.inline_preedit" class="weasel-preedit">
              {{ typingQuery || "wo'men" }}
            </div>
            <div class="weasel-cand-list" :class="{ vertical: !quickConfig.horizontal }">
              <span
                v-for="(word, idx) in stageCandidates"
                :key="word"
                class="weasel-cand-item"
                :class="{ active: idx === 0 }"
              >
                <em>{{ idx + 1 }}.</em> {{ word }}
              </span>
            </div>
          </div>
        </div>

        <!-- Direct Stage Knobs Bar (实时手感微调) -->
        <div class="stage-controls-strip">
          <div class="stage-knob-group">
            <!-- Knob 1: Page size -->
            <div class="stage-knob">
              <span class="knob-label">候选词数:</span>
              <div class="knob-pill-group">
                <button
                  v-for="n in [5, 7, 9, 10]"
                  :key="n"
                  type="button"
                  class="knob-pill-btn"
                  :class="{ active: quickConfig.page_size === n }"
                  @click="
                    quickConfig.page_size = n;
                    onTileToggle();
                  "
                >
                  {{ n }}
                </button>
              </div>
            </div>

            <!-- Knob 2: Orientation -->
            <div class="stage-knob">
              <span class="knob-label">排列方向:</span>
              <div class="knob-pill-group">
                <button
                  type="button"
                  class="knob-pill-btn"
                  :class="{ active: quickConfig.horizontal }"
                  @click="
                    quickConfig.horizontal = true;
                    onTileToggle();
                  "
                >
                  横排
                </button>
                <button
                  type="button"
                  class="knob-pill-btn"
                  :class="{ active: !quickConfig.horizontal }"
                  @click="
                    quickConfig.horizontal = false;
                    onTileToggle();
                  "
                >
                  竖排
                </button>
              </div>
            </div>

            <!-- Knob 3: Preedit -->
            <div class="stage-knob">
              <span class="knob-label">编码拼音:</span>
              <div class="knob-pill-group">
                <button
                  type="button"
                  class="knob-pill-btn"
                  :class="{ active: quickConfig.inline_preedit }"
                  @click="
                    quickConfig.inline_preedit = true;
                    onTileToggle();
                  "
                >
                  内嵌光标
                </button>
                <button
                  type="button"
                  class="knob-pill-btn"
                  :class="{ active: !quickConfig.inline_preedit }"
                  @click="
                    quickConfig.inline_preedit = false;
                    onTileToggle();
                  "
                >
                  窗口顶端
                </button>
              </div>
            </div>
          </div>

          <div class="stage-actions-right">
            <el-select
              v-model="quickConfig.schema_id"
              size="small"
              placeholder="切换输入方案"
              style="width: 170px"
              @change="onTileToggle"
            >
              <el-option
                v-for="schema in schemas"
                :key="schema.id"
                :label="schema.name"
                :value="schema.id"
              />
            </el-select>
            <el-button size="small" @click="emit('navigate', 'quick')"> 完整参数 → </el-button>
          </div>
        </div>
      </section>

      <!-- 3. Modern Bento Feature Tiles Matrix (现代化特性磁贴矩阵) -->
      <section class="bento-section-wrap">
        <div class="bento-header-line">
          <span class="bento-section-kicker"> 输入法特性磁贴 (Feature Tiles) </span>
          <span class="bento-more-link" @click="emit('navigate', 'quick')">
            查看更多组件设置 →
          </span>
        </div>

        <div class="bento-matrix">
          <!-- Tile 1: Emoji -->
          <div class="bento-tile">
            <div class="bento-tile-main">
              <span class="bento-tile-icon">😃</span>
              <div class="bento-tile-texts">
                <span class="bento-tile-title">Emoji 表情联想</span>
                <span class="bento-tile-desc">键入拼音匹配趣味表情符号输出</span>
              </div>
            </div>
            <div class="bento-tile-action">
              <el-switch v-model="iceSettings.emoji" @change="onTileToggle" />
            </div>
          </div>

          <!-- Tile 2: Traditional -->
          <div class="bento-tile">
            <div class="bento-tile-main">
              <span class="bento-tile-icon">繁</span>
              <div class="bento-tile-texts">
                <span class="bento-tile-title">简繁智能输出</span>
                <span class="bento-tile-desc">默认进入 OpenCC 繁体字形模式</span>
              </div>
            </div>
            <div class="bento-tile-action">
              <el-switch v-model="iceSettings.traditionalization" @change="onTileToggle" />
            </div>
          </div>

          <!-- Tile 3: English Punctuation -->
          <div class="bento-tile">
            <div class="bento-tile-main">
              <span class="bento-tile-icon">🔤</span>
              <div class="bento-tile-texts">
                <span class="bento-tile-title">英文半角标点</span>
                <span class="bento-tile-desc">输入逗号句号直接输出半角英数标点</span>
              </div>
            </div>
            <div class="bento-tile-action">
              <el-switch v-model="iceSettings.ascii_punct" @change="onTileToggle" />
            </div>
          </div>

          <!-- Tile 4: Full Shape -->
          <div class="bento-tile">
            <div class="bento-tile-main">
              <span class="bento-tile-icon">🔲</span>
              <div class="bento-tile-texts">
                <span class="bento-tile-title">全角字符输入</span>
                <span class="bento-tile-desc">全角字母与两倍空格宽度模式</span>
              </div>
            </div>
            <div class="bento-tile-action">
              <el-switch v-model="iceSettings.full_shape" @change="onTileToggle" />
            </div>
          </div>

          <!-- Tile 5: Fuzzy Pinyin -->
          <div class="bento-tile">
            <div class="bento-tile-main">
              <span class="bento-tile-icon">🎯</span>
              <div class="bento-tile-texts">
                <span class="bento-tile-title">常用模糊音容错</span>
                <span class="bento-tile-desc">自动纠错 z/zh, c/ch, s/sh, n/l 等音节</span>
              </div>
            </div>
            <div class="bento-tile-action">
              <el-switch v-model="iceSettings.fuzzy_pinyin" @change="onTileToggle" />
            </div>
          </div>

          <!-- Tile 6: Quick Sync -->
          <div class="bento-tile">
            <div class="bento-tile-main">
              <span class="bento-tile-icon">🔄</span>
              <div class="bento-tile-texts">
                <span class="bento-tile-title">用户词库多端同步</span>
                <span class="bento-tile-desc">{{
                  syncConfig?.installation_id || "未设置设备ID"
                }}</span>
              </div>
            </div>
            <div class="bento-tile-action">
              <el-button
                size="small"
                type="primary"
                plain
                :loading="syncingUserdb"
                @click="triggerSync"
              >
                立即同步
              </el-button>
            </div>
          </div>
        </div>
      </section>

      <!-- 4. Clean Bottom Drawer Trigger (把原本占据首页的 3 个大表格收纳进抽屉) -->
      <section class="drawer-trigger-bar">
        <div class="drawer-trigger-badges">
          <span>📁 受管理配置 ({{ foundFiles }}/{{ customFiles.length }} 项就绪)</span>
          <span>·</span>
          <span>📚 活动词库 ({{ env?.user_dicts?.length ?? 0 }} 个)</span>
          <span>·</span>
          <span>💾 配置备份 ({{ backups.length }} 份)</span>
        </div>
        <el-button size="small" type="primary" plain @click="showTelemetryDrawer = true">
          打开系统与文件详情 ↗
        </el-button>
      </section>
    </template>

    <!-- Slide-over Drawer for System Files & Tables (技术与文件详情抽屉) -->
    <el-drawer
      v-model="showTelemetryDrawer"
      title="小狼毫系统文件、词库与快照详情"
      size="640px"
      direction="rtl"
    >
      <div class="drawer-telemetry-body custom-scrollbar">
        <!-- Drawer Tabs -->
        <el-segmented
          v-model="telemetryTab"
          :options="[
            { label: '受管理文件', value: 'files' },
            { label: '词库与快照', value: 'userdb' },
            { label: '快照备份', value: 'backups' },
          ]"
        />

        <!-- Tab 1: Files -->
        <div v-if="telemetryTab === 'files'" class="drawer-section-flow">
          <div class="drawer-meta-line">
            <span class="drawer-meta-text">用户配置目录: {{ env?.user_dir }}</span>
            <el-button
              link
              type="primary"
              size="small"
              @click="emit('openPath', 'open_rime_user_dir')"
            >
              打开文件夹
            </el-button>
          </div>
          <el-table :data="customFiles" stripe size="small" max-height="420">
            <el-table-column label="配置文件" min-width="180">
              <template #default="{ row }: { row: FileStatus }">
                <strong>{{ row.name }}</strong>
                <span class="mono-path file-path drawer-sub-path">{{ row.path }}</span>
              </template>
            </el-table-column>
            <el-table-column label="状态" width="100">
              <template #default="{ row }: { row: FileStatus }">
                <el-tag :type="row.exists ? 'success' : 'warning'" size="small">
                  {{ row.exists ? "存在" : "缺失" }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column label="大小" width="100">
              <template #default="{ row }: { row: FileStatus }">
                {{ formatBytes(row.size) }}
              </template>
            </el-table-column>
          </el-table>
        </div>

        <!-- Tab 2: Userdb & Snapshots -->
        <div v-else-if="telemetryTab === 'userdb'" class="drawer-section-flow">
          <div class="drawer-meta-line">
            <span class="drawer-title-text">活动词库 (*.userdb)</span>
            <div class="drawer-actions-row">
              <el-button size="small" :icon="Setting" @click="openSyncSettings">同步设置</el-button>
              <el-button size="small" type="primary" :loading="syncingUserdb" @click="triggerSync"
                >立即同步</el-button
              >
            </div>
          </div>
          <el-table :data="env?.user_dicts ?? []" stripe size="small" max-height="200">
            <el-table-column label="名称" prop="name" />
            <el-table-column label="大小">
              <template #default="{ row }: { row: UserDictInfo }">{{
                formatBytes(row.size_bytes)
              }}</template>
            </el-table-column>
            <el-table-column label="修改时间">
              <template #default="{ row }: { row: UserDictInfo }">{{
                formatTime(row.modified)
              }}</template>
            </el-table-column>
          </el-table>

          <span class="drawer-title-text drawer-sub-header">同步快照文件 (*.userdb.txt)</span>
          <el-table :data="syncConfig?.snapshot_files ?? []" stripe size="small" max-height="200">
            <el-table-column label="快照" prop="name" min-width="140" />
            <el-table-column label="词条估算" width="100">
              <template #default="{ row }: { row: UserdbSnapshotInfo }"
                >{{ row.entry_count }} 条</template
              >
            </el-table-column>
            <el-table-column label="操作" width="90">
              <template #default="{ row }: { row: UserdbSnapshotInfo }">
                <el-button link type="primary" size="small" @click="inspectSnapshotEntries(row)"
                  >查看词条</el-button
                >
              </template>
            </el-table-column>
          </el-table>
        </div>

        <!-- Tab 3: Backups -->
        <div v-else class="drawer-section-flow">
          <div class="drawer-meta-line">
            <span class="drawer-meta-text">共 {{ backups.length }} 份备份</span>
            <el-button
              type="primary"
              size="small"
              :loading="backingUp"
              @click="emit('createBackup')"
              >创建新备份</el-button
            >
          </div>
          <div v-for="b in backups.slice(0, 10)" :key="b.path" class="drawer-backup-item">
            <div class="drawer-backup-info">
              <strong class="drawer-backup-title">{{ backupLabel(b) }}</strong>
              <span class="drawer-backup-meta"
                >{{ formatTime(b.modified) }} · {{ b.files }} 个文件</span
              >
            </div>
            <div class="drawer-actions-row">
              <el-button link type="primary" size="small" @click="emit('openBackup', b)"
                >打开</el-button
              >
              <el-button
                link
                type="warning"
                size="small"
                :loading="restoringBackup === b.name"
                @click="emit('restoreBackup', b)"
                >恢复</el-button
              >
            </div>
          </div>
        </div>
      </div>
    </el-drawer>

    <!-- Dialogs -->
    <el-dialog v-model="showSyncDialog" title="Rime 同步配置" width="480px">
      <el-form label-position="top">
        <el-form-item label="设备标识 (installation_id)">
          <el-input v-model="syncForm.installation_id" placeholder="例如: win11-laptop" />
        </el-form-item>
        <el-form-item label="同步文件夹绝对路径 (sync_dir)">
          <el-input v-model="syncForm.sync_dir" placeholder="留空则使用默认 sync/ 目录" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showSyncDialog = false">取消</el-button>
        <el-button type="primary" :loading="savingSyncConfig" @click="saveSyncSettings"
          >保存配置</el-button
        >
      </template>
    </el-dialog>

    <el-dialog
      v-model="showEntriesDialog"
      :title="`词库快照: ${currentSnapshot?.name}`"
      width="680px"
    >
      <div class="entries-viewer-header">
        <el-input
          v-model="entriesSearchQuery"
          placeholder="搜索编码或词条..."
          clearable
          size="small"
          style="width: 240px"
          @input="onEntriesSearch"
        />
        <span class="entries-total-badge">共 {{ entriesTotal }} 个词条</span>
      </div>
      <el-table
        v-loading="loadingEntries"
        :data="snapshotEntries"
        stripe
        size="small"
        max-height="360"
      >
        <el-table-column label="编码 (Code)" prop="code" width="160" />
        <el-table-column label="词条 (Text)" prop="text" min-width="180" />
        <el-table-column label="词频权重" prop="weight" width="110" />
      </el-table>
      <div class="entries-viewer-footer">
        <el-pagination
          v-model:current-page="entriesPage"
          :page-size="entriesPageSize"
          :total="entriesTotal"
          layout="prev, pager, next"
          size="small"
          @current-change="fetchEntries"
        />
      </div>
    </el-dialog>
  </div>
</template>

<style scoped>
.overview-dashboard-page {
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding-bottom: 40px;
}

/* Sample chips */
.sample-chips-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.sample-chips-label {
  font-size: 11px;
  color: var(--ink-500);
  font-weight: 600;
}

.sample-chip {
  padding: 3px 9px;
  border-radius: 9999px;
  font-size: 11px;
  font-family: var(--font-mono);
  background: var(--color-surface-soft);
  border: 1px solid var(--color-line-soft);
  color: var(--ink-700);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.sample-chip:hover {
  background: var(--brand-50);
  border-color: var(--brand-200);
  color: var(--brand-600);
}

html[data-theme="dark"] .sample-chip:hover {
  background: rgba(37, 99, 235, 0.2);
  border-color: rgba(96, 165, 250, 0.4);
  color: var(--brand-300);
}

/* Clear button on typing query input */
.stage-clear-btn {
  position: absolute;
  right: 12px;
  background: transparent;
  border: none;
  color: var(--ink-400);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  border-radius: 4px;
}

.stage-clear-btn:hover {
  color: var(--ink-700);
}

/* Candidate list & items in arena */
.weasel-preedit {
  font-size: 15px;
  font-family: var(--font-mono);
  font-weight: 700;
  color: var(--brand-600);
  padding-right: 12px;
  border-right: 1.5px solid var(--color-line-soft);
  user-select: none;
}

.weasel-cand-list {
  display: flex;
  align-items: center;
  gap: 12px;
}

.weasel-cand-list.vertical {
  flex-direction: column;
  align-items: flex-start;
  gap: 6px;
}

.weasel-cand-item {
  font-size: 14px;
  font-weight: 500;
  color: var(--ink-800);
  padding: 2px 6px;
  border-radius: 4px;
  display: inline-flex;
  align-items: baseline;
  gap: 4px;
}

.weasel-cand-item em {
  font-style: normal;
  font-size: 11px;
  font-weight: 600;
  color: var(--ink-400);
}

.weasel-cand-item.active {
  color: var(--brand-600);
  font-weight: 700;
  background: rgba(37, 99, 235, 0.08);
}

html[data-theme="dark"] .weasel-cand-item.active {
  color: #60a5fa;
  background: rgba(59, 130, 246, 0.18);
}

.weasel-cand-item.active em {
  color: var(--brand-500);
}

/* Knob pill buttons */
.knob-pill-group {
  display: inline-flex;
  background: var(--color-surface-soft);
  padding: 2px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--color-line-soft);
  gap: 2px;
}

.knob-pill-btn {
  background: transparent;
  border: none;
  border-radius: 4px;
  padding: 2px 8px;
  font-size: 11px;
  font-weight: 600;
  color: var(--ink-600);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.knob-pill-btn:hover {
  color: var(--ink-900);
}

.knob-pill-btn.active {
  background: var(--color-surface);
  color: var(--brand-600);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
}

html[data-theme="dark"] .knob-pill-btn.active {
  background: rgba(255, 255, 255, 0.1);
  color: #60a5fa;
}

/* Bento section header */
.bento-section-wrap {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.bento-header-line {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.bento-section-kicker {
  font-size: 12px;
  font-weight: 750;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--ink-500);
}

.bento-more-link {
  font-size: 12px;
  font-weight: 600;
  color: var(--brand-500);
  cursor: pointer;
  transition: color var(--transition-fast);
}

.bento-more-link:hover {
  color: var(--brand-600);
  text-decoration: underline;
}

/* Drawer contents */
.drawer-telemetry-body {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.drawer-section-flow {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.drawer-meta-line {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.drawer-meta-text {
  font-size: 12px;
  color: var(--ink-500);
}

.drawer-sub-path {
  display: block;
  font-size: 10px;
  color: var(--ink-400);
  margin-top: 2px;
}

.drawer-title-text {
  font-size: 12px;
  font-weight: 700;
  color: var(--ink-800);
}

.drawer-sub-header {
  margin-top: 8px;
}

.drawer-actions-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.drawer-backup-item {
  padding: 12px 14px;
  background: var(--color-surface-soft);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-line-soft);
  font-size: 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.drawer-backup-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.drawer-backup-title {
  color: var(--ink-800);
  font-size: 12px;
}

.drawer-backup-meta {
  color: var(--ink-400);
  font-size: 10px;
}

/* Dialog Entries Viewer */
.entries-viewer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.entries-total-badge {
  font-size: 12px;
  color: var(--ink-500);
  font-weight: 600;
}

.entries-viewer-footer {
  margin-top: 14px;
  display: flex;
  justify-content: flex-end;
}
</style>
