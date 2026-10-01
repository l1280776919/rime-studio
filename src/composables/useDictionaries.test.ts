import { beforeEach, describe, expect, it, vi } from "vitest";
import type {
  DictHealth,
  DictInfo,
  DictionaryImportPreview,
  OnlineDictionary,
  DictionaryConfig,
} from "../types";

const mocks = vi.hoisted(() => ({
  api: {
    getDictHealth: vi.fn(),
    deleteDictionary: vi.fn(),
    installLmdgDicts: vi.fn(),
    installLmdgGrammar: vi.fn(),
    listOnlineDictionariesByCategory: vi.fn(),
    previewDictionaryUrlImport: vi.fn(),
    importDictionaryUrl: vi.fn(),
    previewDictionaryImport: vi.fn(),
    importDictionary: vi.fn(),
    listDictionaries: vi.fn(),
    getDictionaryConfig: vi.fn(),
    addDictionaryToCurrentSchema: vi.fn(),
    saveDictionaryImports: vi.fn(),
    listOnlineDictionaries: vi.fn(),
    listOnlineDictionaryCategories: vi.fn(),
  },
  confirm: vi.fn(),
  success: vi.fn(),
  error: vi.fn(),
  warning: vi.fn(),
  listen: vi.fn(),
  mounted: [] as (() => void)[],
  deactivated: [] as (() => void)[],
  unmounted: [] as (() => void)[],
}));
vi.mock("../api", () => ({ api: mocks.api }));
vi.mock("element-plus", () => ({
  ElMessage: { success: mocks.success, error: mocks.error, warning: mocks.warning },
  ElMessageBox: { confirm: mocks.confirm },
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));
vi.mock("vue", async (importOriginal) => ({
  ...(await importOriginal<typeof import("vue")>()),
  onDeactivated: (callback: () => void) => mocks.deactivated.push(callback),
  onMounted: (callback: () => void) => mocks.mounted.push(callback),
  onUnmounted: (callback: () => void) => mocks.unmounted.push(callback),
}));
import { useDictionaries } from "./useDictionaries";

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}
const dict = (name: string) =>
  ({ name, path: name, entry_count: 1, size_bytes: 1 }) satisfies DictInfo;
const health = (name: string) =>
  ({
    entries: name === "a" ? 1 : 2,
    duplicate_exact_lines: 1,
    long_low_weight_entries: 0,
  }) satisfies DictHealth;
const online = (id: string) =>
  ({ id, source_name: `${id}.txt`, detail_url: `https://example.com/${id}` }) as OnlineDictionary;
const preview = (name: string) =>
  ({ name, imported_entries: 1, skipped_entries: 0 }) as DictionaryImportPreview;
const config = (): DictionaryConfig => ({
  imports: ["missing", "present"],
  enabled: [{ reference: "present", exists: true }],
  missing: [{ reference: "missing", exists: false }],
  available: [],
});
const tick = async () => {
  await Promise.resolve();
  await Promise.resolve();
  await Promise.resolve();
};

beforeEach(() => {
  vi.resetAllMocks();
  mocks.mounted.length = 0;
  mocks.deactivated.length = 0;
  mocks.unmounted.length = 0;
  mocks.api.listDictionaries.mockResolvedValue([]);
  mocks.api.getDictionaryConfig.mockResolvedValue(config());
  mocks.api.listOnlineDictionaries.mockResolvedValue([]);
  mocks.api.listOnlineDictionaryCategories.mockResolvedValue([]);
  mocks.api.listOnlineDictionariesByCategory.mockResolvedValue([]);
  mocks.listen.mockResolvedValue(vi.fn());
});

describe("dictionary interaction flows", () => {
  it("keeps health results attached to the last expanded dictionary", async () => {
    const a = deferred<DictHealth>();
    const b = deferred<DictHealth>();
    mocks.api.getDictHealth.mockImplementation((name) => (name === "a" ? a.promise : b.promise));
    const state = useDictionaries(vi.fn());
    const first = state.toggleHealth(dict("a"));
    const second = state.toggleHealth(dict("b"));
    a.resolve(health("a"));
    await first;
    expect(state.healthLoading.value).toBe(true);
    expect(state.dictHealth.value).toBeNull();
    b.resolve(health("b"));
    await second;
    expect(state.dictHealth.value?.entries).toBe(2);
  });

  it("does not reopen health data after a row is collapsed", async () => {
    const result = deferred<DictHealth>();
    mocks.api.getDictHealth.mockReturnValue(result.promise);
    const state = useDictionaries(vi.fn());
    const pending = state.toggleHealth(dict("a"));
    await state.toggleHealth(dict("a"));
    result.resolve(health("a"));
    await pending;
    expect(state.dictHealth.value).toBeNull();
    expect(state.healthLoading.value).toBe(false);
  });

  it("ignores old category requests and clears rows while the new category loads", async () => {
    const a = deferred<OnlineDictionary[]>();
    const b = deferred<OnlineDictionary[]>();
    mocks.api.listOnlineDictionariesByCategory.mockImplementation((id) =>
      id === "a" ? a.promise : b.promise,
    );
    const state = useDictionaries(vi.fn());
    const first = state.selectOnlineCategory("a");
    const second = state.selectOnlineCategory("b");
    a.resolve([online("a")]);
    await first;
    expect(state.categoryDictionaries.value).toEqual([]);
    expect(state.categoryLoading.value).toBe(true);
    b.resolve([online("b")]);
    await second;
    expect(state.categoryDictionaries.value[0].id).toBe("b");
  });

  it("imports exactly the source that was previewed, even if the URL draft changes", async () => {
    mocks.api.previewDictionaryUrlImport.mockResolvedValue(preview("a"));
    mocks.api.importDictionaryUrl.mockResolvedValue({
      imported_entries: 1,
      skipped_entries: 0,
      name: "a",
      reference: "a",
    });
    const state = useDictionaries(vi.fn());
    state.importUrl.value = " https://example.com/a ";
    state.importUrlSourceName.value = " a.txt ";
    await state.previewUrlDictionary();
    state.importUrl.value = "https://example.com/b";
    state.importUrlSourceName.value = "b.txt";
    await state.confirmDictionaryImport();
    expect(mocks.api.importDictionaryUrl).toHaveBeenCalledExactlyOnceWith(
      "https://example.com/a",
      "a.txt",
    );
  });

  it("commits the latest preview together with its source when responses arrive out of order", async () => {
    const a = deferred<DictionaryImportPreview>();
    const b = deferred<DictionaryImportPreview>();
    mocks.api.previewDictionaryUrlImport.mockImplementation((url) =>
      url.endsWith("/a") ? a.promise : b.promise,
    );
    mocks.api.importDictionaryUrl.mockResolvedValue({
      imported_entries: 1,
      skipped_entries: 0,
      name: "b",
      reference: "b",
    });
    const state = useDictionaries(vi.fn());
    const first = state.previewOnlineDictionary(online("a"));
    const second = state.previewOnlineDictionary(online("b"));
    b.resolve(preview("b"));
    await second;
    a.resolve(preview("a"));
    await first;
    expect(state.importPreview.value?.name).toBe("b");
    await state.confirmDictionaryImport();
    expect(mocks.api.importDictionaryUrl).toHaveBeenCalledExactlyOnceWith(
      "https://example.com/b",
      "b.txt",
    );
  });

  it("cancels a pending URL preview when its dialog closes", async () => {
    const result = deferred<DictionaryImportPreview>();
    mocks.api.previewDictionaryUrlImport.mockReturnValue(result.promise);
    const state = useDictionaries(vi.fn());
    state.showUrlImportDialog.value = true;
    state.importUrl.value = "https://example.com/a";
    const pending = state.previewUrlDictionary();
    state.showUrlImportDialog.value = false;
    result.resolve(preview("a"));
    await pending;
    expect(state.showImportPreviewDialog.value).toBe(false);
    expect(state.importPreview.value).toBeUndefined();
    expect(state.importing.value).toBe(false);
  });

  it("prevents double imports and reports a partial import-and-enable failure", async () => {
    const result = deferred<{ imported_entries: number; name: string; reference: string }>();
    mocks.api.previewDictionaryUrlImport.mockResolvedValue(preview("a"));
    mocks.api.importDictionaryUrl.mockReturnValue(result.promise);
    mocks.api.addDictionaryToCurrentSchema.mockRejectedValue(new Error("cannot enable"));
    const state = useDictionaries(vi.fn());
    state.importUrl.value = "https://example.com/a";
    await state.previewUrlDictionary();
    const first = state.confirmDictionaryImport(true);
    await state.confirmDictionaryImport(true);
    result.resolve({ imported_entries: 1, name: "a", reference: "a" });
    await first;
    expect(mocks.api.importDictionaryUrl).toHaveBeenCalledTimes(1);
    expect(mocks.warning).toHaveBeenCalledWith(expect.stringContaining("未能加入当前方案"));
  });

  it("displays missing references in their original priority and serializes reorder writes", async () => {
    const result = deferred<DictionaryConfig>();
    mocks.api.saveDictionaryImports.mockReturnValue(result.promise);
    const state = useDictionaries(vi.fn());
    state.dictConfig.value = config();
    expect(state.orderedReferences.value.map((entry) => entry.reference)).toEqual([
      "missing",
      "present",
    ]);
    const first = state.moveReference("present", -1);
    await state.moveReference("missing", 1);
    expect(mocks.api.saveDictionaryImports).toHaveBeenCalledExactlyOnceWith(["present", "missing"]);
    result.resolve(config());
    await first;
  });

  it("loads initial lists even when progress subscription fails", async () => {
    mocks.listen.mockRejectedValue(new Error("no event service"));
    useDictionaries(vi.fn());
    mocks.mounted[0]();
    await tick();
    expect(mocks.api.listDictionaries).toHaveBeenCalledTimes(1);
    expect(mocks.api.listOnlineDictionaries).toHaveBeenCalledTimes(1);
  });

  it("releases an event subscription that resolves after unmount", async () => {
    const result = deferred<() => void>();
    mocks.listen.mockReturnValue(result.promise);
    useDictionaries(vi.fn());
    mocks.mounted[0]();
    mocks.unmounted[0]();
    const unlisten = vi.fn();
    result.resolve(unlisten);
    await tick();
    expect(unlisten).toHaveBeenCalledTimes(1);
  });
});

it("guards repeated delete clicks while confirmation is open and resets on cancel", async () => {
  const dialog = deferred<boolean>();
  mocks.confirm.mockReturnValue(dialog.promise);
  mocks.api.deleteDictionary.mockResolvedValue(undefined);
  const state = useDictionaries(vi.fn());
  const first = state.deleteDictionary(dict("a"));
  await state.deleteDictionary(dict("a"));
  expect(mocks.confirm).toHaveBeenCalledTimes(1);
  dialog.resolve(true);
  await first;
  expect(mocks.api.deleteDictionary).toHaveBeenCalledTimes(1);
  mocks.confirm.mockRejectedValue(new Error("cancel"));
  await state.deleteDictionary(dict("b"));
  expect(state.deletingDict.value).toBeUndefined();
  expect(mocks.api.deleteDictionary).toHaveBeenCalledTimes(1);
});

it("does not start another resource operation while installation is running", async () => {
  const result = deferred<{ message: string }>();
  mocks.api.installLmdgDicts.mockReturnValue(result.promise);
  const state = useDictionaries(vi.fn());
  const first = state.installLmdgDictionaries();
  await state.installLmdgGrammar();
  expect(mocks.api.installLmdgGrammar).not.toHaveBeenCalled();
  result.resolve({ message: "installed" });
  await first;
  expect(state.lmdgInstalling.value).toBe(false);
});

it("does not open a late import preview after its cached page is deactivated", async () => {
  const result = deferred<DictionaryImportPreview>();
  mocks.api.previewDictionaryUrlImport.mockReturnValue(result.promise);
  const state = useDictionaries(vi.fn());
  const pending = state.previewOnlineDictionary(online("a"));
  mocks.deactivated[0]();
  result.resolve(preview("a"));
  await pending;
  expect(state.showImportPreviewDialog.value).toBe(false);
  expect(state.importPreview.value).toBeUndefined();
});
