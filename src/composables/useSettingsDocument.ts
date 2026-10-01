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
  const savedValue = ref<string>();
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
      savedValue.value = JSON.stringify(snapshot());
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
      savedValue.value = JSON.stringify(result);
      return true;
    } finally {
      saving.value = false;
    }
  }

  function dispose() {
    disposed = true;
  }
  function reset() {
    if (disposed || loading.value || saving.value || savedValue.value === undefined) return false;
    apply(JSON.parse(savedValue.value));
    return true;
  }
  function field(value: unknown, path: string) {
    const keys = path.split(".");
    let parent = value;
    for (const key of keys.slice(0, -1)) {
      if (
        !parent ||
        typeof parent !== "object" ||
        !Object.hasOwn(parent, key) ||
        ["__proto__", "constructor", "prototype"].includes(key)
      )
        return;
      parent = (parent as Record<string, unknown>)[key];
    }
    const key = keys[keys.length - 1];
    if (
      !parent ||
      typeof parent !== "object" ||
      !Object.hasOwn(parent, key) ||
      ["__proto__", "constructor", "prototype"].includes(key)
    )
      return;
    return { parent: parent as Record<string, unknown>, key };
  }
  function isChanged(path: string) {
    if (!ready.value || !savedValue.value) return false;
    const current = field(snapshot(), path);
    const baseline = field(JSON.parse(savedValue.value), path);
    return Boolean(
      current &&
      baseline &&
      fingerprint(current.parent[current.key]) !== fingerprint(baseline.parent[baseline.key]),
    );
  }
  function resetFields(paths: string[]) {
    if (disposed || loading.value || saving.value || !savedValue.value) return false;
    const current = JSON.parse(JSON.stringify(snapshot()));
    const baseline = JSON.parse(savedValue.value);
    const fields = paths.map((path) => ({
      current: field(current, path),
      baseline: field(baseline, path),
    }));
    if (!fields.length || fields.some((item) => !item.current || !item.baseline)) return false;
    for (const item of fields)
      item.current!.parent[item.current!.key] = item.baseline!.parent[item.baseline!.key];
    apply(current);
    return true;
  }
  return { ready, loading, saving, dirty, load, save, reset, isChanged, resetFields, dispose };
}
