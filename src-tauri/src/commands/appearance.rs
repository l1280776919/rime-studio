use crate::backend::*;
use crate::types::*;

use super::run_blocking;

#[tauri::command]
pub(crate) async fn get_appearance_config() -> Result<AppearanceConfig, RimeError> {
    run_blocking(get_appearance_config_sync).await
}

#[tauri::command]
pub(crate) async fn save_appearance_config(
    config: AppearanceConfig,
) -> Result<AppearanceConfig, RimeError> {
    run_blocking(move || save_appearance_config_sync(config)).await
}

#[tauri::command]
pub(crate) async fn preview_appearance_config(
    config: AppearanceConfig,
) -> Result<ConfigPreview, RimeError> {
    run_blocking(move || preview_appearance_config_sync(config)).await
}

#[tauri::command]
pub(crate) async fn list_system_fonts() -> Result<Vec<String>, RimeError> {
    run_blocking(list_system_fonts_sync).await
}
