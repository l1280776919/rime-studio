<script setup lang="ts">
import { computed, nextTick, onMounted, onBeforeUnmount, ref, watch } from "vue";
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

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  "update:visible": [value: boolean];
  navigate: [pageKey: string];
  deploy: [];
  restartServer: [];
  createBackup: [];
  openSandbox: [];
}>();

const searchInput = ref<HTMLInputElement>();
const query = ref("");
const selectedIndex = ref(0);

interface CommandItem {
  id: string;
  category: "navigation" | "action" | "theme";
  title: string;
  subtitle?: string;
  icon: unknown;
  action: () => void;
  shortcut?: string;
}

const commands = computed<CommandItem[]>(() => {
  const list: CommandItem[] = [
    // Navigation
    {
      id: "nav-overview",
      category: "navigation",
      title: "概览与状态指挥舱",
      subtitle: "总控工作台 · 实景打字舞台 · 6大特性磁贴",
      icon: Monitor,
      action: () => emit("navigate", "overview"),
      shortcut: "1",
    },
    {
      id: "nav-quick",
      category: "navigation",
      title: "快速设置与排版",
      subtitle: "候选词数 · 横竖排 · 按键行为 · 雾凇组件",
      icon: MagicStick,
      action: () => emit("navigate", "quick"),
      shortcut: "2",
    },
    {
      id: "nav-appearance",
      category: "navigation",
      title: "候选窗主题与外观工坊",
      subtitle: "实景桌面模拟 · 视网膜 Bento 主题矩阵 · 色彩精修",
      icon: Brush,
      action: () => emit("navigate", "appearance"),
      shortcut: "3",
    },
    {
      id: "nav-schemas",
      category: "navigation",
      title: "方案工坊与键盘实验室",
      subtitle: "本地方案库 · 社区生态市场 · 双拼键位图",
      icon: Files,
      action: () => emit("navigate", "schemas"),
      shortcut: "4",
    },
    {
      id: "nav-dictionaries",
      category: "navigation",
      title: "词库与语料枢纽",
      subtitle: "搜狗词库导入 · 社区在线词库 · 智能去重体检",
      icon: Collection,
      action: () => emit("navigate", "dictionaries"),
      shortcut: "5",
    },
    {
      id: "nav-phrases",
      category: "navigation",
      title: "自定义短语工坊",
      subtitle: "短语维护 · 编码映射 · TSV 批量导入导出",
      icon: EditPen,
      action: () => emit("navigate", "phrases"),
      shortcut: "6",
    },
    {
      id: "nav-backups",
      category: "navigation",
      title: "时光机配置备份仓",
      subtitle: "自动轮转快照 · 差异比对 · 一键安全回滚",
      icon: FolderOpened,
      action: () => emit("navigate", "backups"),
      shortcut: "7",
    },
    {
      id: "nav-editor",
      category: "navigation",
      title: "配置中心 (YAML 编辑室)",
      subtitle: "直接查看、定位与无损编辑 Rime 配置文件",
      icon: Document,
      action: () => emit("navigate", "editor"),
      shortcut: "8",
    },
    {
      id: "nav-about",
      category: "navigation",
      title: "关于 Rime Studio 与生态",
      subtitle: "检查软件更新 · 社区开源生态 · 运行日志",
      icon: InfoFilled,
      action: () => emit("navigate", "about"),
      shortcut: "9",
    },

    // Actions
    {
      id: "act-deploy",
      category: "action",
      title: "一键部署生效 (Deploy)",
      subtitle: "下发 Rime 编译指令，立即重新加载全部配置",
      icon: UploadFilled,
      action: () => emit("deploy"),
      shortcut: "Ctrl+D",
    },
    {
      id: "act-restart",
      category: "action",
      title: "重启小狼毫后台服务 (WeaselServer)",
      subtitle: "解决输入法进程卡死或无响应问题",
      icon: SwitchButton,
      action: () => emit("restartServer"),
    },
    {
      id: "act-backup",
      category: "action",
      title: "立即创建配置时光机快照",
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

    // Themes
    ...presets.map((preset) => ({
      id: `theme-${preset.name}`,
      category: "theme" as const,
      title: `应用主题：${preset.label}`,
      subtitle: `配色代号：${preset.name}`,
      icon: Brush,
      action: async () => {
        try {
          await api.saveAppearance({
            theme_name: preset.name,
            font_point: 11,
            label_font_point: 10,
            font_face: "",
            label_font_face: "",
            page_size: 7,
            switch_key: "shift",
            horizontal: true,
            inline_preedit: true,
            candidate_format: "%c. %@",
            corner_radius: 8,
            border_height: 4,
            border_width: 4,
            line_spacing: 6,
            spacing: 8,
            ...preset.colors,
          });
          emit("deploy");
          ElMessage.success(`已切换至「${preset.label}」并触发部署`);
        } catch {
          ElMessage.error("应用配色失败");
        }
      },
    })),
  ];

  return list;
});

const filteredCommands = computed(() => {
  const q = query.value.trim().toLowerCase();
  if (!q) return commands.value;
  return commands.value.filter(
    (c) =>
      c.title.toLowerCase().includes(q) ||
      (c.subtitle && c.subtitle.toLowerCase().includes(q)) ||
      c.category.includes(q),
  );
});

watch(query, () => {
  selectedIndex.value = 0;
});

function close() {
  emit("update:visible", false);
  query.value = "";
  selectedIndex.value = 0;
}

function executeSelected() {
  const item = filteredCommands.value[selectedIndex.value];
  if (item) {
    item.action();
    close();
  }
}

function selectAndExecute(item: CommandItem) {
  item.action();
  close();
}

function handleKeydown(e: KeyboardEvent) {
  if (!props.visible) return;

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

onMounted(() => {
  window.addEventListener("keydown", handleKeydown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <Teleport to="body">
    <Transition name="palette-fade">
      <div v-if="visible" class="palette-overlay" @click.self="close">
        <div class="palette-dialog panel">
          <!-- Search Bar -->
          <div class="palette-search-bar">
            <el-icon class="search-icon"><Search /></el-icon>
            <input
              ref="searchInput"
              v-model="query"
              type="text"
              class="palette-input"
              placeholder="快速搜索功能、直接跳转页面或切换主题... (↑↓ 选择，Enter 执行，Esc 关闭)"
            />
            <span v-if="query" class="clear-query" @click="query = ''">×</span>
            <kbd class="esc-badge" @click="close">ESC</kbd>
          </div>

          <!-- Results Stream -->
          <div class="palette-results-list custom-scrollbar">
            <div v-if="filteredCommands.length === 0" class="palette-empty">
              未找到与「{{ query }}」相关的快捷命令
            </div>

            <div
              v-for="(item, idx) in filteredCommands"
              :key="item.id"
              class="palette-item"
              :class="{ selected: idx === selectedIndex }"
              @mouseenter="selectedIndex = idx"
              @click="selectAndExecute(item)"
            >
              <div class="item-icon-box">
                <component :is="item.icon" />
              </div>

              <div class="item-meta">
                <span class="item-title">{{ item.title }}</span>
                <span v-if="item.subtitle" class="item-subtitle">{{ item.subtitle }}</span>
              </div>

              <div class="item-trailing">
                <span class="category-tag">{{
                  item.category === "navigation"
                    ? "页面"
                    : item.category === "action"
                      ? "动作"
                      : "主题"
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
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.palette-overlay {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.55);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  z-index: 9999;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 12vh;
}

.palette-dialog {
  width: 620px;
  max-width: 90vw;
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
  max-height: 380px;
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

/* Transitions */
.palette-fade-enter-active,
.palette-fade-leave-active {
  transition: all 0.2s ease;
}

.palette-fade-enter-from,
.palette-fade-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
</style>
