import { expect, it, vi } from "vitest";
import { effectScope, nextTick, ref } from "vue";
import { useThemePreview } from "./useThemePreview";

function setup(initialReady = true, initialDirty = false) {
  const request = ref<string | undefined>("blue");
  const ready = ref(initialReady);
  const dirty = ref(initialDirty);
  const apply = vi.fn();
  const confirm = vi.fn(async () => true);
  const clear = vi.fn(async (name: string) => {
    if (request.value === name) request.value = undefined;
  });
  const scope = effectScope();
  const start = () =>
    scope.run(() =>
      useThemePreview(
        () => request.value,
        () => ready.value,
        () => dirty.value,
        apply,
        confirm,
        clear,
      ),
    );
  return { request, ready, apply, confirm, clear, scope, start };
}
it("waits for loaded settings before previewing and consumes the request once", async () => {
  const state = setup(false);
  state.start();
  expect(state.apply).not.toHaveBeenCalled();
  expect(state.request.value).toBe("blue");
  state.ready.value = true;
  await nextTick();
  await nextTick();
  expect(state.apply).toHaveBeenCalledExactlyOnceWith("blue");
  expect(state.confirm).not.toHaveBeenCalled();
  expect(state.request.value).toBeUndefined();
  state.scope.stop();
});
it("keeps a dirty draft when preview replacement is canceled", async () => {
  const state = setup(true, true);
  state.confirm.mockResolvedValue(false);
  state.start();
  await nextTick();
  await nextTick();
  expect(state.apply).not.toHaveBeenCalled();
  expect(state.clear).toHaveBeenCalledExactlyOnceWith("blue");
  state.scope.stop();
});
it("does not apply a stale confirmation and handles the newest request", async () => {
  const state = setup(true, true);
  let resolve!: (accepted: boolean) => void;
  state.confirm.mockReturnValueOnce(
    new Promise<boolean>((done) => {
      resolve = done;
    }),
  );
  state.start();
  state.request.value = "green";
  await nextTick();
  resolve(true);
  await nextTick();
  await nextTick();
  expect(state.apply).toHaveBeenCalledExactlyOnceWith("green");
  state.scope.stop();
});
it("ignores a pending confirmation after the page is destroyed", async () => {
  const state = setup(true, true);
  let resolve!: (accepted: boolean) => void;
  state.confirm.mockReturnValueOnce(
    new Promise<boolean>((done) => {
      resolve = done;
    }),
  );
  state.start();
  state.scope.stop();
  resolve(true);
  await nextTick();
  expect(state.apply).not.toHaveBeenCalled();
  expect(state.clear).not.toHaveBeenCalled();
});
