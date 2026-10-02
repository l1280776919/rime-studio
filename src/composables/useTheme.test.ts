import { afterEach, expect, it, vi } from "vitest";
import "vue";

/** 最小浏览器接口替身：验证持久化与 DOM 变量，不依赖真实 Rime 环境。 */
async function setup(stored: string | null = null, systemDark = false) {
  vi.resetModules();
  const values = new Map<string, string>();
  const style = {
    setProperty: (key: string, value: string) => values.set(key, value),
    removeProperty: (key: string) => values.delete(key),
    colorScheme: "",
  };
  const dataset: Record<string, string> = {};
  let onChange: (event: { matches: boolean }) => void = () => {};
  const addEventListener = vi.fn((_event: string, listener: typeof onChange) => {
    onChange = listener;
  });
  const storage = { getItem: vi.fn(() => stored), setItem: vi.fn() };
  vi.stubGlobal("document", { documentElement: { style, dataset } });
  vi.stubGlobal("localStorage", storage);
  vi.stubGlobal("window", { matchMedia: () => ({ matches: systemDark, addEventListener }) });
  const { useTheme } = await import("./useTheme");
  return {
    useTheme,
    values,
    dataset,
    storage,
    addEventListener,
    changeSystem: (matches: boolean) => onChange({ matches }),
  };
}

afterEach(() => vi.unstubAllGlobals());

it("restores a saved Happy Hues theme and shares selection between both pickers", async () => {
  const state = await setup("happy-hues-12");
  const sidebar = state.useTheme();
  const settings = state.useTheme();
  sidebar.initTheme();
  settings.initTheme();
  expect(sidebar.isDark.value).toBe(true);
  expect(settings.selectedTheme.value).toBe("happy-hues-12");
  expect(state.values.get("--color-bg")).toBe("#232946");
  expect(state.addEventListener).not.toHaveBeenCalled();
  settings.setTheme("happy-hues-17");
  expect(sidebar.selectedTheme.value).toBe("happy-hues-17");
  expect(sidebar.isDark.value).toBe(false);
  expect(state.dataset.theme).toBeUndefined();
  expect(state.storage.setItem).toHaveBeenLastCalledWith("rime-studio-theme", "happy-hues-17");
});

it("clears all palette overrides when returning to the original themes", async () => {
  const state = await setup();
  const theme = state.useTheme();
  theme.initTheme();
  for (const palette of theme.workbenchThemes) {
    theme.setTheme(palette.name);
    expect(state.values.get("--color-bg")).toBe(palette.swatches[0]);
    expect(state.dataset.workbenchTheme).toBe(palette.name);
  }
  theme.setTheme("dark");
  expect(state.values.size).toBe(0);
  expect(state.dataset.workbenchTheme).toBeUndefined();
  expect(state.dataset.theme).toBe("dark");
  theme.toggleTheme();
  expect(theme.selectedTheme.value).toBe("light");
});

it("defaults to Happy Hues 03 even when the system is dark or the saved theme is invalid", async () => {
  const state = await setup("obsolete-theme", true);
  const theme = state.useTheme();
  theme.initTheme();
  expect(theme.selectedTheme.value).toBe("happy-hues-3");
  expect(state.values.get("--color-bg")).toBe("#fffffe");
  state.changeSystem(false);
  expect(theme.isDark.value).toBe(false);
  theme.setTheme("happy-hues-4");
  state.changeSystem(false);
  expect(theme.selectedTheme.value).toBe("happy-hues-4");
});

it("uses Happy Hues 03 on first launch", async () => {
  const state = await setup(null, true);
  const theme = state.useTheme();
  theme.initTheme();
  expect(theme.selectedTheme.value).toBe("happy-hues-3");
  expect(theme.isDark.value).toBe(false);
});

it("still switches themes when storage is unavailable", async () => {
  const state = await setup();
  state.storage.getItem.mockImplementation(() => {
    throw new Error("storage denied");
  });
  state.storage.setItem.mockImplementation(() => {
    throw new Error("storage denied");
  });
  const theme = state.useTheme();
  expect(() => theme.initTheme()).not.toThrow();
  expect(() => theme.setTheme("happy-hues-10")).not.toThrow();
  expect(state.values.get("--color-bg")).toBe("#004643");
});
