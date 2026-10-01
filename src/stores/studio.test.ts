import { beforeEach, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import type { RimeEnvironment, DeployResult, BackupEntry } from "../types";
const mocks = vi.hoisted(() => ({
  api: {
    scanEnvironment: vi.fn(),
    listBackups: vi.fn(),
    scanDictionaryHealth: vi.fn(),
    deploy: vi.fn(),
    cancelDeploy: vi.fn(),
    installRimeIce: vi.fn(),
    createBackup: vi.fn(),
    restoreBackup: vi.fn(),
    deleteBackup: vi.fn(),
  },
  message: vi.fn(),
  success: vi.fn(),
  error: vi.fn(),
  warning: vi.fn(),
  confirm: vi.fn(),
  prompt: vi.fn(),
  listen: vi.fn(),
}));
vi.mock("../api", () => ({ api: mocks.api }));
vi.mock("element-plus", () => ({
  ElMessage: Object.assign(mocks.message, {
    success: mocks.success,
    error: mocks.error,
    warning: mocks.warning,
  }),
  ElMessageBox: { confirm: mocks.confirm, prompt: mocks.prompt },
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));
import { useStudioStore } from "./studio";
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}
const env = (name: string) => ({ user_dir: name }) as RimeEnvironment;
const deployed = (success = true) =>
  ({ success, message: success ? "deployed" : "deployment failed" }) as DeployResult;
const backup = (name = "backup") => ({ name, files: 1 }) as BackupEntry;
beforeEach(() => {
  vi.resetAllMocks();
  setActivePinia(createPinia());
  mocks.api.scanEnvironment.mockResolvedValue(env("current"));
  mocks.api.listBackups.mockResolvedValue([]);
  mocks.api.scanDictionaryHealth.mockResolvedValue(undefined);
  mocks.listen.mockResolvedValue(vi.fn());
});
it("keeps the latest scan and its busy state when responses are reordered", async () => {
  const a = deferred<RimeEnvironment>();
  const b = deferred<RimeEnvironment>();
  mocks.api.scanEnvironment.mockReturnValueOnce(a.promise).mockReturnValueOnce(b.promise);
  const store = useStudioStore();
  const first = store.loadEnvironment();
  const second = store.loadEnvironment();
  a.resolve(env("old"));
  await first;
  expect(store.scanning).toBe(true);
  expect(store.env).toBeUndefined();
  b.resolve(env("new"));
  await second;
  expect(store.env?.user_dir).toBe("new");
  expect(store.scanning).toBe(false);
});
it("does not attach a previous scan's health report to a new environment", async () => {
  const a = deferred<unknown>();
  const b = deferred<unknown>();
  mocks.api.scanDictionaryHealth.mockReturnValueOnce(a.promise).mockReturnValueOnce(b.promise);
  const store = useStudioStore();
  store.env = env("old");
  const first = store.loadDictionaryHealth();
  store.env = env("new");
  const second = store.loadDictionaryHealth();
  a.resolve({ source: "old" });
  await first;
  expect(store.env?.sogou_health).toBeUndefined();
  expect(store.detailsLoading).toBe(true);
  b.resolve({ source: "new" });
  await second;
  expect(store.env?.sogou_health).toEqual({ source: "new" });
});
it("deploys even if progress subscription fails and keeps the result status after scanning", async () => {
  mocks.listen.mockRejectedValue(new Error("no events"));
  mocks.api.deploy.mockResolvedValue(deployed());
  const store = useStudioStore();
  await store.deploy();
  expect(mocks.api.deploy).toHaveBeenCalledTimes(1);
  expect(store.status).toBe("deployed");
  expect(store.deploying).toBe(false);
});
it("prevents duplicate deployments and excludes restoration while deployment is pending", async () => {
  const result = deferred<DeployResult>();
  mocks.api.deploy.mockReturnValue(result.promise);
  const store = useStudioStore();
  const first = store.deploy();
  await Promise.resolve();
  await store.deploy();
  await store.restoreBackup(backup());
  expect(mocks.confirm).not.toHaveBeenCalled();
  result.resolve(deployed());
  await first;
  expect(mocks.api.deploy).toHaveBeenCalledTimes(1);
});
it("preserves deployment failures and clears a previous successful result on a new failure", async () => {
  const store = useStudioStore();
  mocks.api.deploy.mockResolvedValueOnce(deployed(false));
  await store.deploy();
  expect(store.status).toBe("deployment failed");
  store.lastDeploy = deployed();
  mocks.api.deploy.mockRejectedValue(new Error("cannot deploy"));
  await expect(store.deploy()).resolves.toBeUndefined();
  expect(store.lastDeploy).toEqual({
    success: false,
    message: "Error: cannot deploy",
    log: "Error: cannot deploy",
  });
  await store.loadEnvironment();
  expect(store.lastDeploy?.success).toBe(false);
  expect(store.deploying).toBe(false);
});
it("a backup-list refresh failure does not turn a successful backup into a failure", async () => {
  mocks.prompt.mockResolvedValue({ value: "note" });
  mocks.api.createBackup.mockResolvedValue(backup());
  mocks.api.listBackups.mockRejectedValue(new Error("refresh failed"));
  const store = useStudioStore();
  expect(await store.createManualBackup()).toEqual(backup());
  expect(mocks.success).toHaveBeenCalled();
  expect(mocks.warning).toHaveBeenCalled();
  expect(store.backingUp).toBe(false);
});
it("guards repeated restore clicks while confirmation is pending and resets after cancel", async () => {
  const dialog = deferred<boolean>();
  mocks.confirm.mockReturnValue(dialog.promise);
  mocks.api.restoreBackup.mockResolvedValue({ restored_files: 1 });
  const store = useStudioStore();
  const first = store.restoreBackup(backup());
  await store.restoreBackup(backup());
  expect(mocks.confirm).toHaveBeenCalledTimes(1);
  dialog.resolve(true);
  await first;
  expect(mocks.api.restoreBackup).toHaveBeenCalledTimes(1);
  mocks.confirm.mockRejectedValue(new Error("cancel"));
  await store.restoreBackup(backup());
  expect(store.restoringBackup).toBeUndefined();
});
it("guards deletion confirmations and releases state after cancellation", async () => {
  const dialog = deferred<boolean>();
  mocks.confirm.mockReturnValue(dialog.promise);
  const store = useStudioStore();
  const first = store.deleteBackupEntry(backup());
  await store.deleteBackupEntry(backup());
  expect(mocks.confirm).toHaveBeenCalledTimes(1);
  dialog.resolve(true);
  mocks.api.deleteBackup.mockResolvedValue(undefined);
  await first;
  expect(mocks.api.deleteBackup).toHaveBeenCalledTimes(1);
  expect(store.deletingBackup).toBeUndefined();
  mocks.confirm.mockRejectedValue(new Error("cancel"));
  await store.deleteBackupEntry(backup());
  expect(store.deletingBackup).toBeUndefined();
});
