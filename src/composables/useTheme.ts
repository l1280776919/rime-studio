import { ref } from "vue";

const DARK_THEME_KEY = "rime-studio-theme";

function applyTheme(dark: boolean) {
  if (dark) {
    document.documentElement.dataset.theme = "dark";
  } else {
    delete document.documentElement.dataset.theme;
  }
  localStorage.setItem(DARK_THEME_KEY, dark ? "dark" : "light");
}

export function useTheme() {
  const isDark = ref(false);

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

  return { isDark, toggleTheme, initTheme };
}
