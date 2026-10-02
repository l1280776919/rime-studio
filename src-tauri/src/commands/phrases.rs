use crate::backend::*;
use crate::types::*;

use super::run_blocking;

#[tauri::command]
pub(crate) async fn get_custom_phrases() -> Result<Vec<PhraseEntry>, RimeError> {
    run_blocking(get_custom_phrases_sync).await
}

#[tauri::command]
pub(crate) async fn save_custom_phrases(
    phrases: Vec<PhraseEntry>,
    expected: FileRevision,
) -> Result<FileRevision, RimeError> {
    run_blocking(move || save_custom_phrases_guarded_sync(phrases, expected)).await
}

#[tauri::command]
pub(crate) async fn read_phrase_document() -> Result<PhraseDocument, RimeError> {
    run_blocking(read_phrase_document_sync).await
}
