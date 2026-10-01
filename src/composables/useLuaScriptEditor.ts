import { ref } from "vue";
import type { LuaPluginInfo } from "../types";
import { useConfigDocument } from "./useConfigDocument";

export function useLuaScriptEditor(
  read: (id: string) => Promise<string | undefined>,
  write: (id: string, content: string) => Promise<boolean | undefined>,
  saved: () => void,
) {
  const document = useConfigDocument(read, write);
  const editingPlugin = ref<LuaPluginInfo | null>(null);
  const visible = ref(false);

  async function open(plugin: LuaPluginInfo) {
    if (
      await document.load({ name: plugin.id, path: plugin.file_name, exists: plugin.installed })
    ) {
      editingPlugin.value = plugin;
      visible.value = true;
    }
  }

  async function save() {
    if (await document.save()) {
      saved();
      if (!document.dirty.value) visible.value = false;
    }
  }

  return { ...document, editingPlugin, visible, open, save };
}
