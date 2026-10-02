import { expect, it, vi } from "vitest";
import { useLuaScriptEditor } from "./useLuaScriptEditor";
import type { LuaPluginInfo } from "../types";
const plugin = (id: string) =>
  ({ id, name: id, file_name: `${id}.lua`, installed: true }) as LuaPluginInfo;
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}
it("keeps script contents and dialog title from the same completed request", async () => {
  const a = deferred<string>();
  const b = deferred<string>();
  const write = vi.fn().mockResolvedValue(true);
  const editor = useLuaScriptEditor((id) => (id === "a" ? a.promise : b.promise), write, vi.fn());
  const first = editor.open(plugin("a"));
  const second = editor.open(plugin("b"));
  b.resolve("B script");
  await second;
  a.resolve("A script");
  await first;
  expect(editor.editingPlugin.value?.id).toBe("b");
  expect(editor.content.value).toBe("B script");
  editor.content.value = "edited B";
  await editor.save();
  expect(write).toHaveBeenCalledExactlyOnceWith("b", "edited B", { content: "B script" });
});
it("does not expose a stale script under a new title or report failed saves as successful", async () => {
  const notify = vi.fn();
  const write = vi.fn().mockResolvedValue(undefined);
  const editor = useLuaScriptEditor(async (id) => (id === "a" ? "A" : undefined), write, notify);
  await editor.open(plugin("a"));
  editor.content.value = "edited A";
  await editor.open(plugin("b"));
  expect(editor.editingPlugin.value?.id).toBe("a");
  await editor.save();
  expect(notify).not.toHaveBeenCalled();
  expect(editor.visible.value).toBe(true);
  expect(editor.dirty.value).toBe(true);
  expect(write).toHaveBeenCalledExactlyOnceWith("a", "edited A", { content: "A" });
});
it("keeps the editor open if more edits arrive while a save is pending", async () => {
  const result = deferred<boolean>();
  const notify = vi.fn();
  const editor = useLuaScriptEditor(
    async () => "initial",
    () => result.promise,
    notify,
  );
  await editor.open(plugin("a"));
  editor.content.value = "first edit";
  const saving = editor.save();
  editor.content.value = "second edit";
  result.resolve(true);
  await saving;
  expect(editor.visible.value).toBe(true);
  expect(editor.dirty.value).toBe(true);
  expect(notify).toHaveBeenCalledTimes(1);
});
