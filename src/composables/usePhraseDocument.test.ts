import { describe, expect, it, vi } from "vitest";
import { usePhraseDocument } from "./usePhraseDocument";
import type { PhraseEntry } from "../types";
const phrase = (text: string): PhraseEntry => ({ text, code: "a", weight: 1 });
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

describe("phrase document", () => {
  it("undoes unsaved changes to the last successful snapshot without creating another backup", async () => {
    const pending = deferred<boolean>();
    const write = vi.fn(() => pending.promise);
    const read = vi.fn(async () => [phrase("a")]);
    const document = usePhraseDocument(read, write);
    expect(document.reset()).toBe(false);
    await document.load();
    document.entries.value[0].text = "saved";
    const saving = document.save();
    expect(document.reset()).toBe(false);
    pending.resolve(true);
    await saving;
    document.entries.value.push(phrase("draft"));
    document.entries.value[0].text = "changed";
    expect(document.reset()).toBe(true);
    expect(document.entries.value).toEqual([phrase("saved")]);
    expect(document.dirty.value).toBe(false);
    expect(read).toHaveBeenCalledTimes(1);
    expect(write).toHaveBeenCalledTimes(1);
  });
  it("blocks saving before initial load and after failed initial reads", async () => {
    const write = vi.fn();
    const document = usePhraseDocument(async () => undefined, write);
    expect(await document.save()).toBe(false);
    await document.load();
    expect(await document.save()).toBe(false);
    expect(write).not.toHaveBeenCalled();
  });
  it("keeps edits and the previous document after a failed refresh", async () => {
    const read = vi
      .fn()
      .mockResolvedValueOnce([phrase("a")])
      .mockResolvedValueOnce(undefined);
    const document = usePhraseDocument(read, async () => true);
    await document.load();
    document.entries.value[0].text = "edited";
    await document.load();
    expect(document.entries.value[0].text).toBe("edited");
    expect(document.dirty.value).toBe(true);
  });
  it("saves a snapshot, blocks double clicks, and keeps edits made during saving dirty", async () => {
    const result = deferred<boolean>();
    const write = vi.fn(() => result.promise);
    const document = usePhraseDocument(async () => [phrase("a")], write);
    await document.load();
    const first = document.save();
    document.entries.value[0].text = "new edit";
    expect(await document.save()).toBe(false);
    expect(await document.load()).toBe(false);
    result.resolve(true);
    await first;
    expect(write).toHaveBeenCalledExactlyOnceWith([phrase("a")], { content: null });
    expect(document.dirty.value).toBe(true);
  });
  it("ignores late refreshes and pending reads canceled during unmount", async () => {
    const a = deferred<PhraseEntry[]>();
    const b = deferred<PhraseEntry[]>();
    const document = usePhraseDocument(
      vi.fn().mockReturnValueOnce(a.promise).mockReturnValueOnce(b.promise),
      vi.fn(),
    );
    const first = document.load();
    const second = document.load();
    b.resolve([phrase("b")]);
    await second;
    a.resolve([phrase("a")]);
    await first;
    expect(document.entries.value[0].text).toBe("b");
    const c = deferred<PhraseEntry[]>();
    const canceled = usePhraseDocument(() => c.promise, vi.fn());
    const loading = canceled.load();
    canceled.cancelLoad();
    c.resolve([phrase("c")]);
    await loading;
    expect(canceled.ready.value).toBe(false);
  });
});

it("reloads external phrases without reporting a save or deployment", async () => {
  const document = usePhraseDocument(
    async () => [phrase("old")],
    async () => ({ reload: [phrase("external")], revision: { content: "external" } }),
  );
  await document.load();
  document.entries.value = [phrase("draft")];
  expect(await document.save()).toBe(false);
  expect(document.entries.value).toEqual([phrase("external")]);
  expect(document.dirty.value).toBe(false);
});

it("commits phrase rows and their revision from the same accepted read", async () => {
  type Snapshot = { entries: PhraseEntry[]; revision: { content: string } };
  const first = deferred<Snapshot>();
  const second = deferred<Snapshot>();
  const write = vi.fn(async () => true);
  const document = usePhraseDocument(
    vi.fn().mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise),
    write,
  );
  const a = document.load();
  const b = document.load();
  second.resolve({ entries: [phrase("new")], revision: { content: "new revision" } });
  await b;
  first.resolve({ entries: [phrase("old")], revision: { content: "old revision" } });
  await a;
  await document.save();
  expect(write).toHaveBeenCalledExactlyOnceWith([phrase("new")], { content: "new revision" });
});
