import { expect, it, vi } from "vitest";
import { reactive } from "vue";
import { useSettingsDocument } from "./useSettingsDocument";
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}
const state = () => reactive({ theme: "old", pairs: ["a"] });
it("blocks default-value saves after an initial read failure", async () => {
  const form = state();
  const write = vi.fn();
  const doc = useSettingsDocument(
    () => ({ ...form }),
    (value) => Object.assign(form, value),
    async () => undefined,
    write,
  );
  await doc.load();
  expect(await doc.save()).toBe(false);
  expect(write).not.toHaveBeenCalled();
  expect(doc.ready.value).toBe(false);
});
it("keeps unsaved drafts during refreshes and applies clean external updates", async () => {
  const form = state();
  const read = vi
    .fn()
    .mockResolvedValueOnce({ theme: "saved", pairs: ["a"] })
    .mockResolvedValueOnce({ theme: "external", pairs: ["b"] });
  const doc = useSettingsDocument(
    () => ({ ...form }),
    (value) => Object.assign(form, value),
    read,
    vi.fn(),
  );
  await doc.load();
  await doc.load();
  expect(form.theme).toBe("external");
  form.theme = "draft";
  expect(await doc.load()).toBe(false);
  expect(form.theme).toBe("draft");
  expect(read).toHaveBeenCalledTimes(2);
});
it("deeply snapshots array settings and preserves edits made during a save", async () => {
  const form = state();
  const result = deferred<typeof form>();
  const write = vi.fn(() => result.promise);
  const doc = useSettingsDocument(
    () => ({ ...form }),
    (value) => Object.assign(form, value),
    async () => state(),
    write,
  );
  await doc.load();
  const saving = doc.save();
  form.pairs.push("b");
  form.theme = "late edit";
  expect(await doc.save()).toBe(false);
  expect(await doc.load()).toBe(false);
  result.resolve({ theme: "old", pairs: ["a"] });
  await saving;
  expect(write).toHaveBeenCalledExactlyOnceWith({ theme: "old", pairs: ["a"] });
  expect(form.theme).toBe("late edit");
  expect(doc.dirty.value).toBe(true);
});
it("recognizes server normalization independently of JSON property order", async () => {
  const form = state();
  const doc = useSettingsDocument(
    () => ({ ...form }),
    (value) => Object.assign(form, value),
    async () => state(),
    async () => ({ pairs: ["a"], theme: "normalized" }),
  );
  await doc.load();
  await doc.save();
  expect(form.theme).toBe("normalized");
  expect(doc.dirty.value).toBe(false);
});
it("retains the draft on failed saves and ignores disposed reads", async () => {
  const form = state();
  const doc = useSettingsDocument(
    () => ({ ...form }),
    (value) => Object.assign(form, value),
    async () => state(),
    async () => undefined,
  );
  await doc.load();
  form.theme = "draft";
  expect(await doc.save()).toBe(false);
  expect(doc.dirty.value).toBe(true);
  expect(doc.saving.value).toBe(false);
  const result = deferred<typeof form>();
  const canceled = useSettingsDocument(
    () => ({ ...form }),
    (value) => Object.assign(form, value),
    () => result.promise,
    vi.fn(),
  );
  const loading = canceled.load();
  canceled.dispose();
  result.resolve({ theme: "disposed", pairs: [] });
  await loading;
  expect(form.theme).toBe("draft");
  expect(canceled.ready.value).toBe(false);
});
