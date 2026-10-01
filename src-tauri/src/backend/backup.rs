use crate::backend::*;
use crate::*;
use std::{
    ffi::OsStr,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{self},
    sync::atomic::{AtomicU64, Ordering},
};

static BACKUP_OPERATIONS: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub(crate) fn lock_backup_operation() -> Result<std::sync::MutexGuard<'static, ()>, RimeError> {
    BACKUP_OPERATIONS
        .lock()
        .map_err(|_| RimeError::BackupError("备份操作锁不可用".into()))
}

pub(crate) fn is_dictionary_entry_line(trimmed: &str) -> bool {
    !trimmed.is_empty()
        && !trimmed.starts_with('#')
        && trimmed != "---"
        && trimmed != "..."
        && !trimmed.starts_with("name:")
        && !trimmed.starts_with("version:")
        && !trimmed.starts_with("sort:")
        && trimmed.contains('\t')
}

pub(crate) fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "unknown-time".to_string())
}

#[derive(Clone, Copy)]
pub(crate) enum BackupKind {
    Manual,
    BeforeSave,
    BeforeRestore,
    BeforeInstall,
}

impl BackupKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            BackupKind::Manual => "manual",
            BackupKind::BeforeSave => "before-save",
            BackupKind::BeforeRestore => "before-restore",
            BackupKind::BeforeInstall => "before-install",
        }
    }
}

pub(crate) fn backup_kind_from_name(name: &str) -> String {
    let marker = "backup-rime-studio-";
    let Some(rest) = name.strip_prefix(marker) else {
        return BackupKind::Manual.as_str().to_string();
    };

    if rest.starts_with("before-save-") {
        BackupKind::BeforeSave.as_str().to_string()
    } else if rest.starts_with("before-restore-") {
        BackupKind::BeforeRestore.as_str().to_string()
    } else if rest.starts_with("before-install-") {
        BackupKind::BeforeInstall.as_str().to_string()
    } else {
        BackupKind::Manual.as_str().to_string()
    }
}

pub(crate) fn create_unique_backup_dir(
    backup_root: &Path,
    kind: BackupKind,
) -> Result<PathBuf, RimeError> {
    for suffix in 0..100 {
        let base = format!("backup-rime-studio-{}-{}", kind.as_str(), timestamp());
        let name = if suffix == 0 {
            base
        } else {
            format!("{base}-{suffix}")
        };
        let path = backup_root.join(name);
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(RimeError::BackupError(format!("创建备份目录失败: {err}"))),
        }
    }

    Err(RimeError::BackupError(
        "创建备份目录失败: 无法生成唯一目录名".to_string(),
    ))
}

pub(crate) fn is_auto_backup_kind(kind: &str) -> bool {
    matches!(kind, "before-save" | "before-restore" | "before-install")
}

pub(crate) fn backup_dir_modified(path: &Path) -> Option<u64> {
    fs::metadata(path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
}

pub(crate) fn prune_old_auto_backups(
    backup_root: &Path,
    keep_limit: usize,
) -> Result<usize, RimeError> {
    if !backup_root.exists() {
        return Ok(0);
    }

    let mut auto_backups = Vec::new();
    for entry in fs::read_dir(backup_root)
        .map_err(|err| RimeError::BackupError(format!("读取备份目录失败: {err}")))?
    {
        let entry =
            entry.map_err(|err| RimeError::BackupError(format!("检查备份目录失败: {err}")))?;
        let path = entry.path();
        if fs::symlink_metadata(&path)?.file_type().is_symlink() || !path.is_dir() {
            continue;
        }

        let Some(name) = path.file_name().and_then(OsStr::to_str) else {
            continue;
        };
        if !name.starts_with("backup-rime-studio-") {
            continue;
        }
        if !is_auto_backup_kind(&backup_kind_from_name(name)) {
            continue;
        }

        auto_backups.push((backup_dir_modified(&path), name.to_string(), path));
    }

    auto_backups.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
    let mut removed = 0usize;
    for (_, _, path) in auto_backups.into_iter().skip(keep_limit) {
        fs::remove_dir_all(&path)
            .map_err(|err| RimeError::BackupError(format!("清理旧自动备份失败: {err}")))?;
        removed += 1;
    }

    Ok(removed)
}

pub(crate) fn write_text_file(path: &Path, contents: &str, context: &str) -> Result<(), RimeError> {
    write_file_atomically(path, context, |file| {
        file.write_all(contents.as_bytes())
            .map_err(|err| RimeError::FileOperationError(format!("{context}: {err}")))
    })
}

pub(crate) fn write_file_atomically(
    path: &Path,
    context: &str,
    write: impl FnOnce(&mut fs::File) -> Result<(), RimeError>,
) -> Result<(), RimeError> {
    let parent = path
        .parent()
        .ok_or_else(|| RimeError::FileOperationError(format!("{context}: 目标路径无效")))?;
    fs::create_dir_all(parent)
        .map_err(|err| RimeError::FileOperationError(format!("{context}: 创建目录失败: {err}")))?;

    let file_name = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| RimeError::FileOperationError(format!("{context}: 文件名无效")))?;
    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);
    let (temp_path, mut temp_file) = loop {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let temp_path = parent.join(format!(".{file_name}.{}.{id}.tmp", process::id()));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(file) => break (temp_path, file),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(RimeError::FileOperationError(format!("{context}: {err}"))),
        }
    };
    let write_result = write(&mut temp_file).and_then(|()| {
        temp_file
            .sync_all()
            .map_err(|err| RimeError::FileOperationError(format!("{context}: {err}")))
    });
    drop(temp_file);
    if let Err(err) = write_result {
        let _ = fs::remove_file(&temp_path);
        return Err(err);
    }
    // rename replaces files on Windows too. Never delete the original on failure:
    // a locked or inaccessible target must remain intact.
    if let Err(err) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(RimeError::FileOperationError(format!("{context}: {err}")));
    }

    Ok(())
}

pub(crate) fn is_managed_config_file(name: &str) -> bool {
    name.ends_with(".custom.yaml")
        || name.ends_with(".dict.yaml")
        || name == "custom_phrase.txt"
        || name == "default.yaml"
        || name == "weasel.yaml"
}

pub(crate) fn is_manual_backup_file(name: &str) -> bool {
    is_managed_config_file(name) || is_editable_config_name(name)
}

pub(crate) fn backup_scope_label(_kind: &str) -> String {
    "配置快照（含子目录）：YAML、YML、TXT、Lua 配置、词库、短语、方案、installation.yaml。不含 build/、sync/、*.userdb 和链接目录。".to_string()
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct BackupMeta {
    note: Option<String>,
    kind: String,
    created_at: u64,
}

fn write_backup_meta(dir: &Path, kind: BackupKind, note: Option<&str>) -> Result<(), RimeError> {
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let meta = BackupMeta {
        note: note
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string),
        kind: kind.as_str().to_string(),
        created_at,
    };
    let payload = serde_json::to_string_pretty(&meta)
        .map_err(|err| RimeError::BackupError(format!("写入备份备注失败: {err}")))?;
    fs::write(dir.join("backup-meta.json"), payload)
        .map_err(|err| RimeError::BackupError(format!("写入备份备注失败: {err}")))
}

fn read_backup_meta(dir: &Path) -> BackupMeta {
    fs::read_to_string(dir.join("backup-meta.json"))
        .ok()
        .and_then(|contents| serde_json::from_str(&contents).ok())
        .unwrap_or_default()
}

/// Snapshot files keep their relative paths; generated data and links are excluded.
pub(crate) fn collect_snapshot_files(root: &Path) -> Result<Vec<PathBuf>, RimeError> {
    let mut files = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir)
            .map_err(|err| RimeError::BackupError(format!("读取快照目录失败: {err}")))?
        {
            let entry =
                entry.map_err(|err| RimeError::BackupError(format!("读取快照文件失败: {err}")))?;
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                continue;
            }
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if file_type.is_dir() {
                if name.eq_ignore_ascii_case("build")
                    || name.eq_ignore_ascii_case("sync")
                    || name.ends_with(".userdb")
                    || name.starts_with('.')
                    || name.starts_with("backup-")
                {
                    continue;
                }
                pending.push(path);
            } else if file_type.is_file() && is_manual_backup_file(&name) {
                files.push(
                    path.strip_prefix(root)
                        .map_err(|err| RimeError::BackupError(err.to_string()))?
                        .to_path_buf(),
                );
            }
        }
    }
    files.sort();
    Ok(files)
}

pub(crate) fn backup_user_config(user_dir: &Path, kind: BackupKind) -> Result<PathBuf, RimeError> {
    backup_user_config_with_note(user_dir, kind, None)
}

pub(crate) fn backup_user_config_with_note(
    user_dir: &Path,
    kind: BackupKind,
    note: Option<&str>,
) -> Result<PathBuf, RimeError> {
    let _guard = lock_backup_operation()?;
    backup_user_config_with_note_unlocked(user_dir, kind, note)
}

fn backup_user_config_with_note_unlocked(
    user_dir: &Path,
    kind: BackupKind,
    note: Option<&str>,
) -> Result<PathBuf, RimeError> {
    let backup_root = app_data_dir()?;
    fs::create_dir_all(&backup_root)
        .map_err(|err| RimeError::BackupError(format!("创建备份根目录失败: {err}")))?;
    let backup_dir = create_unique_backup_dir(&backup_root, kind)?;

    let snapshot_result = (|| {
        for relative in collect_snapshot_files(user_dir)? {
            let target = backup_dir.join(&relative);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(user_dir.join(&relative), target).map_err(|err| {
                RimeError::BackupError(format!("备份 {} 失败: {err}", relative.display()))
            })?;
        }
        Ok::<(), RimeError>(())
    })();
    if let Err(err) = snapshot_result {
        let _ = fs::remove_dir_all(&backup_dir);
        return Err(err);
    }

    write_backup_meta(&backup_dir, kind, note)?;

    // Pruning here during a restore can delete the selected source backup
    // before restore_backup_dir has read it. Defer pruning until it is copied.
    if !matches!(kind, BackupKind::Manual | BackupKind::BeforeRestore) {
        let _ = prune_old_auto_backups(&backup_root, AUTO_BACKUP_KEEP_LIMIT);
    }

    Ok(backup_dir)
}

pub(crate) fn list_backup_dirs(_user_dir: &Path) -> Result<Vec<BackupEntry>, RimeError> {
    let backup_root = app_data_dir()?;
    if !backup_root.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    for entry in fs::read_dir(&backup_root)
        .map_err(|err| RimeError::BackupError(format!("读取备份目录失败: {err}")))?
    {
        let entry =
            entry.map_err(|err| RimeError::BackupError(format!("检查 Rime 文件失败: {err}")))?;
        let path = entry.path();
        if fs::symlink_metadata(&path)?.file_type().is_symlink() || !path.is_dir() {
            continue;
        }

        let Some(name) = path.file_name().and_then(OsStr::to_str) else {
            continue;
        };
        if !name.starts_with("backup-rime-studio-") {
            continue;
        }

        let modified = entry
            .metadata()
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs());
        let files = collect_snapshot_files(&path)?.len();

        let kind = backup_kind_from_name(name);
        let meta = read_backup_meta(&path);
        backups.push(BackupEntry {
            name: name.to_string(),
            path: path.display().to_string(),
            kind: kind.clone(),
            modified,
            files,
            scope: backup_scope_label(&kind),
            note: meta.note,
        });
    }

    backups.sort_by_key(|right| std::cmp::Reverse(right.modified));
    Ok(backups)
}

pub(crate) fn validated_backup_dir(
    _user_dir: &Path,
    backup_name: &str,
) -> Result<PathBuf, RimeError> {
    if !backup_name.starts_with("backup-rime-studio-")
        || backup_name.contains('/')
        || backup_name.contains('\\')
        || backup_name.contains("..")
    {
        return Err(RimeError::BackupError("无效的备份名称".to_string()));
    }

    let backup_root = app_data_dir()?;
    if fs::symlink_metadata(backup_root.join(backup_name))?
        .file_type()
        .is_symlink()
    {
        return Err(RimeError::BackupError("备份目录不能是链接".into()));
    }
    let backup_dir = resolve_user_relative_path(&backup_root, backup_name, true)
        .map_err(|err| RimeError::BackupError(format!("备份路径无效: {err}")))?;
    if !backup_dir.is_dir() {
        return Err(RimeError::BackupError(format!("备份不存在: {backup_name}")));
    }

    Ok(backup_dir)
}

pub(crate) fn restore_backup_dir(
    user_dir: &Path,
    backup_dir: &Path,
) -> Result<RestoreResult, RimeError> {
    let _config_guard = lock_config_write()?;
    let _backup_guard = lock_backup_operation()?;
    fs::create_dir_all(user_dir)?;
    let mut copies = Vec::new();
    for relative in collect_snapshot_files(backup_dir)? {
        let rel = relative.to_string_lossy().replace('\\', "/");
        let target = resolve_user_relative_path(user_dir, &rel, false)?;
        copies.push((backup_dir.join(relative), target));
    }
    // Include schema, Lua and installation files that restoration may replace.
    // Do not prune the source or safety snapshot while restoring.
    let safety_backup_dir =
        backup_user_config_with_note_unlocked(user_dir, BackupKind::BeforeRestore, None)?;
    for (source, target) in &copies {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        write_file_atomically(target, "恢复备份文件失败", |destination| {
            let mut input = fs::File::open(source)?;
            std::io::copy(&mut input, destination)?;
            Ok(())
        })?;
    }
    let restored_files = copies.len();

    Ok(RestoreResult {
        restored_files,
        safety_backup_dir: safety_backup_dir.display().to_string(),
    })
}

#[cfg(test)]
mod robustness_tests {
    use super::*;

    #[test]
    fn restores_old_backup_before_pruning_it() {
        const CHILD_ROOT: &str = "RIME_STUDIO_RESTORE_TEST_ROOT";
        if let Some(root) = std::env::var_os(CHILD_ROOT) {
            let root = PathBuf::from(root);
            let user_dir = root.join("user");
            let app_dir = app_data_dir().expect("isolated app directory");
            let source = app_dir.join("backup-rime-studio-before-install-0");
            let result = restore_backup_dir(&user_dir, &source).expect("restore oldest backup");
            assert_eq!(result.restored_files, 1);
            assert_eq!(
                fs::read_to_string(user_dir.join("default.custom.yaml")).expect("read restored"),
                "old configuration"
            );
            return;
        }
        // Run with isolated environment variables in a child process, without
        // mutating the environment of other parallel tests.
        let root = std::env::temp_dir().join(format!("rime-restore-prune-{}", process::id()));
        let app_dir = root.join("RimeStudio");
        let user_dir = root.join("user");
        fs::create_dir_all(&user_dir).expect("create user directory");
        fs::create_dir_all(&app_dir).expect("create app directory");
        fs::write(
            user_dir.join("default.custom.yaml"),
            "current configuration",
        )
        .expect("write current");
        let source = app_dir.join("backup-rime-studio-before-install-0");
        fs::create_dir(&source).expect("create oldest backup");
        fs::write(source.join("default.custom.yaml"), "old configuration").expect("write backup");
        for index in 1..AUTO_BACKUP_KEEP_LIMIT {
            fs::create_dir(app_dir.join(format!("backup-rime-studio-before-save-{index}")))
                .expect("create newer backup");
        }
        let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .arg("restores_old_backup_before_pruning_it")
            .arg("--test-threads=1")
            .env(CHILD_ROOT, &root)
            .env("LOCALAPPDATA", &root)
            .output()
            .expect("run isolated restore test");
        fs::remove_dir_all(root).expect("cleanup");
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn concurrent_backups_get_distinct_directories() {
        let root = std::env::temp_dir().join(format!("rime-backup-concurrent-{}", process::id()));
        fs::create_dir_all(&root).expect("create fixture");
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let root = root.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    create_unique_backup_dir(&root, BackupKind::BeforeSave).expect("create backup")
                })
            })
            .collect();
        let paths: std::collections::HashSet<_> = handles
            .into_iter()
            .map(|handle| handle.join().expect("join"))
            .collect();
        assert_eq!(paths.len(), 8);
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn concurrent_writes_leave_a_complete_file_and_no_temporary_files() {
        let root = std::env::temp_dir().join(format!("rime-write-concurrent-{}", process::id()));
        fs::create_dir_all(&root).expect("create fixture");
        let path = root.join("test.yaml");
        let handles: Vec<_> = (0..8)
            .map(|id| {
                let path = path.clone();
                std::thread::spawn(move || {
                    write_text_file(&path, &id.to_string().repeat(10000), "test")
                })
            })
            .collect();
        for handle in handles {
            handle.join().expect("join").expect("write");
        }
        let contents = fs::read_to_string(&path).expect("read");
        assert_eq!(contents.len(), 10000);
        assert!(contents.bytes().all(|byte| byte == contents.as_bytes()[0]));
        assert_eq!(fs::read_dir(&root).expect("list").count(), 1);
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn failed_replacement_preserves_target_and_cleans_temporary_file() {
        let root = std::env::temp_dir().join(format!("rime-write-failure-{}", process::id()));
        let path = root.join("test.yaml");
        fs::create_dir_all(&path).expect("create target directory");
        fs::write(path.join("original"), "keep").expect("write original");
        assert!(write_text_file(&path, "new", "test").is_err());
        assert_eq!(
            fs::read_to_string(path.join("original")).expect("read"),
            "keep"
        );
        assert_eq!(fs::read_dir(&root).expect("list").count(), 1);
        fs::remove_dir_all(root).expect("cleanup");
    }
}
