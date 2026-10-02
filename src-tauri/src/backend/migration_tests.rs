use super::*;
use std::io::Cursor;

fn fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("rime-migration-{}-{name}", std::process::id()));
    fs::create_dir_all(&root).expect("fixture directory");
    root
}

fn archive_with_entries(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, content) in entries {
        zip.start_file(*name, zip::write::SimpleFileOptions::default())
            .expect("entry");
        zip.write_all(content.as_bytes()).expect("content");
    }
    zip.finish().expect("finish").into_inner()
}

#[test]
fn archive_roundtrip_preserves_unicode_paths_and_exact_contents() {
    let files = BTreeMap::from([
        (
            "词库/自定义.dict.yaml".into(),
            "---\nname: custom\n...\n私人短语\tabc\t9\n".into(),
        ),
        (
            "lua/date.lua".into(),
            "-- user script\r\nreturn {}\r\n".into(),
        ),
    ]);
    let (_, decoded) = decode_migration_archive(encode_migration_archive(&files).expect("encode"))
        .expect("decode");
    assert_eq!(decoded, files);
}

#[test]
fn archives_reject_traversal_reserved_data_aliases_and_path_collisions() {
    for path in [
        "../escape.yaml",
        "C:/escape.yaml",
        "a\\escape.yaml",
        "BUILD/default.yaml",
        "sync/words.txt",
        "luna.userdb/words.txt",
        "luna.userdb.txt",
        "installation.yaml",
        "CON.yaml",
        "a./x.yaml",
        "run.exe",
    ] {
        assert!(migration_category(path).is_none(), "{path}");
        assert!(
            decode_migration_archive(archive_with_entries(&[(&format!("files/{path}"), "x")]))
                .is_err()
        );
    }
    for files in [
        BTreeMap::from([
            ("a.yaml".into(), "a: 1".into()),
            ("A.yaml".into(), "a: 2".into()),
        ]),
        BTreeMap::from([
            ("a.yaml".into(), "a: 1".into()),
            ("a.yaml/child.txt".into(), "x".into()),
        ]),
    ] {
        assert!(
            decode_migration_archive(encode_migration_archive(&files).expect("encode")).is_err()
        );
    }
}

#[test]
fn archives_require_a_supported_manifest_matching_the_payload() {
    let manifest = |version: u32, bytes: usize| {
        serde_json::json!({
            "format": "rime-studio-migration", "format_version": version,
            "app_version": "0.11.0", "created_at": "1",
            "files": [{"name":"a.yaml", "category":"config", "bytes":bytes}],
        })
        .to_string()
    };
    for invalid in [manifest(2, 4), manifest(1, 99)] {
        assert!(decode_migration_archive(archive_with_entries(&[
            ("rime-studio-migration.json", &invalid),
            ("files/a.yaml", "a: 1")
        ]))
        .is_err());
    }
    assert!(decode_migration_archive(archive_with_entries(&[("files/a.yaml", "a: 1")])).is_err());
}

#[test]
fn dependencies_use_selected_files_and_existing_shared_resources() {
    let root = fixture("dependencies");
    let shared = root.join("shared");
    fs::create_dir_all(&shared).expect("shared");
    fs::write(
        shared.join("base.dict.yaml"),
        "---\nname: base\n...\nword\tabc\n",
    )
    .expect("dictionary");
    let mut selected = BTreeMap::from([
        (
            "default.custom.yaml".into(),
            "patch:\n  schema_list:\n    - schema: demo\n".into(),
        ),
        (
            "demo.schema.yaml".into(),
            "schema:\n  schema_id: demo\ntranslator:\n  dictionary: base\n".into(),
        ),
    ]);
    assert!(
        migration_dependency_errors(&root, &selected, std::slice::from_ref(&shared)).is_empty()
    );
    selected.remove("demo.schema.yaml");
    assert!(migration_dependency_errors(&root, &selected, &[shared])
        .iter()
        .any(|error| error.contains("demo.schema.yaml")));
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn failed_import_restores_overwrites_and_removes_new_files() {
    let root = fixture("rollback");
    fs::write(root.join("a.txt"), "original").expect("original");
    let plan = MigrationPlan {
        token: "test".into(),
        created: Instant::now(),
        user_dir: root.clone(),
        blockers: Vec::new(),
        files: vec![
            PlannedMigrationFile {
                name: "a.txt".into(),
                content: "replacement".into(),
                expected: FileRevision {
                    content: Some("original".into()),
                },
            },
            PlannedMigrationFile {
                name: "b.txt".into(),
                content: "new".into(),
                expected: FileRevision { content: None },
            },
            PlannedMigrationFile {
                name: "c.txt".into(),
                content: "fail".into(),
                expected: FileRevision { content: None },
            },
        ],
    };
    let result = apply_migration_files(&plan, |path, content| {
        if content == "fail" {
            return Err(migration_error("injected disk failure"));
        }
        write_text_file(path, content, "test")
    });
    assert!(result.is_err());
    assert_eq!(
        fs::read_to_string(root.join("a.txt")).expect("read"),
        "original"
    );
    assert!(!root.join("b.txt").exists());
    assert!(!root.join("c.txt").exists());
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn plan_verification_rejects_external_edits_before_any_write() {
    let root = fixture("stale");
    fs::write(root.join("a.txt"), "external").expect("external edit");
    let plan = MigrationPlan {
        token: "test".into(),
        created: Instant::now(),
        user_dir: root.clone(),
        blockers: Vec::new(),
        files: vec![PlannedMigrationFile {
            name: "a.txt".into(),
            content: "draft".into(),
            expected: FileRevision {
                content: Some("old".into()),
            },
        }],
    };
    assert!(matches!(
        verify_plan_files(&plan),
        Err(RimeError::ConfigConflict(_))
    ));
    assert_eq!(
        fs::read_to_string(root.join("a.txt")).expect("read"),
        "external"
    );
    fs::remove_dir_all(root).expect("cleanup");
}
