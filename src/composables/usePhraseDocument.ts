import { computed, ref } from "vue";
import type { FileRevision } from "../utils/fileConflict";
import type { PhraseEntry } from "../types";

export function usePhraseDocument(
  read: () => Promise<
    PhraseEntry[] | { entries: PhraseEntry[]; revision: FileRevision } | undefined
  >,
  write: (
    entries: PhraseEntry[],
    expected: FileRevision,
  ) => Promise<
    | boolean
    | { saved: FileRevision }
    | { reload: PhraseEntry[]; revision: FileRevision }
    | undefined
  >,
) {
  const entries = ref<PhraseEntry[]>([]);
  const loading = ref(false);
  const saving = ref(false);
  const ready = ref(false);
  const original = ref("[]");
  const dirty = computed(() => ready.value && JSON.stringify(entries.value) !== original.value);
  let version = 0;
  let revision: FileRevision = { content: null };

  async function load() {
    if (saving.value) return false;
    const request = ++version;
    loading.value = true;
    try {
      const result = await read();
      if (request !== version || result === undefined) return false;
      // Commit rows and revision together only for the accepted read request.
      entries.value = Array.isArray(result) ? result : result.entries;
      revision = Array.isArray(result) ? { content: null } : result.revision;
      original.value = JSON.stringify(entries.value);
      ready.value = true;
      return true;
    } finally {
      if (request === version) loading.value = false;
    }
  }

  async function save() {
    if (!ready.value || loading.value || saving.value) return false;
    const snapshot = entries.value.map((entry) => ({ ...entry }));
    saving.value = true;
    try {
      const result = await write(snapshot, revision);
      if (!result) return false;
      if (typeof result === "object" && "reload" in result) {
        revision = result.revision;
        original.value = JSON.stringify(result.reload);
        if (JSON.stringify(entries.value) === JSON.stringify(snapshot))
          entries.value = result.reload;
        return false;
      }
      if (typeof result === "object" && "saved" in result) revision = result.saved;
      original.value = JSON.stringify(snapshot);
      return true;
    } finally {
      saving.value = false;
    }
  }

  function cancelLoad() {
    ++version;
    loading.value = false;
  }

  function reset() {
    if (!ready.value || loading.value || saving.value) return false;
    entries.value = JSON.parse(original.value);
    return true;
  }

  return { entries, loading, saving, ready, dirty, load, save, reset, cancelLoad };
}
