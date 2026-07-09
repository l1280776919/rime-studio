use crate::types::RimeError;

mod app_update;
mod appearance;
mod backup;
mod dictionaries;
mod phrases;
mod schemas;
mod settings;
mod system;

pub(crate) use app_update::*;
pub(crate) use appearance::*;
pub(crate) use backup::*;
pub(crate) use dictionaries::*;
pub(crate) use phrases::*;
pub(crate) use schemas::*;
pub(crate) use settings::*;
pub(crate) use system::*;

pub(crate) async fn run_blocking<T, F>(task: F) -> Result<T, RimeError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, RimeError> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|err| RimeError::CommandExecutionFailed(format!("后台任务失败: {err}")))?
}
