import { onActivated, onDeactivated, watch } from "vue";

// KeepAlive pages must re-read shared configuration when revisited; page loaders preserve dirty drafts.
export function useConfigReload(source: () => unknown, reload: () => unknown) {
  let active = true;
  let firstActivation = true;
  onActivated(() => {
    active = true;
    if (firstActivation) {
      firstActivation = false;
      return;
    }
    void reload();
  });
  onDeactivated(() => {
    active = false;
  });
  watch(source, () => {
    if (active) void reload();
  });
}
