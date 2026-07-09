use crate::backend::*;
use crate::*;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{self, Command},
};
use tauri::Emitter;

pub(crate) fn github_release_asset_url(
    api_url: &str,
    asset_name: &str,
) -> Result<(String, String), RimeError> {
    let response = http_get(api_url)
        .call()
        .map_err(|err| RimeError::NetworkError(format!("获取 GitHub 发布信息失败: {err}")))?;
    let releases: serde_json::Value = response
        .into_json()
        .map_err(|err| RimeError::NetworkError(format!("解析 GitHub 发布信息失败: {err}")))?;
    let releases = releases
        .as_array()
        .ok_or_else(|| RimeError::NetworkError("GitHub 发布信息格式无效".to_string()))?;

    for release in releases {
        let release_name = release["name"]
            .as_str()
            .or_else(|| release["tag_name"].as_str())
            .unwrap_or("RIME-LMDG");
        let Some(assets) = release["assets"].as_array() else {
            continue;
        };
        for asset in assets {
            if asset["name"].as_str() == Some(asset_name) {
                let url = asset["browser_download_url"].as_str().ok_or_else(|| {
                    RimeError::NetworkError("GitHub 发布资源缺少下载地址".to_string())
                })?;
                return Ok((url.to_string(), release_name.to_string()));
            }
        }
    }

    Err(RimeError::NetworkError(format!(
        "未在 RIME-LMDG 发布资源中找到 {asset_name}"
    )))
}

pub(crate) fn unique_temp_dir(prefix: &str) -> Result<PathBuf, RimeError> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| RimeError::CommandExecutionFailed(format!("读取系统时间失败: {err}")))?
        .as_millis();
    Ok(app_data_dir()?.join(format!("{prefix}-{}-{millis}", process::id())))
}

pub(crate) fn expand_zip_archive(zip_path: &Path, destination: &Path) -> Result<(), RimeError> {
    fs::create_dir_all(destination)
        .map_err(|err| RimeError::FileOperationError(format!("创建解压目录失败: {err}")))?;
    let mut command = Command::new("powershell");
    command
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg("& { param($zip, $dest) Expand-Archive -LiteralPath $zip -DestinationPath $dest -Force }")
        .arg(zip_path)
        .arg(destination);
    let (success, log) = run_command(command)?;
    if success {
        Ok(())
    } else {
        Err(RimeError::CommandExecutionFailed(format!(
            "解压万象词库失败:\n{log}"
        )))
    }
}

pub(crate) fn safe_relative_path(path: &Path) -> bool {
    path.components().all(|component| {
        matches!(
            component,
            std::path::Component::Normal(_) | std::path::Component::CurDir
        )
    })
}

pub(crate) fn copy_lmdg_dictionaries(
    source_dir: &Path,
    target_dir: &Path,
) -> Result<usize, RimeError> {
    fs::create_dir_all(target_dir)
        .map_err(|err| RimeError::FileOperationError(format!("创建万象词库目录失败: {err}")))?;
    let mut installed = 0usize;
    let mut pending = vec![source_dir.to_path_buf()];

    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).map_err(|err| {
            RimeError::FileOperationError(format!("读取万象词库解压目录失败: {err}"))
        })? {
            let entry = entry.map_err(|err| {
                RimeError::FileOperationError(format!("读取万象词库文件失败: {err}"))
            })?;
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }

            let Some(file_name) = path.file_name().and_then(OsStr::to_str) else {
                continue;
            };
            if !file_name.ends_with(".dict.yaml") {
                continue;
            }

            let relative = path.strip_prefix(source_dir).map_err(|err| {
                RimeError::FileOperationError(format!("计算万象词库路径失败: {err}"))
            })?;
            if !safe_relative_path(relative) {
                continue;
            }
            let target = target_dir.join(relative);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|err| {
                    RimeError::FileOperationError(format!("创建万象词库子目录失败: {err}"))
                })?;
            }
            fs::copy(&path, &target)
                .map_err(|err| RimeError::FileOperationError(format!("复制万象词库失败: {err}")))?;
            installed += 1;
        }
    }

    if installed == 0 {
        Err(RimeError::DownloadError(
            "万象词库包里没有找到 .dict.yaml 文件".to_string(),
        ))
    } else {
        Ok(installed)
    }
}

pub(crate) fn emit_download_progress(
    window: &tauri::Window,
    kind: &str,
    stage: &str,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
) {
    let percent =
        total_bytes.map(|total| ((downloaded_bytes as f64 / total as f64) * 100.0).min(100.0));
    let _ = window.emit(
        "lmdg-download-progress",
        DownloadProgressPayload {
            kind: kind.to_string(),
            stage: stage.to_string(),
            downloaded_bytes,
            total_bytes,
            percent,
        },
    );
}

pub(crate) fn install_lmdg_dicts_sync_with_progress<F>(
    progress: F,
) -> Result<LmdgInstallResult, RimeError>
where
    F: FnMut(u64, Option<u64>),
{
    let (download_url, release_name) = github_release_asset_url(
        "https://api.github.com/repos/amzxyz/RIME-LMDG/releases",
        "dicts.zip",
    )?;
    let app_dir = app_data_dir()?;
    fs::create_dir_all(&app_dir)
        .map_err(|err| RimeError::FileOperationError(format!("创建下载目录失败: {err}")))?;
    let zip_path = app_dir.join("RIME-LMDG-dicts.zip");
    download_url_to_file_with_progress(
        &download_url,
        &zip_path,
        MAX_LMDG_DOWNLOAD_BYTES,
        "万象词库下载结果为空",
        "万象词库包超过 256MB，已取消安装",
        progress,
    )?;

    let extract_dir = unique_temp_dir("lmdg-dicts")?;
    expand_zip_archive(&zip_path, &extract_dir)?;

    let target_dir = rime_user_dir()?.join("wanxiang");
    let installed_count = copy_lmdg_dictionaries(&extract_dir, &target_dir)?;
    let _ = fs::remove_dir_all(&extract_dir);

    Ok(LmdgInstallResult {
        installed_count,
        target_dir: target_dir.display().to_string(),
        source_url: download_url,
        message: format!("已安装 {installed_count} 个万象词库文件（{release_name}）"),
    })
}

pub(crate) fn install_lmdg_grammar_sync_with_progress<F>(
    progress: F,
) -> Result<LmdgGrammarInstallResult, RimeError>
where
    F: FnMut(u64, Option<u64>),
{
    let model_name = "wanxiang-lts-zh-hans";
    let asset_name = format!("{model_name}.gram");
    let (download_url, release_name) = github_release_asset_url(
        "https://api.github.com/repos/amzxyz/RIME-LMDG/releases",
        &asset_name,
    )?;

    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir)
        .map_err(|err| RimeError::FileOperationError(format!("创建 Rime 目录失败: {err}")))?;
    let model_path = user_dir.join(&asset_name);
    let patch_path = user_dir.join("rime_ice.custom.yaml");

    backup_user_config(&user_dir, BackupKind::BeforeSave)?;
    download_url_to_file_with_progress(
        &download_url,
        &model_path,
        MAX_LMDG_GRAMMAR_BYTES,
        "万象语言模型下载结果为空",
        "万象语言模型超过 512MB，已取消安装",
        progress,
    )?;

    let settings = get_rime_ice_settings_sync()?;
    write_text_file(
        &patch_path,
        &render_rime_ice_custom(&settings, true, settings.fuzzy_pinyin),
        "写入 rime_ice.custom.yaml 失败",
    )?;

    Ok(LmdgGrammarInstallResult {
        model_name: model_name.to_string(),
        model_path: model_path.display().to_string(),
        patch_path: patch_path.display().to_string(),
        source_url: download_url,
        message: format!("已安装万象语言模型 {asset_name}（{release_name}），重新部署后生效"),
    })
}

pub(crate) fn uninstall_lmdg_grammar_sync() -> Result<LmdgGrammarUninstallResult, RimeError> {
    let model_name = "wanxiang-lts-zh-hans";
    let asset_name = format!("{model_name}.gram");
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir)
        .map_err(|err| RimeError::FileOperationError(format!("创建 Rime 目录失败: {err}")))?;
    let model_path = user_dir.join(&asset_name);
    let patch_path = user_dir.join("rime_ice.custom.yaml");

    backup_user_config(&user_dir, BackupKind::BeforeSave)?;
    let removed_model = if model_path.exists() {
        fs::remove_file(&model_path)
            .map_err(|err| RimeError::FileOperationError(format!("删除万象语言模型失败: {err}")))?;
        true
    } else {
        false
    };

    let settings = get_rime_ice_settings_sync()?;
    write_text_file(
        &patch_path,
        &render_rime_ice_custom(&settings, false, settings.fuzzy_pinyin),
        "写入 rime_ice.custom.yaml 失败",
    )?;

    Ok(LmdgGrammarUninstallResult {
        model_name: model_name.to_string(),
        model_path: model_path.display().to_string(),
        patch_path: patch_path.display().to_string(),
        removed_model,
        message: if removed_model {
            "已卸载万象语言模型，重新部署后生效".to_string()
        } else {
            "已移除万象语言模型配置，未发现模型文件，重新部署后生效".to_string()
        },
    })
}

pub(crate) fn preview_dictionary_import_sync(
    source_name: String,
    data: Vec<u8>,
) -> Result<DictionaryImportPreview, RimeError> {
    let user_dir = rime_user_dir()?;
    let (dict_name, reference, entries, skipped_entries, _) =
        parse_dictionary_import_payload(source_name, data)?;
    let path = user_dir.join(&dict_name);
    let sample_entries = entries
        .iter()
        .take(20)
        .map(|(text, code, weight)| DictionaryPreviewEntry {
            text: text.clone(),
            code: code.clone(),
            weight: *weight,
        })
        .collect();

    Ok(DictionaryImportPreview {
        reference,
        name: dict_name,
        path: path.display().to_string(),
        imported_entries: entries.len(),
        skipped_entries,
        sample_entries,
        will_overwrite: path.exists(),
    })
}

pub(crate) fn import_dictionary_sync(
    source_name: String,
    data: Vec<u8>,
) -> Result<DictionaryImportResult, RimeError> {
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir)
        .map_err(|err| RimeError::FileOperationError(format!("创建 Rime 目录失败: {err}")))?;

    let (dict_name, reference, entries, skipped_entries, rendered_contents) =
        parse_dictionary_import_payload(source_name, data)?;
    let path = user_dir.join(&dict_name);
    write_text_file(&path, &rendered_contents, "写入导入词库失败")?;

    Ok(DictionaryImportResult {
        reference,
        name: dict_name,
        path: path.display().to_string(),
        imported_entries: entries.len(),
        skipped_entries,
    })
}

pub(crate) fn export_dictionary_sync(
    dict_name: String,
) -> Result<DictionaryExportResult, RimeError> {
    let user_dir = rime_user_dir()?;
    let path = validate_dictionary_path(&user_dir, &dict_name)?;

    let contents = fs::read_to_string(&path)
        .map_err(|err| RimeError::FileOperationError(format!("读取词库失败: {err}")))?;
    Ok(DictionaryExportResult {
        name: path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("dictionary.dict.yaml")
            .to_string(),
        contents,
    })
}
