use crate::backend::*;
use crate::types::*;

use super::run_blocking;

#[tauri::command]
pub(crate) async fn get_quick_settings() -> Result<QuickSettingsConfig, RimeError> {
    run_blocking(get_quick_settings_sync).await
}

#[tauri::command]
pub(crate) async fn save_quick_settings(
    config: QuickSettingsConfig,
) -> Result<QuickSettingsConfig, RimeError> {
    run_blocking(move || save_quick_settings_sync(config)).await
}

#[tauri::command]
pub(crate) async fn preview_quick_settings(
    config: QuickSettingsConfig,
) -> Result<ConfigPreview, RimeError> {
    run_blocking(move || preview_quick_settings_sync(config)).await
}

#[tauri::command]
pub(crate) async fn inspect_config_health() -> Result<ConfigHealthReport, RimeError> {
    run_blocking(inspect_config_health_sync).await
}

#[tauri::command]
pub(crate) async fn repair_config_health() -> Result<ConfigHealthReport, RimeError> {
    run_blocking(repair_config_health_sync).await
}

#[tauri::command]
pub(crate) async fn repair_config_health_item(
    name: String,
) -> Result<ConfigHealthReport, RimeError> {
    run_blocking(move || repair_config_health_item_sync(name)).await
}

#[tauri::command]
pub(crate) async fn get_rime_ice_settings() -> Result<RimeIceSettings, RimeError> {
    run_blocking(get_rime_ice_settings_sync).await
}

#[tauri::command]
pub(crate) async fn save_rime_ice_settings(
    settings: RimeIceSettings,
) -> Result<RimeIceSettings, RimeError> {
    run_blocking(move || save_rime_ice_settings_sync(settings)).await
}
