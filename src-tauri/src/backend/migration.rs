//! Migration plans hold the reviewed source and target revisions in memory.
//! Import consumes a short-lived plan, backs up first and rolls back partial writes.
use crate::backend::*;
use crate::types::RimeError;
use serde::Serialize;
use std::{
    collections::{BTreeMap, HashSet, VecDeque},
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    time::{Duration, Instant},
};

static NEXT_MIGRATION_ID: AtomicU64 = AtomicU64::new(0);
static MIGRATION_PLAN: Mutex<Option<MigrationPlan>> = Mutex::new(None);

#[derive(Serialize)]
pub(crate) struct MigrationExport {
    pub path: String,
    pub files: usize,
    pub bytes: usize,
}

#[derive(Serialize)]
pub(crate) struct MigrationPreviewFile {
    pub name: String,
    pub category: String,
    pub bytes: usize,
    pub status: String,
    pub selected: bool,
    pub diff: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct MigrationPreview {
    pub token: String,
    pub app_version: String,
    pub created_at: String,
    pub files: Vec<MigrationPreviewFile>,
    pub blockers: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct MigrationResult {
    pub imported_files: usize,
    pub safety_backup_dir: String,
}

struct PlannedMigrationFile {
    name: String,
    content: String,
    expected: FileRevision,
}

struct MigrationPlan {
    token: String,
    created: Instant,
    user_dir: PathBuf,
    files: Vec<PlannedMigrationFile>,
    blockers: Vec<String>,
}

/// Bounded UTF-8 reads distinguish a missing file from an unreadable one and
/// protect preview memory even when the destination contains oversized files.
fn migration_revision(path: &Path) -> Result<FileRevision, RimeError> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(FileRevision { content: None })
        }
        Err(error) => return Err(error.into()),
    };
    if file.metadata()?.len() > MIGRATION_MAX_FILE as u64 {
        return Err(migration_error("配置单文件不能超过 64 MiB"));
    }
    let mut content = String::new();
    file.take(MIGRATION_MAX_FILE as u64 + 1)
        .read_to_string(&mut content)?;
    if content.len() > MIGRATION_MAX_FILE {
        return Err(migration_error("配置单文件不能超过 64 MiB"));
    }
    Ok(FileRevision {
        content: Some(content),
    })
}

pub(crate) fn list_migration_files_sync() -> Result<Vec<MigrationFileInfo>, RimeError> {
    let user = rime_user_dir()?;
    if !user.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for relative in collect_snapshot_files(&user)? {
        let name = relative.to_string_lossy().replace('\\', "/");
        if let Some(category) = migration_category(&name) {
            let path = resolve_user_relative_path(&user, &name, true)?;
            files.push(MigrationFileInfo {
                name,
                category: category.into(),
                bytes: fs::metadata(path)?.len() as usize,
            });
        }
    }
    Ok(files)
}

pub(crate) fn export_migration_sync(categories: Vec<String>) -> Result<MigrationExport, RimeError> {
    if categories.is_empty()
        || categories.iter().any(|category| {
            !["config", "schemas", "dictionaries", "lua", "phrases"].contains(&category.as_str())
        })
    {
        return Err(migration_error("请选择有效的导出类别"));
    }
    let _guard = lock_config_write()?;
    let user = rime_user_dir()?;
    let mut files = BTreeMap::new();
    let mut total = 0usize;
    for info in list_migration_files_sync()? {
        if !categories.contains(&info.category) {
            continue;
        }
        let path = resolve_user_relative_path(&user, &info.name, true)?;
        let content = migration_revision(&path)?
            .content
            .ok_or_else(|| migration_error("导出期间文件已被删除，请重试"))?;
        total = total.saturating_add(content.len());
        if total > MIGRATION_MAX_TOTAL || files.len() >= MIGRATION_MAX_FILES {
            return Err(migration_error(
                "导出内容超过 256 MiB 或 1000 个文件，请减少类别",
            ));
        }
        files.insert(info.name, content);
    }
    if files.is_empty() {
        return Err(migration_error("所选类别中没有可导出的文件"));
    }
    let data = encode_migration_archive(&files)?;
    let root = migration_export_dir()?;
    let id = NEXT_MIGRATION_ID.fetch_add(1, Ordering::Relaxed);
    let path = root.join(format!(
        "rime-migration-{}-{}-{id}.zip",
        timestamp(),
        std::process::id()
    ));
    // Reserve the name so an existing export can never be overwritten.
    drop(
        fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?,
    );
    if let Err(error) = write_file_atomically(&path, "保存迁移包失败", |file| {
        file.write_all(&data)?;
        Ok(())
    }) {
        let _ = fs::remove_file(&path);
        return Err(error);
    }
    Ok(MigrationExport {
        path: path.display().to_string(),
        files: files.len(),
        bytes: data.len(),
    })
}

fn migration_export_dir() -> Result<PathBuf, RimeError> {
    let root = app_data_dir()?;
    fs::create_dir_all(&root)?;
    let path = resolve_user_relative_path(&root, "migration-exports", false)?;
    fs::create_dir_all(&path)?;
    Ok(path)
}

pub(crate) fn open_migration_export_dir_sync() -> Result<(), RimeError> {
    open_in_explorer(&migration_export_dir()?)
}

/// Resolve the effective post-import tree: selected source files override local
/// files; skipped files and shared Rime resources remain available as dependencies.
fn migration_dependency_errors(
    user: &Path,
    selected: &BTreeMap<String, String>,
    shared: &[PathBuf],
) -> Vec<String> {
    let selected_by_name: BTreeMap<_, _> = selected
        .iter()
        .map(|(name, content)| (name.to_lowercase(), content))
        .collect();
    let mut queue: VecDeque<(String, String)> = selected
        .keys()
        .filter(|name| {
            let lower = name.to_lowercase();
            lower.ends_with(".yaml") || lower.ends_with(".yml")
        })
        .map(|name| (name.clone(), name.clone()))
        .collect();
    let mut visited = HashSet::new();
    let mut errors = Vec::new();
    while let Some((name, source)) = queue.pop_front() {
        let lower_name = name.to_lowercase();
        if !visited.insert(name.to_lowercase()) {
            continue;
        }
        if visited.len() > MIGRATION_MAX_FILES {
            errors.push("依赖超过 1000 个文件，无法完成检查".into());
            break;
        }
        let content = if let Some(content) = selected_by_name.get(&name.to_lowercase()) {
            Some((*content).clone())
        } else {
            let mut found = None;
            for root in std::iter::once(user).chain(shared.iter().map(PathBuf::as_path)) {
                if !root.exists() {
                    continue;
                }
                match resolve_user_relative_path(root, &name, false) {
                    Ok(path) if path.exists() => {
                        match read_yaml_header(&path, lower_name.ends_with(".dict.yaml")) {
                            Ok(content) => found = Some(content),
                            Err(_) => errors
                                .push(format!("{source} → {name}：依赖无法读取或超过检查限制")),
                        }
                        break;
                    }
                    Ok(_) => {}
                    Err(_) => {
                        errors.push(format!("{source} → {name}：依赖路径无效"));
                        break;
                    }
                }
            }
            found
        };
        let Some(content) = content else {
            errors.push(format!(
                "{source} → {name}：缺少依赖，请加入迁移包或先安装对应方案"
            ));
            continue;
        };
        let header = if lower_name.ends_with(".dict.yaml") {
            split_dictionary_header(&content).0
        } else {
            &content
        };
        match serde_yaml::from_str::<serde_yaml::Value>(header) {
            Ok(yaml) => {
                for (dependency, required) in config_yaml_dependencies(&lower_name, header, &yaml) {
                    if required {
                        queue.push_back((dependency, name.clone()));
                    }
                }
            }
            Err(error) => errors.push(format!("{name}：YAML 错误 {error}")),
        }
    }
    errors.sort();
    errors.dedup();
    errors
}

pub(crate) fn preview_migration_sync(
    data: Vec<u8>,
    selected_names: Option<Vec<String>>,
) -> Result<MigrationPreview, RimeError> {
    let (manifest, contents) = decode_migration_archive(data)?;
    let selected_names = selected_names.map(|names| names.into_iter().collect::<HashSet<_>>());
    if selected_names
        .as_ref()
        .is_some_and(|names| names.iter().any(|name| !contents.contains_key(name)))
    {
        return Err(migration_error("所选文件不在迁移包中"));
    }
    let _guard = lock_config_write()?;
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir)?;
    let user_dir = fs::canonicalize(user_dir)?;
    let mut files = Vec::new();
    let mut plan_files = Vec::new();
    let mut selected = BTreeMap::new();
    let mut blockers = Vec::new();
    let mut target_bytes = 0usize;
    for (name, content) in contents {
        let target = resolve_user_relative_path(&user_dir, &name, false)?;
        let expected = migration_revision(&target)?;
        let status = match &expected.content {
            None => "new",
            Some(old) if old == &content => "same",
            Some(_) => "conflict",
        };
        let is_selected = status != "same"
            && selected_names
                .as_ref()
                .map(|names| names.contains(&name))
                .unwrap_or(status == "new");
        let mut diff = Vec::new();
        if is_selected {
            target_bytes =
                target_bytes.saturating_add(expected.content.as_ref().map_or(0, String::len));
            if target_bytes > MIGRATION_MAX_TOTAL {
                return Err(migration_error("目标文件总大小超过预览限制"));
            }
            if let Err(error) = validate_config_content(&name.to_lowercase(), &content) {
                blockers.push(format!("{name}：{error}"));
            }
            let old = expected.content.as_deref().unwrap_or_default();
            if old.len() + content.len() <= 256 * 1024 {
                diff = build_text_diff(old, &content);
                if diff.len() > 120 {
                    diff.truncate(120);
                    diff.push("… 差异过长，仅展示前 120 行".into());
                }
            } else {
                diff.push("大文件省略行差异；导入将写入迁移包中的完整文件".into());
            }
            selected.insert(name.clone(), content.clone());
            plan_files.push(PlannedMigrationFile {
                name: name.clone(),
                content: content.clone(),
                expected,
            });
        }
        files.push(MigrationPreviewFile {
            category: migration_category(&name).unwrap_or("config").into(),
            name,
            bytes: content.len(),
            status: status.into(),
            selected: is_selected,
            diff,
        });
    }
    blockers.extend(migration_dependency_errors(
        &user_dir,
        &selected,
        &system_data_dirs(),
    ));
    blockers.sort();
    blockers.dedup();
    let token = format!(
        "{}-{}",
        timestamp(),
        NEXT_MIGRATION_ID.fetch_add(1, Ordering::Relaxed)
    );
    // Import releases this mutex before acquiring the configuration lock.
    *MIGRATION_PLAN
        .lock()
        .map_err(|_| migration_error("迁移预览不可用"))? = Some(MigrationPlan {
        token: token.clone(),
        created: Instant::now(),
        user_dir,
        files: plan_files,
        blockers: blockers.clone(),
    });
    Ok(MigrationPreview {
        token,
        app_version: manifest.app_version,
        created_at: manifest.created_at,
        files,
        blockers,
    })
}

fn verify_plan_files(plan: &MigrationPlan) -> Result<(), RimeError> {
    for file in &plan.files {
        let target = resolve_user_relative_path(&plan.user_dir, &file.name, false)?;
        if migration_revision(&target)? != file.expected {
            return Err(RimeError::ConfigConflict(format!(
                "{} 在预览后发生变化，请重新预览",
                file.name
            )));
        }
    }
    Ok(())
}

/// Roll back only our own writes. If another program edits a written file during
/// failure recovery, preserve that edit and report the retained safety backup.
fn apply_migration_files(
    plan: &MigrationPlan,
    mut write: impl FnMut(&Path, &str) -> Result<(), RimeError>,
) -> Result<(), RimeError> {
    let mut applied: Vec<&PlannedMigrationFile> = Vec::new();
    for file in &plan.files {
        let result = (|| {
            let target = resolve_user_relative_path(&plan.user_dir, &file.name, false)?;
            if migration_revision(&target)? != file.expected {
                return Err(RimeError::ConfigConflict(file.name.clone()));
            }
            write(&target, &file.content)
        })();
        if let Err(error) = result {
            let mut failures = Vec::new();
            for previous in applied.into_iter().rev() {
                let restore = (|| {
                    let target = resolve_user_relative_path(&plan.user_dir, &previous.name, false)?;
                    if migration_revision(&target)?.content.as_deref()
                        != Some(previous.content.as_str())
                    {
                        return Err(migration_error("文件被外部修改"));
                    }
                    match &previous.expected.content {
                        Some(content) => write_text_file(&target, content, "回滚迁移失败"),
                        None => fs::remove_file(&target).map_err(Into::into),
                    }
                })();
                if restore.is_err() {
                    failures.push(previous.name.clone());
                }
            }
            let recovery = if failures.is_empty() {
                "已回滚本次已写入的文件".into()
            } else {
                format!(
                    "以下文件未能回滚，请从安全备份恢复：{}",
                    failures.join("、")
                )
            };
            return Err(migration_error(format!("迁移中止：{error}。{recovery}")));
        }
        applied.push(file);
    }
    Ok(())
}

pub(crate) fn import_migration_sync(token: String) -> Result<MigrationResult, RimeError> {
    let plan = {
        let mut pending = MIGRATION_PLAN
            .lock()
            .map_err(|_| migration_error("迁移预览不可用"))?;
        if !pending.as_ref().is_some_and(|plan| plan.token == token) {
            return Err(migration_error("迁移预览已失效，请重新预览"));
        }
        pending
            .take()
            .ok_or_else(|| migration_error("请先预览迁移包"))?
    };
    if plan.created.elapsed() > Duration::from_secs(600)
        || !plan.blockers.is_empty()
        || plan.files.is_empty()
    {
        return Err(migration_error(
            "预览已过期、存在未解决依赖或未选择文件，请重新预览",
        ));
    }
    let _guard = lock_config_write()?;
    if fs::canonicalize(rime_user_dir()?)? != plan.user_dir {
        return Err(migration_error("用户目录已变化，请重新预览"));
    }
    verify_plan_files(&plan)?;
    let selected = plan
        .files
        .iter()
        .map(|file| (file.name.clone(), file.content.clone()))
        .collect();
    let dependency_errors =
        migration_dependency_errors(&plan.user_dir, &selected, &system_data_dirs());
    if !dependency_errors.is_empty() {
        return Err(migration_error(format!(
            "依赖检查失败：{}",
            dependency_errors.join("；")
        )));
    }
    // A manual snapshot is retained permanently, even if importing fails.
    let safety = backup_user_config_with_note(
        &plan.user_dir,
        BackupKind::Manual,
        Some("配置迁移导入前的安全备份"),
    )?;
    verify_plan_files(&plan)?;
    apply_migration_files(&plan, |path, content| {
        write_text_file(path, content, "导入迁移文件失败")
    })
    .map_err(|error| migration_error(format!("{error}。安全备份：{}", safety.display())))?;
    Ok(MigrationResult {
        imported_files: plan.files.len(),
        safety_backup_dir: safety.display().to_string(),
    })
}

/// Closing a preview releases its contents without clearing a newer preview.
pub(crate) fn discard_migration_preview_sync(token: String) -> Result<(), RimeError> {
    let mut pending = MIGRATION_PLAN
        .lock()
        .map_err(|_| migration_error("迁移预览不可用"))?;
    if pending.as_ref().is_some_and(|plan| plan.token == token) {
        *pending = None;
    }
    Ok(())
}

#[cfg(test)]
#[path = "migration_tests.rs"]
mod tests;
