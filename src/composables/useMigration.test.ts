import { beforeEach, expect, it, vi } from "vitest";
import type { MigrationPreview } from "../migration/types";
const mocks = vi.hoisted(() => ({
  previewMigration: vi.fn(),
  importMigration: vi.fn(),
  discardMigrationPreview: vi.fn(),
}));
vi.mock("../api", () => ({ api: mocks }));
import { useMigration } from "./useMigration";

const preview = (token: string, selected = true): MigrationPreview => ({
  token,
  app_version: "0.11.0",
  created_at: "1",
  blockers: [],
  files: [{ name: "a.yaml", category: "config", bytes: 2, status: "new", selected, diff: ["+ a"] }],
});
const file = {
  name: "migration.zip",
  size: 2,
  arrayBuffer: async () => new Uint8Array([1, 2]).buffer,
};
const perform = <T>(task: () => Promise<T>) => task();
beforeEach(() => {
  vi.resetAllMocks();
  mocks.discardMigrationPreview.mockResolvedValue(undefined);
});

it("requires a reviewed selection and consumes the token exactly once", async () => {
  mocks.previewMigration
    .mockResolvedValueOnce(preview("catalog"))
    .mockResolvedValueOnce(preview("reviewed"));
  mocks.importMigration.mockResolvedValue({ imported_files: 1, safety_backup_dir: "backup" });
  const migration = useMigration(perform);
  await migration.load(file);
  expect(migration.canImport.value).toBe(false);
  await migration.review();
  expect(migration.canImport.value).toBe(true);
  migration.selected.value = [];
  expect(migration.canImport.value).toBe(false);
  migration.selected.value = ["a.yaml"];
  await migration.importSelected();
  await migration.importSelected();
  expect(mocks.importMigration).toHaveBeenCalledExactlyOnceWith("reviewed");
  expect(migration.result.value?.imported_files).toBe(1);
});

it("invalidates old previews when a new archive fails to load", async () => {
  mocks.previewMigration
    .mockResolvedValueOnce(preview("old"))
    .mockRejectedValueOnce(new Error("bad zip"));
  const migration = useMigration(perform);
  await migration.load(file);
  await expect(migration.load(file)).rejects.toThrow("bad zip");
  expect(migration.preview.value).toBeUndefined();
  expect(migration.canImport.value).toBe(false);
  expect(mocks.discardMigrationPreview).toHaveBeenCalledWith("old");
});

it("blocks missing dependencies and requires re-preview after an import failure", async () => {
  mocks.previewMigration.mockResolvedValue(preview("first"));
  const migration = useMigration(perform);
  await migration.load(file);
  mocks.previewMigration.mockResolvedValue({ ...preview("blocked"), blockers: ["missing schema"] });
  await migration.review();
  expect(migration.canImport.value).toBe(false);
  mocks.previewMigration.mockResolvedValue(preview("valid"));
  await migration.review();
  mocks.importMigration.mockRejectedValue(new Error("external change"));
  await expect(migration.importSelected()).rejects.toThrow("external change");
  expect(migration.canImport.value).toBe(false);
  expect(migration.preview.value?.files).toHaveLength(1);
});

it("rejects oversized archives before reading or invoking the backend", async () => {
  const migration = useMigration(perform);
  const arrayBuffer = vi.fn();
  await expect(migration.load({ ...file, size: 65 * 1024 * 1024, arrayBuffer })).rejects.toThrow(
    "64 MiB",
  );
  expect(arrayBuffer).not.toHaveBeenCalled();
  expect(mocks.previewMigration).not.toHaveBeenCalled();
});
