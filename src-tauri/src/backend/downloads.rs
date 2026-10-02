use crate::backend::*;
use crate::*;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex,
};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process,
};
use tauri::Emitter;

static LMDG_OPERATION: Mutex<()> = Mutex::new(());

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
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    let root = app_data_dir()?;
    fs::create_dir_all(&root)?;
    loop {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let path = root.join(format!("{prefix}-{}-{id}", process::id()));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => {
                return Err(RimeError::FileOperationError(format!(
                    "创建临时目录失败: {err}"
                )))
            }
        }
    }
}

pub(crate) fn expand_zip_archive(zip_path: &Path, destination: &Path) -> Result<(), RimeError> {
    fs::create_dir_all(destination)
        .map_err(|err| RimeError::FileOperationError(format!("创建解压目录失败: {err}")))?;

    let file = fs::File::open(zip_path)
        .map_err(|err| RimeError::FileOperationError(format!("打开压缩包失败: {err}")))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|err| RimeError::FileOperationError(format!("解析压缩包失败: {err}")))?;

    for i in 0..archive.len() {
        let mut zip_file = archive
            .by_index(i)
            .map_err(|err| RimeError::FileOperationError(format!("读取压缩包文件失败: {err}")))?;

        let enclosed_name = match zip_file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue, // Skip insecure paths (Zip Slip protection)
        };

        if !valid_user_relative_path(&enclosed_name.to_string_lossy().replace('\\', "/")) {
            continue;
        }
        let out_path = destination.join(enclosed_name);

        if zip_file.is_dir() {
            fs::create_dir_all(&out_path)
                .map_err(|err| RimeError::FileOperationError(format!("创建解压目录失败: {err}")))?;
        } else {
            if let Some(parent) = out_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).map_err(|err| {
                        RimeError::FileOperationError(format!("创建解压目录失败: {err}"))
                    })?;
                }
            }
            let mut outfile = fs::File::create(&out_path)
                .map_err(|err| RimeError::FileOperationError(format!("创建解压文件失败: {err}")))?;
            std::io::copy(&mut zip_file, &mut outfile)
                .map_err(|err| RimeError::FileOperationError(format!("解压文件写入失败: {err}")))?;
        }
    }

    Ok(())
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
    let _config_guard = lock_config_write()?;
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
            let target = resolve_user_relative_path(
                target_dir,
                &relative.to_string_lossy().replace('\\', "/"),
                false,
            )?;
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
    let percent = total_bytes
        .filter(|total| *total > 0)
        .map(|total| ((downloaded_bytes as f64 / total as f64) * 100.0).min(100.0));
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
    let _guard = LMDG_OPERATION.try_lock().map_err(|_| {
        RimeError::CommandExecutionFailed("已有万象安装或卸载任务正在运行".to_string())
    })?;
    let (download_url, release_name) = github_release_asset_url(
        "https://api.github.com/repos/amzxyz/RIME-LMDG/releases",
        "dicts.zip",
    )?;
    let temporary = unique_temp_dir("lmdg-dicts")?;
    let result = (|| {
        let zip_path = temporary.join("dicts.zip");
        download_url_to_file_with_progress(
            &download_url,
            &zip_path,
            MAX_LMDG_DOWNLOAD_BYTES,
            "万象词库下载结果为空",
            "万象词库包超过 256MB，已取消安装",
            progress,
        )?;
        let extract_dir = temporary.join("contents");
        expand_zip_archive(&zip_path, &extract_dir)?;
        let user_dir = rime_user_dir()?;
        fs::create_dir_all(&user_dir)?;
        let target_dir = resolve_user_relative_path(&user_dir, "wanxiang", false)?;
        backup_user_config(&user_dir, BackupKind::BeforeInstall)?;
        let installed_count = copy_lmdg_dictionaries(&extract_dir, &target_dir)?;
        Ok(LmdgInstallResult {
            installed_count,
            target_dir: target_dir.display().to_string(),
            source_url: download_url,
            message: format!("已安装 {installed_count} 个万象词库文件（{release_name}）"),
        })
    })();
    let _ = fs::remove_dir_all(&temporary);
    result
}

/// Scan the actual model file, including models installed outside the workbench.
pub(crate) fn lmdg_grammar_installed_sync() -> Result<bool, RimeError> {
    let path = rime_user_dir()?.join("wanxiang-lts-zh-hans.gram");
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_file()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(RimeError::FileOperationError(format!(
            "扫描万象语言模型失败: {error}"
        ))),
    }
}

pub(crate) fn install_lmdg_grammar_sync_with_progress<F>(
    progress: F,
) -> Result<LmdgGrammarInstallResult, RimeError>
where
    F: FnMut(u64, Option<u64>),
{
    let _guard = LMDG_OPERATION.try_lock().map_err(|_| {
        RimeError::CommandExecutionFailed("已有万象安装或卸载任务正在运行".to_string())
    })?;
    if lmdg_grammar_installed_sync()? {
        return Err(RimeError::CommandExecutionFailed(
            "模型已安装，请先卸载再安装".into(),
        ));
    }
    let model_name = "wanxiang-lts-zh-hans";
    let asset_name = format!("{model_name}.gram");
    let (download_url, release_name) = github_release_asset_url(
        "https://api.github.com/repos/amzxyz/RIME-LMDG/releases",
        &asset_name,
    )?;

    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir)
        .map_err(|err| RimeError::FileOperationError(format!("创建 Rime 目录失败: {err}")))?;
    let model_path = resolve_user_relative_path(&user_dir, &asset_name, false)?;
    let patch_path = resolve_user_relative_path(&user_dir, "rime_ice.custom.yaml", false)?;

    let settings = get_rime_ice_settings_sync()?;
    let existing = read_optional_config(&patch_path)?;
    let rendered = merge_rime_ice_custom(&existing, &settings, LmdgPatchAction::Enable)?;
    backup_user_config(&user_dir, BackupKind::BeforeSave)?;
    download_url_to_file_with_progress(
        &download_url,
        &model_path,
        MAX_LMDG_GRAMMAR_BYTES,
        "万象语言模型下载结果为空",
        "万象语言模型超过 512MB，已取消安装",
        progress,
    )?;

    write_text_file(&patch_path, &rendered, "写入 rime_ice.custom.yaml 失败")?;

    Ok(LmdgGrammarInstallResult {
        model_name: model_name.to_string(),
        model_path: model_path.display().to_string(),
        patch_path: patch_path.display().to_string(),
        source_url: download_url,
        message: format!("已安装万象语言模型 {asset_name}（{release_name}），重新部署后生效"),
    })
}

pub(crate) fn uninstall_lmdg_grammar_sync() -> Result<LmdgGrammarUninstallResult, RimeError> {
    let _guard = LMDG_OPERATION.try_lock().map_err(|_| {
        RimeError::CommandExecutionFailed("已有万象安装或卸载任务正在运行".to_string())
    })?;
    if !lmdg_grammar_installed_sync()? {
        return Err(RimeError::CommandExecutionFailed(
            "未安装万象语言模型，无需卸载".into(),
        ));
    }
    let model_name = "wanxiang-lts-zh-hans";
    let asset_name = format!("{model_name}.gram");
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir)
        .map_err(|err| RimeError::FileOperationError(format!("创建 Rime 目录失败: {err}")))?;
    let model_path = resolve_user_relative_path(&user_dir, &asset_name, false)?;
    let patch_path = resolve_user_relative_path(&user_dir, "rime_ice.custom.yaml", false)?;

    let settings = get_rime_ice_settings_sync()?;
    let existing = read_optional_config(&patch_path)?;
    let rendered = merge_rime_ice_custom(&existing, &settings, LmdgPatchAction::Disable)?;
    backup_user_config(&user_dir, BackupKind::BeforeSave)?;
    write_text_file(&patch_path, &rendered, "写入 rime_ice.custom.yaml 失败")?;
    let removed_model = if model_path.exists() {
        fs::remove_file(&model_path)
            .map_err(|err| RimeError::FileOperationError(format!("删除万象语言模型失败: {err}")))?;
        true
    } else {
        false
    };

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
        parse_dictionary_import_payload(source_name.clone(), data)?;
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
    import_dictionary_with_source(source_name, data, None, None, None)
}

/// Keep provenance in a YAML comment so exports and backups retain it.
pub(crate) fn import_dictionary_with_source(
    source_name: String,
    data: Vec<u8>,
    source_url: Option<String>,
    display_name: Option<String>,
    source_label: Option<String>,
) -> Result<DictionaryImportResult, RimeError> {
    let _config_guard = lock_config_write()?;
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir)
        .map_err(|err| RimeError::FileOperationError(format!("创建 Rime 目录失败: {err}")))?;

    let (dict_name, reference, entries, skipped_entries, rendered_contents) =
        parse_dictionary_import_payload(source_name.clone(), data)?;
    let path = resolve_user_relative_path(&user_dir, &dict_name, false)?;
    backup_user_config(&user_dir, BackupKind::BeforeSave)?;
    let provenance = serde_json::json!({
        "name": display_name.filter(|s| !s.trim().is_empty()).unwrap_or(source_name),
        "source": source_label.filter(|s| !s.trim().is_empty()).unwrap_or_else(|| {
            if source_url.is_some() { "URL 导入".to_string() } else { "本地文件导入".to_string() }
        }),
        "url": source_url,
    });
    let rendered_contents = format!(
        "# rime-studio-source: {}\n{}",
        provenance, rendered_contents
    );
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

#[cfg(test)]
mod operation_tests {
    use super::*;

    #[test]
    fn concurrent_lmdg_operations_are_rejected_before_network_or_file_changes() {
        let _guard = LMDG_OPERATION.lock().expect("hold operation lock");
        assert!(install_lmdg_dicts_sync_with_progress(|_, _| {})
            .expect_err("reject concurrent install")
            .to_string()
            .contains("已有万象"));
        assert!(install_lmdg_grammar_sync_with_progress(|_, _| {})
            .expect_err("reject concurrent install")
            .to_string()
            .contains("已有万象"));
        assert!(uninstall_lmdg_grammar_sync()
            .expect_err("reject concurrent uninstall")
            .to_string()
            .contains("已有万象"));
    }
}
