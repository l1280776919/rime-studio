use crate::types::RimeError;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

/// Exact content snapshots avoid hash collisions and distinguish a missing file
/// from an empty file. They are local IPC data, never diagnostic report content.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct FileRevision {
    pub content: Option<String>,
}

pub(crate) fn read_file_revision(path: &Path) -> Result<FileRevision, RimeError> {
    let content = match fs::read_to_string(path) {
        Ok(content) => Some(content),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    Ok(FileRevision { content })
}

/// Call under the shared configuration write lock, immediately before writing.
/// Even an explicit overwrite must compare against the version the user reviewed.
pub(crate) fn check_file_revision(path: &Path, expected: &FileRevision) -> Result<(), RimeError> {
    if read_file_revision(path)? != *expected {
        return Err(RimeError::ConfigConflict(
            "磁盘文件已变化，请比较差异后重新加载或明确覆盖；草稿已保留".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_edits_deletions_and_creation_since_read() -> Result<(), Box<dyn std::error::Error>> {
        let root = std::env::temp_dir().join(format!("rime-revision-{}", std::process::id()));
        fs::create_dir_all(&root)?;
        let file = root.join("config.txt");
        let missing = FileRevision { content: None };
        fs::write(&file, "")?;
        assert!(check_file_revision(&file, &missing).is_err());
        let empty = read_file_revision(&file)?;
        fs::write(&file, "external edit")?;
        assert!(check_file_revision(&file, &empty).is_err());
        let current = read_file_revision(&file)?;
        assert!(check_file_revision(&file, &current).is_ok());
        fs::remove_file(&file)?;
        assert!(check_file_revision(&file, &current).is_err());
        fs::remove_dir(&root)?;
        Ok(())
    }
}
