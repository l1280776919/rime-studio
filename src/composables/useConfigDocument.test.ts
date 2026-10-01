import { describe, expect, it, vi } from "vitest";
import { useConfigDocument } from "./useConfigDocument";
import type { FileStatus } from "../types";

const file = (name: string): FileStatus => ({ name, path: name, exists: true });
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => (resolve = done));
  return { promise, resolve };
}

describe("configuration document", () => {
  it("ignores late reads when files are switched quickly", async () => {
    const a = deferred<string>();
    const b = deferred<string>();
    const doc = useConfigDocument((name) => (name === "a" ? a.promise : b.promise), vi.fn());
    const first = doc.load(file("a"));
    const second = doc.load(file("b"));
    b.resolve("B");
    expect(await second).toBe(true);
    a.resolve("A");
    expect(await first).toBe(false);
    expect(doc.selectedFile.value?.name).toBe("b");
    expect(doc.content.value).toBe("B");
  });

  it("keeps the previous file and unsaved contents when a read fails", async () => {
    const write = vi.fn().mockResolvedValue(true);
    const doc = useConfigDocument(async (name) => (name === "a" ? "A" : undefined), write);
    await doc.load(file("a"));
    doc.content.value = "edited A";
    expect(await doc.load(file("b"))).toBe(false);
    expect(doc.selectedFile.value?.name).toBe("a");
    expect(doc.dirty.value).toBe(true);
    await doc.save();
    expect(write).toHaveBeenCalledWith("a", "edited A");
  });

  it("blocks saves during reads and duplicate saves, retaining edits made during a save", async () => {
    const read = deferred<string>();
    const write = deferred<boolean>();
    const writer = vi.fn(() => write.promise);
    const doc = useConfigDocument(() => read.promise, writer);
    const loading = doc.load(file("a"));
    expect(await doc.save()).toBe(false);
    read.resolve("A");
    await loading;
    doc.content.value = "first edit";
    const saving = doc.save();
    expect(await doc.save()).toBe(false);
    expect(await doc.load(file("b"))).toBe(false);
    doc.content.value = "second edit";
    write.resolve(true);
    expect(await saving).toBe(true);
    expect(doc.dirty.value).toBe(true);
    expect(writer).toHaveBeenCalledExactlyOnceWith("a", "first edit");
  });

  it("retains dirty state on failed saves and resets busy state on rejected calls", async () => {
    const doc = useConfigDocument(
      async () => "A",
      async () => false,
    );
    await doc.load(file("a"));
    doc.content.value = "edit";
    expect(await doc.save()).toBe(false);
    expect(doc.dirty.value).toBe(true);
    expect(doc.saving.value).toBe(false);
    const broken = useConfigDocument(async () => {
      throw new Error("read failed");
    }, vi.fn());
    await expect(broken.load(file("b"))).rejects.toThrow("read failed");
    expect(broken.loading.value).toBe(false);
  });

  it("does not commit pending reads after the editor is destroyed", async () => {
    const read = deferred<string>();
    const doc = useConfigDocument(() => read.promise, vi.fn());
    const loading = doc.load(file("a"));
    doc.cancelLoad();
    read.resolve("A");
    expect(await loading).toBe(false);
    expect(await doc.load(file("b"))).toBe(false);
    expect(doc.selectedFile.value).toBeNull();
  });
});
