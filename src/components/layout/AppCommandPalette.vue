<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";
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
  Search,
  SwitchButton,
  UploadFilled,
} from "@element-plus/icons-vue";
import { presets } from "../../appearance/schemes";
import { api } from "../../api";
import { ElMessage } from "element-plus";
import { quickSettingsCatalog } from "../../settings/quickSettingsCatalog";
import { parseRecentCommands, rememberCommand, searchCommands } from "../../utils/commandSearch";

const props = defineProps<{
  visible: boolean;
  busy: boolean;
  hasDeployer: boolean;
}>();

const emit = defineEmits<{
  "update:visible": [value: boolean];
  navigate: [pageKey: string];
  deploy: [];
  restartServer: [];
  createBackup: [];
  previewTheme: [name: string];
  navigateSetting: [id: string];
}>();

const searchInput = ref<HTMLInputElement>();
const query = ref("");
const selectedIndex = ref(0);
const resultsList = ref<HTMLDivElement>();
const executing = ref(false);
const recentCommands = ref<string[]>([]);
const category = ref("all");
const categories = [
  { id: "all", label: "全部" },
  { id: "navigation", label: "页面" },
  { id: "setting", label: "设置" },
  { id: "action", label: "操作" },
  { id: "theme", label: "配色" },
];
const RECENT_KEY = "rime-studio:recent-commands:v1";
function persistRecent() {
  try {
    localStorage.setItem(RECENT_KEY, JSON.stringify(recentCommands.value));
  } catch {
    /* Search remains usable when storage is unavailable. */
  }
}
onMounted(() => {
  try {
    recentCommands.value = parseRecentCommands(
      localStorage.getItem(RECENT_KEY),
      commands.value.filter((item) => item.category !== "action").map((item) => item.id),
    );
  } catch {
    recentCommands.value = [];
  }
});

interface CommandItem {
  id: string;
  category: "navigation" | "action" | "theme" | "setting";
  title: string;
  subtitle?: string;
  keywords?: string;
  icon: unknown;
  action: () => unknown;
  unavailable?: string;
  shortcut?: string;
}

const commands = computed<CommandItem[]>(() => {
  const list: CommandItem[] = [
    // Navigation
    {
      id: "nav-overview",
      category: "navigation",
      title: "概览与状态",
      subtitle: "当前方案 · 输入测试 · 部署状态",
      icon: Monitor,
      action: () => emit("navigate", "overview"),
    },
    {
      id: "nav-quick",
      category: "navigation",
      title: "快速设置与排版",
      subtitle: "候选词数 · 横竖排 · 按键行为 · 雾凇组件",
      icon: MagicStick,
      action: () => emit("navigate", "quick"),
    },
    {
      id: "nav-appearance",
      category: "navigation",
      title: "主题配置",
      subtitle: "工作台主题 · 候选窗配色 · 字体排版 · 候选窗布局",
      icon: Brush,
      action: () => emit("navigate", "appearance"),
    },
    {
      id: "nav-schemas",
      category: "navigation",
      title: "方案管理",
      subtitle: "本地方案库 · 社区生态市场 · 双拼键位图",
      icon: Files,
      action: () => emit("navigate", "schemas"),
    },
    {
      id: "nav-dictionaries",
      category: "navigation",
      title: "词库管理",
      subtitle: "搜狗词库导入 · 社区在线词库 · 智能去重体检",
      icon: Collection,
      action: () => emit("navigate", "dictionaries"),
    },
    {
      id: "nav-phrases",
      category: "navigation",
      title: "自定义短语",
      subtitle: "短语维护 · 编码映射 · TSV 批量导入导出",
      icon: EditPen,
      action: () => emit("navigate", "phrases"),
    },
    {
      id: "nav-backups",
      category: "navigation",
      title: "快照与备份",
      subtitle: "自动轮转快照 · 差异比对 · 一键安全回滚",
      icon: FolderOpened,
      action: () => emit("navigate", "backups"),
    },
    {
      id: "nav-editor",
      category: "navigation",
      title: "配置中心 (YAML 编辑室)",
      subtitle: "直接查看、定位与无损编辑 Rime 配置文件",
      icon: Document,
      action: () => emit("navigate", "editor"),
    },
    {
      id: "nav-about",
      category: "navigation",
      title: "关于应用",
      subtitle: "检查软件更新 · 社区开源生态 · 运行日志",
      icon: InfoFilled,
      action: () => emit("navigate", "about"),
    },

    // Actions
    {
      id: "act-deploy",
      unavailable: props.busy
        ? "请等待当前操作完成"
        : !props.hasDeployer
          ? "未检测到小狼毫部署工具"
          : undefined,
      category: "action",
      title: "一键部署生效 (Deploy)",
      subtitle: "下发 Rime 编译指令，立即重新加载全部配置",
      icon: UploadFilled,
      action: () => emit("deploy"),
    },
    {
      id: "act-restart",
      unavailable: props.busy
        ? "请等待当前操作完成"
        : !props.hasDeployer
          ? "未检测到小狼毫部署工具"
          : undefined,
      category: "action",
      title: "重启小狼毫后台服务 (WeaselServer)",
      subtitle: "解决输入法进程卡死或无响应问题",
      icon: SwitchButton,
      action: () => emit("restartServer"),
    },
    {
      id: "act-backup",
      unavailable: props.busy ? "请等待当前操作完成" : undefined,
      category: "action",
      title: "创建配置备份",
      subtitle: "将当前全部配置文件安全封存到应用归档目录",
      icon: FolderOpened,
      action: () => emit("createBackup"),
    },
    {
      id: "act-userdir",
      category: "action",
      title: "打开 Rime 用户数据目录",
      subtitle: "在 Windows 系统文件资源管理器中直接定位",
      icon: FolderOpened,
      action: async () => {
        await api.openRimeUserDir();
      },
    },
    {
      id: "act-logdir",
      category: "action",
      title: "打开应用运行日志目录",
      subtitle: "排查前端控制台与后端崩溃日志",
      icon: Document,
      action: async () => {
        await api.openAppLogDir();
      },
    },

    ...quickSettingsCatalog.map((setting) => ({
      id: `setting-${setting.id}`,
      category: "setting" as const,
      title: setting.title,
      subtitle: "打开快速设置并定位到此项",
      keywords: setting.keywords,
      icon: MagicStick,
      action: () => emit("navigateSetting", setting.id),
    })),
    // Themes
    ...presets.map((preset) => ({
      id: `theme-${preset.name}`,
      category: "theme" as const,
      title: `预览配色：${preset.label}`,
      subtitle: `进入外观页预览，保留当前字体与布局；保存后才生效 · ${preset.name}`,
      icon: Brush,
      action: () => emit("previewTheme", preset.name),
    })),
  ];

  return list;
});

const filteredCommands = computed(() =>
  searchCommands(commands.value, query.value, recentCommands.value, category.value),
);

watch([query, category], () => {
  selectedIndex.value = 0;
});

function close() {
  emit("update:visible", false);
  query.value = "";
  category.value = "all";
  selectedIndex.value = 0;
}

async function selectAndExecute(item: CommandItem) {
  if (executing.value || item.unavailable) return;
  executing.value = true;
  close();
  try {
    await item.action();
    if (item.category !== "action") {
      recentCommands.value = rememberCommand(recentCommands.value, item.id);
      persistRecent();
    }
  } catch (error) {
    ElMessage.error(`操作失败：${String(error)}`);
  } finally {
    executing.value = false;
  }
}

function executeSelected() {
  const item = filteredCommands.value[selectedIndex.value];
  if (item) void selectAndExecute(item);
}

function handleKeydown(e: KeyboardEvent) {
  if (!props.visible || e.isComposing || e.repeat || e.keyCode === 229) return;
  if (e.key !== "Escape" && e.target !== searchInput.value) return;
  if (!filteredCommands.value.length && ["ArrowDown", "ArrowUp", "Enter"].includes(e.key)) {
    e.preventDefault();
    return;
  }

  if (e.key === "ArrowDown") {
    e.preventDefault();
    if (selectedIndex.value < filteredCommands.value.length - 1) {
      selectedIndex.value++;
    } else {
      selectedIndex.value = 0;
    }
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    if (selectedIndex.value > 0) {
      selectedIndex.value--;
    } else {
      selectedIndex.value = filteredCommands.value.length - 1;
    }
  } else if (e.key === "Enter") {
    e.preventDefault();
    executeSelected();
  } else if (e.key === "Escape") {
    e.preventDefault();
    close();
  }
}

watch(
  () => props.visible,
  (val) => {
    if (val) {
      selectedIndex.value = 0;
      nextTick(() => {
        searchInput.value?.focus();
      });
    }
  },
);

watch(selectedIndex, async () => {
  await nextTick();
  resultsList.value?.querySelector(".selected")?.scrollIntoView({ block: "nearest" });
});
</script>

<template>
  <el-dialog
    :model-value="visible"
    title="快捷命令"
    width="620px"
    top="12vh"
    class="command-palette"
    :show-close="false"
    @update:model-value="close"
    @opened="searchInput?.focus()"
  >
    <div class="palette-dialog" @keydown="handleKeydown">
      <!-- Search Bar -->
      <div class="palette-search-bar">
        <el-icon class="search-icon"><Search /></el-icon>
        <input
          ref="searchInput"
          v-model="query"
          type="text"
          class="palette-input"
          aria-label="搜索页面和操作"
          role="combobox"
          aria-autocomplete="list"
          :aria-expanded="visible"
          aria-controls="command-results"
          :aria-activedescendant="
            filteredCommands[selectedIndex]
              ? `command-${filteredCommands[selectedIndex].id}`
              : undefined
          "
          placeholder="搜索页面、操作或配色…"
        />
        <button
          v-if="query"
          type="button"
          class="clear-query"
          aria-label="清空搜索"
          @click="
            query = '';
            searchInput?.focus();
          "
        >
          ×
        </button>
        <button type="button" class="esc-badge" aria-label="关闭快捷命令" @click="close">
          ESC
        </button>
      </div>

      <div class="palette-filters" role="group" aria-label="快捷命令分类">
        <button
          v-for="filter in categories"
          :key="filter.id"
          type="button"
          :aria-pressed="category === filter.id"
          :class="{ active: category === filter.id }"
          @click="category = filter.id"
        >
          {{ filter.label }}
        </button>
        <button
          v-if="!query && recentCommands.length"
          type="button"
          class="clear-recent"
          @click="
            recentCommands = [];
            persistRecent();
          "
        >
          清除最近使用
        </button>
      </div>
      <!-- Results Stream -->
      <div
        id="command-results"
        ref="resultsList"
        role="listbox"
        aria-label="快捷命令"
        class="palette-results-list custom-scrollbar"
      >
        <div v-if="filteredCommands.length === 0" class="palette-empty">
          未找到与「{{ query }}」相关的快捷命令
        </div>

        <div
          v-for="(item, idx) in filteredCommands"
          :id="`command-${item.id}`"
          :key="item.id"
          class="palette-item"
          role="option"
          :aria-selected="idx === selectedIndex"
          :aria-disabled="Boolean(item.unavailable)"
          :class="{ selected: idx === selectedIndex, unavailable: item.unavailable }"
          @mouseenter="selectedIndex = idx"
          @click="selectAndExecute(item)"
        >
          <div class="item-icon-box">
            <component :is="item.icon" />
          </div>

          <div class="item-meta">
            <span class="item-title">{{ item.title }}</span>
            <span v-if="item.subtitle" class="item-subtitle">{{
              item.unavailable ?? item.subtitle
            }}</span>
          </div>

          <div class="item-trailing">
            <span v-if="!query && recentCommands.includes(item.id)" class="category-tag">最近</span>
            <span class="category-tag">{{
              item.category === "navigation"
                ? "页面"
                : item.category === "setting"
                  ? "设置"
                  : item.category === "action"
                    ? "操作"
                    : "配色"
            }}</span>
            <kbd v-if="item.shortcut" class="item-shortcut">{{ item.shortcut }}</kbd>
          </div>
        </div>
      </div>

      <!-- Bottom Status Footer -->
      <div class="palette-footer">
        <div class="footer-tips">
          <span><kbd>↑</kbd><kbd>↓</kbd> 导航</span>
          <span><kbd>↵</kbd> 执行</span>
          <span><kbd>ESC</kbd> 退出</span>
        </div>
        <span class="total-matches">找到 {{ filteredCommands.length }} 个快捷命令</span>
      </div>
    </div>
  </el-dialog>
</template>

<style scoped>
.palette-dialog {
  width: 100%;
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  border-radius: var(--radius-lg);
  box-shadow:
    0 24px 64px -12px rgba(0, 0, 0, 0.45),
    0 0 0 1px rgba(255, 255, 255, 0.1);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.palette-search-bar {
  display: flex;
  align-items: center;
  padding: 14px 18px;
  gap: 12px;
  border-bottom: 1px solid var(--color-line-soft);
  background: var(--color-surface-soft);
}

.search-icon {
  font-size: 18px;
  color: var(--brand-500);
}

.palette-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  font-size: 14px;
  font-weight: 550;
  color: var(--ink-900);
}

.palette-input::placeholder {
  color: var(--color-muted);
  font-weight: 450;
}

.clear-query {
  border: 0;
  background: transparent;
  font-size: 18px;
  color: var(--ink-400);
  cursor: pointer;
  padding: 0 4px;
}

.esc-badge {
  font-size: 10px;
  font-family: var(--font-mono);
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  padding: 2px 6px;
  border-radius: 4px;
  color: var(--ink-500);
  cursor: pointer;
}

.palette-results-list {
  max-height: min(380px, 50vh);
  overflow-y: auto;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.palette-empty {
  padding: 36px;
  text-align: center;
  font-size: 13px;
  color: var(--color-muted);
}

.palette-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.12s ease;
}

.palette-item.unavailable {
  opacity: 0.55;
  cursor: not-allowed;
}

.palette-item:hover,
.palette-item.selected {
  background: var(--brand-50, #eff6ff);
  transform: translateX(2px);
}

html[data-theme="dark"] .palette-item:hover,
html[data-theme="dark"] .palette-item.selected {
  background: rgba(37, 99, 235, 0.15);
}

.item-icon-box {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm);
  background: var(--color-surface-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--brand-600);
  font-size: 16px;
  flex-shrink: 0;
}

.palette-item.selected .item-icon-box {
  background: var(--brand-600);
  color: #fff;
}

.item-meta {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.item-title {
  font-size: 13px;
  font-weight: 700;
  color: var(--ink-900);
}

.item-subtitle {
  font-size: 11px;
  color: var(--color-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-trailing {
  display: flex;
  align-items: center;
  gap: 6px;
}

.category-tag {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 4px;
  background: var(--color-surface-soft);
  color: var(--ink-500);
}

.item-shortcut {
  font-size: 10px;
  font-family: var(--font-mono);
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  padding: 1px 5px;
  border-radius: 4px;
  color: var(--ink-500);
}

.palette-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 16px;
  background: var(--color-surface-soft);
  border-top: 1px solid var(--color-line-soft);
  font-size: 11px;
  color: var(--color-muted);
}

.footer-tips {
  display: flex;
  gap: 12px;
}

.footer-tips kbd {
  font-family: var(--font-mono);
  background: var(--color-surface);
  border: 1px solid var(--color-line-soft);
  padding: 1px 4px;
  border-radius: 3px;
  font-size: 10px;
}
</style>

<style>
.command-palette.el-dialog {
  max-width: calc(100vw - 32px);
  padding: 0;
  background: var(--color-surface);
  border-radius: var(--radius-lg);
}
.command-palette .el-dialog__header {
  padding: 0;
  height: 0;
  overflow: hidden;
}
</style>

<style scoped>
.palette-filters {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 8px 16px;
  border-bottom: 1px solid var(--color-line-soft);
}
.palette-filters button {
  border: 1px solid var(--color-line);
  background: var(--color-surface);
  color: var(--ink-600);
  border-radius: var(--radius-sm);
  font-size: 11px;
  padding: 4px 8px;
  cursor: pointer;
}
.palette-filters button.active {
  background: var(--brand-50);
  border-color: var(--brand-400);
  color: var(--brand-700);
}
.palette-filters .clear-recent {
  margin-left: auto;
}
.palette-filters button:focus-visible {
  outline: 2px solid var(--brand-500);
  outline-offset: 2px;
}
</style>
