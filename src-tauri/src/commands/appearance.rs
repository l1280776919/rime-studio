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
