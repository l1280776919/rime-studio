import { computed, ref } from "vue";
import type { FileRevision } from "../utils/fileConflict";
import type { FileStatus } from "../types";

export function useConfigDocument(
  read: (name: string) => Promise<string | FileRevision | undefined>,
  write: (
    name: string,
    content: string,
    expected: FileRevision,
  ) => Promise<boolean | { reload: FileRevision } | undefined>,
) {
  const selectedFile = ref<FileStatus | null>(null);
  const content = ref("");
  const originalContent = ref("");
  const loading = ref(false);
  const saving = ref(false);
  const dirty = computed(() => content.value !== originalContent.value);
  let revision: FileRevision = { content: null };
  let readVersion = 0;
  let disposed = false;

  async function load(file: FileStatus): Promise<boolean> {
    if (disposed || saving.value) return false;
    const version = ++readVersion;
    loading.value = true;
    try {
      const result = await read(file.name);
      if (version !== readVersion || result === undefined) return false;
      // Commit the filename and its contents together, only after a successful read.
      selectedFile.value = file;
      revision = typeof result === "string" ? { content: result } : result;
      originalContent.value = revision.initialContent ?? revision.content ?? "";
      content.value = originalContent.value;
      return true;
    } finally {
      if (version === readVersion) loading.value = false;
    }
  }

  async function save(): Promise<boolean> {
    if (loading.value || saving.value || !selectedFile.value) return false;
    const file = selectedFile.value;
    const snapshot = content.value;
    saving.value = true;
    try {
      const result = await write(file.name, snapshot, revision);
      if (!result) return false;
      if (typeof result === "object") {
        revision = result.reload;
        originalContent.value = revision.initialContent ?? revision.content ?? "";
        // Keep any new edits made while the conflict dialog was open.
        if (content.value === snapshot) content.value = originalContent.value;
        return false;
      }
      revision = { content: snapshot };
      // Edits made while saving still need to be saved.
      originalContent.value = snapshot;
      return true;
    } finally {
      saving.value = false;
    }
  }

  function cancelLoad() {
    disposed = true;
    ++readVersion;
    loading.value = false;
  }

  return { selectedFile, content, dirty, loading, saving, load, save, cancelLoad };
}
