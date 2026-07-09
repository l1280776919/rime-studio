use crate::backend::*;
use crate::types::*;

use super::run_blocking;

#[tauri::command]
pub(crate) async fn get_custom_phrases() -> Result<Vec<PhraseEntry>, RimeError> {
    run_blocking(get_custom_phrases_sync).await
}

#[tauri::command]
pub(crate) async fn save_custom_phrases(phrases: Vec<PhraseEntry>) -> Result<(), RimeError> {
    run_blocking(move || save_custom_phrases_sync(phrases)).await
}
