import { computed, ref, shallowRef } from "vue";
import { api } from "../api";
import {
  migrationSelectionKey,
  type MigrationPreview,
  type MigrationResult,
} from "../migration/types";

/** The UI cannot submit a plan for a different file selection or reuse a consumed token. */
export function useMigration(perform: <T>(task: () => Promise<T>) => Promise<T>) {
  const preview = shallowRef<MigrationPreview>();
  const selected = ref<string[]>([]);
  const reviewedSelection = ref("");
  const result = shallowRef<MigrationResult>();
  const filename = ref("");
  const busy = ref(false);
  const reviewed = ref(false);
  let bytes: number[] | undefined;
  const canImport = computed(
    () =>
      !busy.value &&
      reviewed.value &&
      !!preview.value?.token &&
      selected.value.length > 0 &&
      preview.value.blockers.length === 0 &&
      reviewedSelection.value === migrationSelectionKey(selected.value),
  );

  function acceptPreview(value: MigrationPreview, ready: boolean) {
    preview.value = value;
    selected.value = value.files.filter((file) => file.selected).map((file) => file.name);
    reviewedSelection.value = migrationSelectionKey(selected.value);
    reviewed.value = ready;
  }

  function reset() {
    if (busy.value) return;
    const token = preview.value?.token;
    if (token)
      void api.discardMigrationPreview(token).catch(() => {
        /* Session cleanup is best effort; tokens also expire. */
      });
    bytes = undefined;
    preview.value = undefined;
    selected.value = [];
    reviewedSelection.value = "";
    result.value = undefined;
    filename.value = "";
    reviewed.value = false;
  }

  async function load(file: Pick<File, "name" | "size" | "arrayBuffer">) {
    if (busy.value) return;
    reset();
    if (file.size > 64 * 1024 * 1024) throw new Error("迁移包不能超过 64 MiB");
    busy.value = true;
    try {
      const data = Array.from(new Uint8Array(await file.arrayBuffer()));
      const value = await perform(() => api.previewMigration(data));
      bytes = data;
      filename.value = file.name;
      acceptPreview(value, false);
    } finally {
      busy.value = false;
    }
  }

  async function review() {
    if (busy.value || !bytes) return;
    const data = bytes;
    const names = [...selected.value];
    reviewed.value = false;
    busy.value = true;
    try {
      acceptPreview(await perform(() => api.previewMigration(data, names)), true);
    } finally {
      busy.value = false;
    }
  }

  async function importSelected() {
    if (!canImport.value || !preview.value) return;
    const token = preview.value.token;
    // A failed import may have consumed its plan too; require another preview.
    reviewed.value = false;
    busy.value = true;
    try {
      result.value = await perform(() => api.importMigration(token));
      preview.value = undefined;
      bytes = undefined;
      return result.value;
    } finally {
      busy.value = false;
    }
  }

  return {
    preview,
    selected,
    result,
    filename,
    busy,
    reviewed,
    canImport,
    reset,
    load,
    review,
    importSelected,
  };
}
