use crate::backend::*;
use crate::commands::run_blocking;
use crate::types::RimeError;

#[tauri::command]
pub(crate) async fn list_lua_plugins() -> Result<Vec<LuaPluginInfo>, RimeError> {
    run_blocking(list_lua_plugins_sync).await
}

#[tauri::command]
pub(crate) async fn toggle_lua_plugin(
    plugin_id: String,
    enabled: bool,
) -> Result<Vec<LuaPluginInfo>, RimeError> {
    run_blocking(move || toggle_lua_plugin_sync(plugin_id, enabled)).await
}

#[tauri::command]
pub(crate) async fn get_lua_script_content(plugin_id: String) -> Result<String, RimeError> {
    run_blocking(move || get_lua_script_content_sync(plugin_id)).await
}

#[tauri::command]
pub(crate) async fn save_lua_script_content(
    plugin_id: String,
    content: String,
    expected: FileRevision,
) -> Result<(), RimeError> {
    run_blocking(move || save_lua_script_guarded_sync(plugin_id, content, expected)).await
}

#[tauri::command]
pub(crate) async fn read_lua_script_revision(
    plugin_id: String,
) -> Result<LuaScriptRevision, RimeError> {
    run_blocking(move || read_lua_script_revision_sync(plugin_id)).await
}
