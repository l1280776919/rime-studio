use crate::backend::*;
use crate::*;
use std::{
    ffi::OsStr,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter};

const MAX_INSTALLER_BYTES: u64 = 512 * 1024 * 1024;
const DOWNLOAD_ATTEMPTS: u32 = 3;
static INSTALLER_DOWNLOAD: Mutex<()> = Mutex::new(());

fn emit_installer_progress(
    app: Option<&AppHandle>,
    stage: &str,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
) {
    if let Some(app) = app {
        let _ = app.emit(
            "rime-installer-progress",
            DownloadProgressPayload {
                kind: "weasel".to_string(),
                stage: stage.to_string(),
                downloaded_bytes,
                total_bytes,
                percent: total_bytes
                    .filter(|total| *total > 0)
                    .map(|total| (downloaded_bytes as f64 / total as f64 * 100.0).min(100.0)),
            },
        );
    }
}

fn is_installer_filename(name: &str) -> bool {
    !name.contains(['/', '\\', ':'])
        && Path::new(name).extension().and_then(OsStr::to_str) == Some("exe")
}

fn copy_installer<R: Read, W: Write>(
    mut reader: R,
    mut writer: W,
    total: Option<u64>,
    mut progress: impl FnMut(u64, Option<u64>),
) -> Result<(), RimeError> {
    if total.is_some_and(|total| !(2..=MAX_INSTALLER_BYTES).contains(&total)) {
        return Err(RimeError::DownloadError(
            "安装包大小无效或超过 512MB".to_string(),
        ));
    }
    progress(0, total);
    // Reject empty responses and HTML error pages before launching anything.
    let mut signature = [0u8; 2];
    reader
        .read_exact(&mut signature)
        .map_err(|err| RimeError::DownloadError(format!("安装包为空或下载中断: {err}")))?;
    if &signature != b"MZ" {
        return Err(RimeError::DownloadError(
            "下载内容不是 Windows 安装程序".to_string(),
        ));
    }
    writer
        .write_all(&signature)
        .map_err(|err| RimeError::FileOperationError(format!("保存安装包失败: {err}")))?;
    let mut downloaded = 2u64;
    let mut buffer = [0u8; 64 * 1024];
    let mut last_emit = Instant::now();
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|err| RimeError::DownloadError(format!("安装包下载中断: {err}")))?;
        if count == 0 {
            break;
        }
        downloaded += count as u64;
        if downloaded > MAX_INSTALLER_BYTES || total.is_some_and(|total| downloaded > total) {
            return Err(RimeError::DownloadError(
                "安装包大小与发布信息不符".to_string(),
            ));
        }
        writer
            .write_all(&buffer[..count])
            .map_err(|err| RimeError::FileOperationError(format!("保存安装包失败: {err}")))?;
        if last_emit.elapsed() >= Duration::from_millis(200) {
            progress(downloaded, total);
            last_emit = Instant::now();
        }
    }
    if total.is_some_and(|total| total != downloaded) {
        return Err(RimeError::DownloadError("安装包下载不完整".to_string()));
    }
    writer
        .flush()
        .map_err(|err| RimeError::FileOperationError(format!("保存安装包失败: {err}")))?;
    progress(downloaded, total);
    Ok(())
}

fn installer_http_error(err: ureq::Error) -> RimeError {
    match err {
        // Retrying cannot fix permissions or a nonexistent release asset.
        ureq::Error::Status(status, _) if (400..500).contains(&status) && status != 429 => {
            RimeError::CommandExecutionFailed(format!(
                "GitHub 返回 HTTP {status}，请使用官网手动下载"
            ))
        }
        _ => RimeError::NetworkError(format!("连接 GitHub 失败: {err}")),
    }
}

fn retry_installer_download<T>(
    mut attempt: impl FnMut(u32) -> Result<T, RimeError>,
    mut retry: impl FnMut(u32, &RimeError),
) -> Result<T, RimeError> {
    let mut number = 1;
    loop {
        match attempt(number) {
            Ok(value) => return Ok(value),
            Err(err)
                if number < DOWNLOAD_ATTEMPTS
                    && matches!(
                        err,
                        RimeError::NetworkError(_) | RimeError::DownloadError(_)
                    ) =>
            {
                retry(number, &err);
                number += 1;
            }
            Err(err) => return Err(err),
        }
    }
}

fn download_github_release_installer(
    api_url: &str,
    asset_filter: impl Fn(&str) -> bool,
    app: Option<&AppHandle>,
) -> Result<RimeDownloadResult, RimeError> {
    let _download_guard = INSTALLER_DOWNLOAD.try_lock().map_err(|_| {
        RimeError::CommandExecutionFailed("已有安装包正在下载，请等待完成后再试".to_string())
    })?;
    // Allow slower installer transfers without changing timeouts for other APIs.
    let agent = http_agent_with_read_timeout(Duration::from_secs(90));
    retry_installer_download(
        |number| {
            emit_installer_progress(app, &format!("正在获取安装包（第 {number} 次）"), 0, None);
            let response = agent
                .get(api_url)
                .set(
                    "User-Agent",
                    concat!("RimeStudio/", env!("CARGO_PKG_VERSION")),
                )
                .set("Accept", "application/vnd.github+json")
                .call()
                .map_err(installer_http_error)?;
            let json: serde_json::Value = response.into_json().map_err(|err| {
                RimeError::NetworkError(format!("读取 GitHub 发布信息失败: {err}"))
            })?;
            let asset = json["assets"]
                .as_array()
                .and_then(|assets| {
                    assets.iter().find(|asset| {
                        asset["name"]
                            .as_str()
                            .is_some_and(|name| is_installer_filename(name) && asset_filter(name))
                    })
                })
                .ok_or_else(|| {
                    RimeError::CommandExecutionFailed(
                        "未找到合适的安装包，请使用官网手动下载".to_string(),
                    )
                })?;
            let filename = asset["name"]
                .as_str()
                .ok_or_else(|| RimeError::DownloadError("发布资源缺少文件名".to_string()))?;
            let url = asset["browser_download_url"]
                .as_str()
                .ok_or_else(|| RimeError::DownloadError("发布资源缺少下载地址".to_string()))?;
            let expected_size = asset["size"].as_u64();
            let dest_dir = app_data_dir()?;
            fs::create_dir_all(&dest_dir)
                .map_err(|err| RimeError::FileOperationError(format!("创建下载目录失败: {err}")))?;
            let destination = dest_dir.join(filename);
            let partial = destination.with_extension("exe.part");
            let response = agent
                .get(url)
                .set(
                    "User-Agent",
                    concat!("RimeStudio/", env!("CARGO_PKG_VERSION")),
                )
                .set("Accept", "application/octet-stream")
                .call()
                .map_err(installer_http_error)?;
            let total = expected_size.or_else(|| {
                response
                    .header("Content-Length")
                    .and_then(|value| value.parse().ok())
            });
            let result = (|| {
                let file = fs::File::create(&partial).map_err(|err| {
                    RimeError::FileOperationError(format!("创建安装包失败: {err}"))
                })?;
                copy_installer(response.into_reader(), file, total, |downloaded, total| {
                    emit_installer_progress(app, "正在下载安装包", downloaded, total);
                })?;
                // Only publish a complete, validated download as an executable.
                if destination.exists() {
                    fs::remove_file(&destination).map_err(|err| {
                        RimeError::FileOperationError(format!("替换旧安装包失败: {err}"))
                    })?;
                }
                fs::rename(&partial, &destination).map_err(|err| {
                    RimeError::FileOperationError(format!("保存安装包失败: {err}"))
                })?;
                Ok(RimeDownloadResult {
                    success: true,
                    installer_path: Some(destination.display().to_string()),
                    message: format!("已下载 {filename}"),
                })
            })();
            if result.is_err() {
                let _ = fs::remove_file(&partial);
            }
            result
        },
        |number, err| {
            log::warn!("Installer download attempt {number} failed: {err}");
            emit_installer_progress(app, "下载中断，正在重试", 0, None);
            thread::sleep(Duration::from_secs(u64::from(number)));
        },
    )
    .map_err(|err| {
        if matches!(
            err,
            RimeError::NetworkError(_) | RimeError::DownloadError(_)
        ) {
            RimeError::DownloadError(format!(
                "{err}。已尝试 {DOWNLOAD_ATTEMPTS} 次，请检查网络或使用官网手动下载"
            ))
        } else {
            err
        }
    })
}

pub(crate) fn download_rime_installer_sync(
    app: &AppHandle,
) -> Result<RimeDownloadResult, RimeError> {
    download_github_release_installer(
        "https://api.github.com/repos/rime/weasel/releases/latest",
        |name| name.starts_with("weasel-") && name.ends_with("-installer.exe"),
        Some(app),
    )
}

pub(crate) fn download_git_installer_sync() -> Result<RimeDownloadResult, RimeError> {
    download_github_release_installer(
        "https://api.github.com/repos/git-for-windows/git/releases/latest",
        |name| name.starts_with("Git-") && name.contains("64-bit"),
        None,
    )
}

pub(crate) fn validate_downloaded_installer_path(path: String) -> Result<PathBuf, RimeError> {
    let installer = PathBuf::from(path);
    if !installer.is_file()
        || !is_installer_filename(installer.file_name().and_then(OsStr::to_str).unwrap_or(""))
    {
        return Err(RimeError::DownloadError(
            "安装包文件不存在或不是 .exe 文件".to_string(),
        ));
    }
    let app_dir = app_data_dir()?
        .canonicalize()
        .map_err(|err| RimeError::FileOperationError(format!("读取下载目录失败: {err}")))?;
    let installer = installer
        .canonicalize()
        .map_err(|err| RimeError::FileOperationError(format!("读取安装包路径失败: {err}")))?;
    if !installer.starts_with(app_dir) {
        return Err(RimeError::DownloadError(
            "只能启动 Rime Studio 下载目录内的安装包".to_string(),
        ));
    }
    Ok(installer)
}

pub(crate) fn launch_installer_sync(path: String) -> Result<(), RimeError> {
    let installer = validate_downloaded_installer_path(path)?;
    #[cfg(windows)]
    {
        // Shell execution requests UAC elevation. Pass the path as data, so
        // quotes or non-ASCII characters cannot change the PowerShell command.
        let mut command = Command::new("powershell");
        command.args(["-NoProfile", "-NonInteractive", "-Command",
            "$ErrorActionPreference='Stop'; [Console]::OutputEncoding=[System.Text.Encoding]::UTF8; Start-Process -FilePath $env:RIME_STUDIO_INSTALLER_PATH -Verb RunAs -PassThru | Out-Null"])
            .env("RIME_STUDIO_INSTALLER_PATH", &installer);
        let output = suppress_console_window(&mut command)
            .output()
            .map_err(|err| RimeError::CommandExecutionFailed(format!("启动安装程序失败: {err}")))?;
        if !output.status.success() {
            return Err(RimeError::CommandExecutionFailed(format!(
                "安装程序未启动，请确认管理员授权提示是否被取消: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
    }
    #[cfg(not(windows))]
    Command::new(&installer)
        .spawn()
        .map_err(|err| RimeError::CommandExecutionFailed(format!("启动安装程序失败: {err}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn rejects_truncated_and_html_downloads() {
        for (body, size) in [
            (b"MZpartial".as_slice(), Some(100)),
            (b"<html>error".as_slice(), None),
            (b"".as_slice(), None),
        ] {
            assert!(copy_installer(Cursor::new(body), Vec::new(), size, |_, _| {}).is_err());
        }
    }

    #[test]
    fn copies_complete_installer_and_emits_final_progress() {
        let body = b"MZinstaller";
        let mut output = Vec::new();
        let mut events = Vec::new();
        copy_installer(
            Cursor::new(body),
            &mut output,
            Some(body.len() as u64),
            |bytes, total| {
                events.push((bytes, total));
            },
        )
        .expect("complete download");
        assert_eq!(output, body);
        assert_eq!(
            events.last(),
            Some(&(body.len() as u64, Some(body.len() as u64)))
        );
    }

    #[test]
    fn retries_network_failures_but_not_disk_failures() {
        let mut calls = 0;
        let result = retry_installer_download(
            |_| {
                calls += 1;
                if calls < 3 {
                    Err(RimeError::NetworkError("timeout".to_string()))
                } else {
                    Ok(42)
                }
            },
            |_, _| {},
        );
        assert_eq!(result.expect("third attempt succeeds"), 42);
        assert_eq!(calls, 3);
        calls = 0;
        let result: Result<(), _> = retry_installer_download(
            |_| {
                calls += 1;
                Err(RimeError::FileOperationError("disk full".to_string()))
            },
            |_, _| {},
        );
        assert!(result.is_err());
        assert_eq!(calls, 1);
    }

    #[test]
    fn stops_after_three_failed_attempts() {
        let mut calls = 0;
        let result: Result<(), _> = retry_installer_download(
            |_| {
                calls += 1;
                Err(RimeError::DownloadError("incomplete".to_string()))
            },
            |_, _| {},
        );
        assert!(result.is_err());
        assert_eq!(calls, 3);
    }

    #[test]
    fn propagates_interrupted_reads_and_full_disk() {
        struct InterruptedReader(Cursor<Vec<u8>>);
        impl Read for InterruptedReader {
            fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
                if self.0.position() >= 2 {
                    Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout"))
                } else {
                    self.0.read(buffer)
                }
            }
        }
        let result = copy_installer(
            InterruptedReader(Cursor::new(b"MZ".to_vec())),
            Vec::new(),
            Some(100),
            |_, _| {},
        );
        assert!(matches!(result, Err(RimeError::DownloadError(_))));
        // A zero-sized output slice behaves like an exhausted destination.
        let result = copy_installer(Cursor::new(b"MZinstaller"), &mut [][..], None, |_, _| {});
        assert!(matches!(result, Err(RimeError::FileOperationError(_))));
    }

    #[test]
    fn rejects_oversized_or_mismatched_downloads() {
        for size in [0, 1, 3, MAX_INSTALLER_BYTES + 1] {
            assert!(copy_installer(
                Cursor::new(b"MZinstaller"),
                Vec::new(),
                Some(size),
                |_, _| {}
            )
            .is_err());
        }
    }

    #[test]
    fn rejects_path_components_in_release_filenames() {
        assert!(is_installer_filename("weasel-0.17.4-installer.exe"));
        for name in [
            "../installer.exe",
            r"..\installer.exe",
            "C:installer.exe",
            "installer.exe.part",
        ] {
            assert!(!is_installer_filename(name));
        }
    }
}
