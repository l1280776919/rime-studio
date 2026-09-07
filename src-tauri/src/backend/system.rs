use crate::backend::*;
use crate::*;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter};

static CANCEL_DEPLOY: AtomicBool = AtomicBool::new(false);

pub(crate) fn request_cancel_deploy() {
    CANCEL_DEPLOY.store(true, Ordering::SeqCst);
}

fn emit_deploy_progress(app: Option<&AppHandle>, stage: &str, log: &str, started: Instant) {
    let Some(app) = app else {
        return;
    };
    let _ = app.emit(
        "deploy-progress",
        DeployProgress {
            stage: stage.to_string(),
            log: log.to_string(),
            elapsed_ms: started.elapsed().as_millis() as u64,
        },
    );
}

fn directory_size(path: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| {
            let child = entry.path();
            if child.is_dir() {
                directory_size(&child)
            } else {
                entry.metadata().map(|meta| meta.len()).unwrap_or(0)
            }
        })
        .sum()
}

pub(crate) fn list_user_dicts(user_dir: &Path) -> Vec<UserDictInfo> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(user_dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if !name.ends_with(".userdb") {
            continue;
        }
        let modified = file_mtime(&path)
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs());
        out.push(UserDictInfo {
            name: name.to_string(),
            path: path.display().to_string(),
            size_bytes: directory_size(&path),
            modified,
        });
    }
    out.sort_by(|left, right| left.name.cmp(&right.name));
    out
}

pub(crate) fn open_in_explorer(path: &Path) -> Result<(), RimeError> {
    if !path.exists() {
        return Err(RimeError::FileOperationError(format!(
            "路径不存在: {}",
            path.display()
        )));
    }

    Command::new("explorer")
        .arg(path)
        .spawn()
        .map_err(|err| RimeError::CommandExecutionFailed(format!("打开资源管理器失败: {err}")))?;
    Ok(())
}

pub(crate) fn reveal_in_explorer(path: &Path) -> Result<(), RimeError> {
    if !path.exists() {
        return Err(RimeError::FileOperationError(format!(
            "路径不存在: {}",
            path.display()
        )));
    }

    Command::new("explorer")
        .arg("/select,")
        .arg(path)
        .spawn()
        .map_err(|err| RimeError::CommandExecutionFailed(format!("打开资源管理器失败: {err}")))?;
    Ok(())
}

pub(crate) fn run_command(mut command: Command) -> Result<(bool, String), RimeError> {
    let output = suppress_console_window(&mut command)
        .output()
        .map_err(|err| RimeError::CommandExecutionFailed(format!("运行命令失败: {err}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let log = format!("{stdout}{stderr}");

    Ok((output.status.success(), log))
}

pub(crate) fn ensure_plum(plum_dir: &Path) -> Result<String, RimeError> {
    let git = locate_git().ok_or_else(|| {
        RimeError::CommandExecutionFailed("安装 rime-ice 需要 Git，但未找到".to_string())
    })?;

    let proxy_envs = get_proxy_env_vars();
    let mut log = String::new();
    if plum_dir.join(".git").exists() {
        let mut command = Command::new(&git);
        command.arg("-C").arg(plum_dir).arg("pull").arg("--ff-only");
        for (key, value) in &proxy_envs {
            command.env(key, value);
        }
        let (success, command_log) = run_command(command)?;
        log.push_str(&command_log);
        if !success {
            return Err(RimeError::CommandExecutionFailed(format!(
                "更新 plum 失败:\n{log}"
            )));
        }
    } else {
        if let Some(parent) = plum_dir.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                RimeError::FileOperationError(format!("创建应用数据目录失败: {err}"))
            })?;
        }

        let mut command = Command::new(&git);
        command
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("https://github.com/rime/plum.git")
            .arg(plum_dir);
        for (key, value) in &proxy_envs {
            command.env(key, value);
        }
        let (success, command_log) = run_command(command)?;
        log.push_str(&command_log);
        if !success {
            return Err(RimeError::CommandExecutionFailed(format!(
                "克隆 plum 失败:\n{log}"
            )));
        }
    }

    Ok(log)
}

fn file_mtime(path: &Path) -> Option<std::time::SystemTime> {
    fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
}

fn collect_weasel_logs(user_dir: &Path) -> String {
    let mut candidates = vec![
        user_dir.join("rime.log"),
        user_dir.join("weasel.log"),
        user_dir.join("build").join("rime.log"),
    ];
    if let Ok(tmp) = env::var("TEMP").or_else(|_| env::var("TMP")) {
        if let Ok(entries) = fs::read_dir(tmp) {
            let mut temp_logs: Vec<(std::time::SystemTime, PathBuf)> = entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| {
                    let name = path
                        .file_name()
                        .and_then(|value| value.to_str())
                        .unwrap_or("")
                        .to_ascii_lowercase();
                    name.contains("rime")
                        && (name.ends_with(".log") || name.ends_with(".txt"))
                        && path.is_file()
                })
                .filter_map(|path| file_mtime(&path).map(|mtime| (mtime, path)))
                .collect();
            temp_logs.sort_by(|left, right| right.0.cmp(&left.0));
            candidates.extend(temp_logs.into_iter().take(3).map(|(_, path)| path));
        }
    }

    let mut chunks = Vec::new();
    for path in candidates {
        let Ok(contents) = fs::read_to_string(&path) else {
            continue;
        };
        if contents.trim().is_empty() {
            continue;
        }
        let tail: String = contents
            .lines()
            .rev()
            .take(80)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        chunks.push(format!("# {}\n{tail}", path.display()));
    }

    let mut log = chunks.join("\n\n");
    if log.len() > 8000 {
        log = log[log.len() - 8000..].to_string();
    }
    log
}

fn diagnose_yaml_files(user_dir: &Path, hints: &mut Vec<String>) {
    for name in [
        "default.custom.yaml",
        "weasel.custom.yaml",
        "rime_ice.custom.yaml",
    ] {
        let path = user_dir.join(name);
        if !path.exists() {
            continue;
        }
        let contents = read_to_string(&path);
        if let Err(err) = serde_yaml::from_str::<serde_yaml::Value>(&contents) {
            hints.push(format!("{name} YAML 语法错误: {err}"));
        }
    }
}

pub(crate) fn deploy_rime_internal(app: Option<&AppHandle>) -> Result<DeployResult, RimeError> {
    CANCEL_DEPLOY.store(false, Ordering::SeqCst);
    let started = Instant::now();
    let mut hints = Vec::new();
    let deployer_path = match locate_deployer() {
        Some(path) => path,
        None => {
            return Ok(DeployResult {
                success: false,
                message: "未找到 WeaselDeployer.exe".to_string(),
                log: String::new(),
                hints: vec![
                    "请先安装小狼毫输入法".to_string(),
                    "安装完成后回到概览页重新扫描环境".to_string(),
                ],
                duration_ms: started.elapsed().as_millis() as u64,
            });
        }
    };

    let user_dir = rime_user_dir()?;
    if !user_dir.exists() {
        hints.push("Rime 用户目录不存在，部署器可能没有可编译的配置".to_string());
    }
    diagnose_yaml_files(&user_dir, &mut hints);

    let build_weasel = user_dir.join("build").join("weasel.yaml");
    let build_default = user_dir.join("build").join("default.yaml");
    let before_weasel = file_mtime(&build_weasel);
    let before_default = file_mtime(&build_default);

    let mut command = Command::new(&deployer_path);
    command
        .arg("/deploy")
        .current_dir(
            deployer_path
                .parent()
                .ok_or_else(|| RimeError::DeployerNotFound("部署器路径异常".to_string()))?,
        )
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    suppress_console_window(&mut command);

    log::info!("Starting WeaselDeployer: {}", deployer_path.display());
    emit_deploy_progress(app, "正在启动 WeaselDeployer...", "", started);
    let mut child = command
        .spawn()
        .map_err(|err| RimeError::CommandExecutionFailed(format!("运行部署器失败: {err}")))?;

    let timeout = Duration::from_secs(90);
    let mut last_emit = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                let log = collect_weasel_logs(&user_dir);
                emit_deploy_progress(app, "部署超时", &log, started);
                return Ok(DeployResult {
                    success: false,
                    message: "部署超时（90 秒），已尝试结束部署器".to_string(),
                    log,
                    hints: vec![
                        "请手动运行「开始菜单 → 小狼毫输入法 → 重新部署」".to_string(),
                        "如果弹出 UAC 或部署窗口，请在窗口内确认".to_string(),
                    ],
                    duration_ms: started.elapsed().as_millis() as u64,
                });
            }
            Ok(None) => {
                if CANCEL_DEPLOY.swap(false, Ordering::SeqCst) {
                    let _ = child.kill();
                    let _ = child.wait();
                    let log = collect_weasel_logs(&user_dir);
                    emit_deploy_progress(app, "已取消部署", &log, started);
                    return Ok(DeployResult {
                        success: false,
                        message: "已取消部署".to_string(),
                        log,
                        hints: vec!["部署已被用户取消".to_string()],
                        duration_ms: started.elapsed().as_millis() as u64,
                    });
                }
                if last_emit.elapsed() >= Duration::from_secs(1) {
                    let log = collect_weasel_logs(&user_dir);
                    emit_deploy_progress(app, "正在部署小狼毫...", &log, started);
                    last_emit = Instant::now();
                }
                thread::sleep(Duration::from_millis(200));
            }
            Err(err) => {
                return Err(RimeError::CommandExecutionFailed(format!(
                    "等待部署器失败: {err}"
                )));
            }
        }
    };

    let log = collect_weasel_logs(&user_dir);
    let success = status.success();
    if success && before_weasel == file_mtime(&build_weasel) && build_weasel.exists() {
        hints.push(
            "build/weasel.yaml 修改时间未变化。若刚改过主题，请再部署一次或检查 YAML。".to_string(),
        );
    }
    if success && before_default == file_mtime(&build_default) && !build_default.exists() {
        hints.push("build/default.yaml 不存在，方案可能没有成功编译。".to_string());
    }
    if !success {
        hints.push(format!(
            "部署器退出码: {}",
            status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "未知".to_string())
        ));
        hints.push("常见原因：YAML 语法错误、方案文件缺失，或部署窗口被取消。".to_string());
    }

    let duration_ms = started.elapsed().as_millis() as u64;
    let message = if !success {
        "部署失败".to_string()
    } else if hints.iter().any(|hint| hint.contains("语法错误")) {
        "部署器已退出，但配置仍有问题".to_string()
    } else {
        format!("部署完成（{duration_ms} ms），候选窗应已更新")
    };

    log::info!("Weasel deploy finished: success={success}, {message}");

    Ok(DeployResult {
        success,
        message,
        log,
        hints,
        duration_ms,
    })
}

pub(crate) fn scan_rime_environment_sync() -> Result<RimeEnvironment, RimeError> {
    let user_dir = rime_user_dir()?;
    let build_dir = user_dir.join("build");
    let plum_dir = app_data_dir()?.join("plum");
    let default_custom = read_to_string(&user_dir.join("default.custom.yaml"));
    let appearance = read_appearance_config(&user_dir);
    let git_path = locate_git();
    let bash_path = locate_git_bash();

    Ok(RimeEnvironment {
        user_dir: user_dir.display().to_string(),
        build_dir: build_dir.display().to_string(),
        deployer_path: locate_deployer().map(|path| path.display().to_string()),
        plum_dir: plum_dir.display().to_string(),
        git_available: git_path.is_some(),
        bash_available: bash_path.is_some(),
        git_path: git_path.map(|path| path.display().to_string()),
        bash_path: bash_path.map(|path| path.display().to_string()),
        active_schema: parse_schema(&default_custom),
        page_size: parse_u32_after_key(&default_custom, "menu/page_size"),
        theme_name: Some(appearance.theme_name),
        font_point: Some(appearance.font_point),
        label_font_point: Some(appearance.label_font_point),
        custom_files: [
            "default.custom.yaml",
            "rime_ice.custom.yaml",
            "weasel.custom.yaml",
            "custom_phrase.txt",
            "rime_ice.schema.yaml",
            "rime_ice.dict.yaml",
            "rime_ice_ext.dict.yaml",
            "sogou_ext.dict.yaml",
        ]
        .into_iter()
        .map(|name| file_status(&user_dir, name))
        .collect(),
        sogou_health: None,
        user_dicts: list_user_dicts(&user_dir),
        sync_dir: file_status(&user_dir, "sync"),
    })
}

pub(crate) fn scan_dictionary_health_sync() -> Result<Option<DictHealth>, RimeError> {
    let path = rime_user_dir()?.join("sogou_ext.dict.yaml");
    Ok(analyze_dict_health(&path))
}

pub(crate) fn deploy_rime_sync(app: Option<AppHandle>) -> Result<DeployResult, RimeError> {
    deploy_rime_internal(app.as_ref())
}

pub(crate) fn install_rime_ice_sync(recipe: Option<String>) -> Result<InstallResult, RimeError> {
    let bash = locate_git_bash().ok_or_else(|| {
        RimeError::CommandExecutionFailed("运行 rime-install 需要 Git Bash，但未找到".to_string())
    })?;

    let recipe = recipe.unwrap_or_else(|| "iDvel/rime-ice:others/recipes/full".to_string());
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir)
        .map_err(|err| RimeError::FileOperationError(format!("创建 Rime 目录失败: {err}")))?;
    let backup_dir = backup_user_config(&user_dir, BackupKind::BeforeInstall)?;
    let backup_dir_display = backup_dir.display().to_string();
    let plum_dir = app_data_dir()?.join("plum");

    let mut log = String::new();
    log.push_str("已创建安装前备份: ");
    log.push_str(&backup_dir_display);
    log.push('\n');
    log.push_str("正在准备 plum...\n");
    log.push_str(&ensure_plum(&plum_dir)?);
    log.push_str("\n正在安装方案: ");
    log.push_str(&recipe);
    log.push('\n');

    let proxy_envs = get_proxy_env_vars();
    let mut command = Command::new(&bash);
    command
        .arg("rime-install")
        .arg(&recipe)
        .current_dir(&plum_dir)
        .env("rime_dir", &user_dir);
    for (key, value) in &proxy_envs {
        command.env(key, value);
    }
    let (install_success, install_log) = run_command(command)?;
    log.push_str(&install_log);

    if !install_success {
        return Ok(InstallResult {
            success: false,
            recipe,
            backup_dir: Some(backup_dir_display.clone()),
            log,
        });
    }

    log.push_str("\n正在部署小狼毫...\n");
    match deploy_rime_internal(None) {
        Ok(result) => {
            log.push_str(&result.message);
            Ok(InstallResult {
                success: result.success,
                recipe,
                backup_dir: Some(backup_dir_display.clone()),
                log,
            })
        }
        Err(err) => {
            log.push_str(&err.to_string());
            Ok(InstallResult {
                success: false,
                recipe,
                backup_dir: Some(backup_dir_display.clone()),
                log,
            })
        }
    }
}
