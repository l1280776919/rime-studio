import { computed, ref } from "vue";

function fingerprint(value: unknown) {
  return JSON.stringify(value, (_key, item) =>
    item && typeof item === "object" && !Array.isArray(item)
      ? Object.fromEntries(
          Object.keys(item)
            .sort()
            .map((key) => [key, item[key]]),
        )
      : item,
  );
}

// The form remains editable while saving. Only the exact submitted snapshot is marked saved.
export function useSettingsDocument<T>(
  snapshot: () => T,
  apply: (value: T) => void,
  read: () => Promise<T | undefined>,
  write: (value: T) => Promise<T | undefined>,
) {
  const ready = ref(false);
  const loading = ref(false);
  const saving = ref(false);
  const original = ref(fingerprint(snapshot()));
  let savedValue: string | undefined;
  const dirty = computed(() => ready.value && fingerprint(snapshot()) !== original.value);
  let disposed = false;

  async function load() {
    if (disposed || loading.value || saving.value || dirty.value) return false;
    const before = fingerprint(snapshot());
    loading.value = true;
    try {
      const value = await read();
      if (disposed || value === undefined || fingerprint(snapshot()) !== before) return false;
      apply(value);
      original.value = fingerprint(snapshot());
      savedValue = JSON.stringify(snapshot());
      ready.value = true;
      return true;
    } finally {
      loading.value = false;
    }
  }

  async function save() {
    if (disposed || !ready.value || loading.value || saving.value) return false;
    const submitted = JSON.stringify(snapshot());
    const value: T = JSON.parse(submitted);
    saving.value = true;
    try {
      const result = await write(value);
      if (disposed || result === undefined) return false;
      if (fingerprint(snapshot()) === fingerprint(value)) apply(result);
      original.value = fingerprint(result);
      savedValue = JSON.stringify(result);
      return true;
    } finally {
      saving.value = false;
    }
  }

  function dispose() {
    disposed = true;
  }
  function reset() {
    if (disposed || loading.value || saving.value || savedValue === undefined) return false;
    apply(JSON.parse(savedValue));
    return true;
  }
  return { ready, loading, saving, dirty, load, save, reset, dispose };
}
