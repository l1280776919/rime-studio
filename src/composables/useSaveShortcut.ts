import { onActivated, onBeforeUnmount, onDeactivated, onMounted } from "vue";

export function useSaveShortcut(save: () => unknown) {
  let active = true;
  function onKey(event: KeyboardEvent) {
    if (!active || event.defaultPrevented || event.isComposing || event.repeat || event.altKey)
      return;
    if (!(event.ctrlKey || event.metaKey) || event.key.toLowerCase() !== "s") return;
    if (
      event.target instanceof Element &&
      event.target.closest(".el-dialog, .el-message-box, .palette-overlay")
    )
      return;
    event.preventDefault();
    void save();
  }
  onMounted(() => window.addEventListener("keydown", onKey));
  onActivated(() => {
    active = true;
  });
  onDeactivated(() => {
    active = false;
  });
  onBeforeUnmount(() => window.removeEventListener("keydown", onKey));
}
