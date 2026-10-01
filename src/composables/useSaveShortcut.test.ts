import { expect, it, vi } from "vitest";
const hooks = vi.hoisted(() => ({
  mounted: [] as (() => void)[],
  activated: [] as (() => void)[],
  deactivated: [] as (() => void)[],
  unmounted: [] as (() => void)[],
}));
vi.mock("vue", () => ({
  onMounted: (callback: () => void) => hooks.mounted.push(callback),
  onActivated: (callback: () => void) => hooks.activated.push(callback),
  onDeactivated: (callback: () => void) => hooks.deactivated.push(callback),
  onBeforeUnmount: (callback: () => void) => hooks.unmounted.push(callback),
}));
import { useSaveShortcut } from "./useSaveShortcut";

it("only saves the active page, ignores composition and dialogs, and cleans up its listener", () => {
  let listener: (event: KeyboardEvent) => void = () => {};
  const remove = vi.fn();
  class Target {
    constructor(public dialog = false) {}
    closest() {
      return this.dialog;
    }
  }
  vi.stubGlobal("Element", Target);
  vi.stubGlobal("window", {
    addEventListener: (_name: string, callback: typeof listener) => {
      listener = callback;
    },
    removeEventListener: remove,
  });
  const save = vi.fn();
  useSaveShortcut(save);
  hooks.mounted[0]();
  const event = (extra = {}) =>
    ({
      key: "s",
      ctrlKey: true,
      preventDefault: vi.fn(),
      target: new Target(),
      ...extra,
    }) as unknown as KeyboardEvent;
  const first = event();
  listener(first);
  expect(save).toHaveBeenCalledTimes(1);
  expect(first.preventDefault).toHaveBeenCalled();
  for (const extra of [
    { isComposing: true },
    { repeat: true },
    { altKey: true },
    { defaultPrevented: true },
    { target: new Target(true) },
  ])
    listener(event(extra));
  hooks.deactivated[0]();
  listener(event());
  expect(save).toHaveBeenCalledTimes(1);
  hooks.activated[0]();
  listener(event({ ctrlKey: false, metaKey: true }));
  expect(save).toHaveBeenCalledTimes(2);
  hooks.unmounted[0]();
  expect(remove).toHaveBeenCalledWith("keydown", listener);
  vi.unstubAllGlobals();
});
