import { computed, ref } from "vue";
import { workbenchThemes } from "../appearance/workbenchThemes";

const THEME_KEY = "rime-studio-theme";
const DEFAULT_THEME = "happy-hues-3";
// 多个入口共享状态，确保侧栏显示与实际主题一致。
const selectedTheme = ref(DEFAULT_THEME);
const isDark = computed(
  () =>
    selectedTheme.value === "dark" ||
    !!workbenchThemes.find((theme) => theme.name === selectedTheme.value)?.dark,
);
let appliedTokens: string[] = [];
let initialized = false;

/** 清除上一套颜色后应用新主题，切回默认模式时恢复原始 CSS。 */
function applyTheme(name: string) {
  const resolvedName =
    workbenchThemes.some((item) => item.name === name) || name === "light" || name === "dark"
      ? name
      : DEFAULT_THEME;
  const theme = workbenchThemes.find((item) => item.name === resolvedName);
  selectedTheme.value = resolvedName;
  const root = document.documentElement;
  for (const key of appliedTokens) root.style.removeProperty(key);
  appliedTokens = Object.keys(theme?.tokens ?? {});
  for (const [key, value] of Object.entries(theme?.tokens ?? {}))
    root.style.setProperty(key, value);
  if (isDark.value) root.dataset.theme = "dark";
  else delete root.dataset.theme;
  if (theme) root.dataset.workbenchTheme = theme.name;
  else delete root.dataset.workbenchTheme;
  root.style.colorScheme = isDark.value ? "dark" : "light";
}

function setTheme(name: string) {
  applyTheme(name);
  // 存储受限时仍允许即时切换；无需阻断工作台操作。
  try {
    localStorage.setItem(THEME_KEY, selectedTheme.value);
  } catch {
    /* 当前会话继续生效。 */
  }
}

function initTheme() {
  if (initialized) return;
  initialized = true;
  let stored: string | null = null;
  try {
    stored = localStorage.getItem(THEME_KEY);
  } catch {
    /* 使用 Happy Hues 03 浅色默认主题。 */
  }
  // 默认固定为 03 浅色，已有的手动选择继续保留，不再被系统明暗变化覆盖。
  applyTheme(stored ?? DEFAULT_THEME);
}

export function useTheme() {
  return {
    isDark,
    selectedTheme,
    workbenchThemes,
    setTheme,
    initTheme,
    toggleTheme: () => setTheme(isDark.value ? "light" : "dark"),
  };
}
