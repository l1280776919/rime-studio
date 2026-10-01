import { expect, it, vi } from "vitest";
const hooks = vi.hoisted(() => ({
  activated: [] as (() => void)[],
  deactivated: [] as (() => void)[],
}));
vi.mock("vue", async (original) => ({
  ...(await original<typeof import("vue")>()),
  onActivated: (callback: () => void) => hooks.activated.push(callback),
  onDeactivated: (callback: () => void) => hooks.deactivated.push(callback),
}));
import { effectScope, nextTick, ref } from "vue";
import { useConfigReload } from "./useConfigReload";
it("refreshes cached pages on reactivation without fetching inactive pages or duplicating mount loads", async () => {
  const revision = ref(0);
  const reload = vi.fn();
  const scope = effectScope();
  scope.run(() => useConfigReload(() => revision.value, reload));
  hooks.activated[0]();
  expect(reload).not.toHaveBeenCalled();
  revision.value++;
  await nextTick();
  expect(reload).toHaveBeenCalledTimes(1);
  hooks.deactivated[0]();
  revision.value++;
  await nextTick();
  expect(reload).toHaveBeenCalledTimes(1);
  hooks.activated[0]();
  expect(reload).toHaveBeenCalledTimes(2);
  scope.stop();
});
