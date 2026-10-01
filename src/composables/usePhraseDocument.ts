import { computed, ref } from "vue";
import type { PhraseEntry } from "../types";

export function usePhraseDocument(
  read: () => Promise<PhraseEntry[] | undefined>,
  write: (entries: PhraseEntry[]) => Promise<boolean | undefined>,
) {
  const entries = ref<PhraseEntry[]>([]);
  const loading = ref(false);
  const saving = ref(false);
  const ready = ref(false);
  const original = ref("[]");
  const dirty = computed(() => ready.value && JSON.stringify(entries.value) !== original.value);
  let version = 0;

  async function load() {
    if (saving.value) return false;
    const request = ++version;
    loading.value = true;
    try {
      const result = await read();
      if (request !== version || result === undefined) return false;
      entries.value = result;
      original.value = JSON.stringify(result);
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
      if (!(await write(snapshot))) return false;
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
