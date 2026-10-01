import { onScopeDispose, watch } from "vue";

// A command requests a preview; configuration writes remain explicit page actions.
export function useThemePreview(
  request: () => string | undefined,
  ready: () => boolean,
  dirty: () => boolean,
  apply: (name: string) => void,
  confirm: () => Promise<boolean>,
  clear: (name: string) => Promise<void>,
) {
  let busy = false;
  let disposed = false;
  onScopeDispose(() => {
    disposed = true;
  });
  async function preview() {
    const name = request();
    if (!name || !ready() || busy || disposed) return;
    busy = true;
    try {
      const accepted = !dirty() || (await confirm());
      if (accepted && !disposed && ready() && request() === name) apply(name);
    } finally {
      try {
        if (!disposed && request() === name) await clear(name);
      } finally {
        busy = false;
        if (!disposed && request() !== name) void preview();
      }
    }
  }
  watch(
    () => [request(), ready()],
    () => {
      void preview();
    },
    { immediate: true },
  );
}
