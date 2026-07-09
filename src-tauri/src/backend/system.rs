use crate::backend::*;
use crate::*;
use std::{fs, path::Path, process::Command};

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

pub(crate) fn deploy_rime_internal() -> Result<DeployResult, RimeError> {
    let deployer_path = locate_deployer()
        .ok_or_else(|| RimeError::DeployerNotFound("未找到 WeaselDeployer.exe".to_string()))?;

    let mut command = Command::new(&deployer_path);
    command.arg("/deploy").current_dir(
        deployer_path
            .parent()
            .ok_or_else(|| RimeError::DeployerNotFound("部署器路径异常".to_string()))?,
    );
    suppress_console_window(&mut command)
        .spawn()
        .map_err(|err| RimeError::CommandExecutionFailed(format!("运行部署器失败: {err}")))?;

    Ok(DeployResult {
        success: true,
        message: "已启动小狼毫重新部署，请稍候查看候选窗变化".to_string(),
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
        sogou_health: analyze_sogou(&user_dir.join("sogou_ext.dict.yaml")),
    })
}

pub(crate) fn deploy_rime_sync() -> Result<DeployResult, RimeError> {
    deploy_rime_internal()
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
    match deploy_rime_internal() {
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
