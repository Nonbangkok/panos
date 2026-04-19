use panos::file_ops::history::{MoveRecord, Session};
use panos::organizer::undo::run_undo;
use panos::ui::NoopReporter;
use std::fs;
use tempfile::TempDir;

use crate::common::test_config;

#[test]
fn test_undo_single_file() -> anyhow::Result<()> {
    // Basic single file revert
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let src = root.join("file.txt");
    let dst = root.join("dir/file.txt");

    fs::create_dir_all(dst.parent().unwrap())?;
    fs::write(&dst, "data")?;

    let session = Session {
        moves: vec![MoveRecord {
            source: src.clone(),
            destination: dst.clone(),
            timestamp: chrono::Utc::now(),
        }],
    };
    session.save(root, ".history.json")?;

    let config = test_config(root);

    run_undo(&config, false, &NoopReporter)?;
    assert!(src.exists(), "Source file should be restored at {:?}", src);
    assert!(
        !dst.exists(),
        "Destination file should be removed at {:?}",
        dst
    );
    Ok(())
}

#[test]
fn test_undo_multiple_files_filo_order() -> anyhow::Result<()> {
    // Ensure files are restored in reverse order (Last In, First Out)
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let mut moves = Vec::new();

    for i in 0..5 {
        let src = root.join(format!("src_{}.txt", i));
        let dst = root.join(format!("dst_{}.txt", i));
        fs::write(&dst, format!("data {}", i))?;
        moves.push(MoveRecord {
            source: src,
            destination: dst,
            timestamp: chrono::Utc::now(),
        });
    }

    let session = Session { moves };
    session.save(root, ".history.json")?;

    let config = test_config(root);

    run_undo(&config, false, &NoopReporter)?;

    for i in 0..5 {
        assert!(
            root.join(format!("src_{}.txt", i)).exists(),
            "File {} should be restored",
            i
        );
        assert!(
            !root.join(format!("dst_{}.txt", i)).exists(),
            "File {} should be removed from dest",
            i
        );
    }
    Ok(())
}

#[test]
fn test_undo_dry_run_no_changes() -> anyhow::Result<()> {
    // Verify dry run prevents any disk changes
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let src = root.join("test.txt");
    let dst = root.join("test_dst.txt");
    fs::write(&dst, "data")?;

    let session = Session {
        moves: vec![MoveRecord {
            source: src.clone(),
            destination: dst.clone(),
            timestamp: chrono::Utc::now(),
        }],
    };
    session.save(root, ".history.json")?;

    let config = test_config(root);

    run_undo(&config, true, &NoopReporter)?;
    assert!(!src.exists(), "Source should NOT be created in dry run");
    assert!(dst.exists(), "Destination should STILL exist in dry run");
    assert!(
        root.join(".history.json").exists(),
        "History file should NOT be deleted in dry run"
    );
    Ok(())
}

#[test]
fn test_undo_empty_session() -> anyhow::Result<()> {
    // Handle session with no moves gracefully
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let session = Session { moves: vec![] };
    session.save(root, ".history.json")?;

    let config = test_config(root);

    let result = run_undo(&config, false, &NoopReporter);
    assert!(result.is_ok(), "Empty undo should succeed without error");
    Ok(())
}

#[test]
fn test_undo_missing_history_file() -> anyhow::Result<()> {
    // Handle missing history file gracefully
    let tmp = TempDir::new()?;
    let mut config = test_config(tmp.path());
    config.source_dir = tmp.path().to_path_buf();
    config.history_file = "nonexistent.json".to_string();

    let result = run_undo(&config, false, &NoopReporter);
    assert!(
        result.is_ok(),
        "Missing history file should be treated as empty success"
    );
    Ok(())
}

#[test]
fn test_undo_missing_file_at_destination() -> anyhow::Result<()> {
    // Skip records where file was already moved or deleted from dest
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let src = root.join("missing.txt");
    let dst = root.join("missing_dst.txt");
    // File NOT created at dst

    let session = Session {
        moves: vec![MoveRecord {
            source: src.clone(),
            destination: dst.clone(),
            timestamp: chrono::Utc::now(),
        }],
    };
    session.save(root, ".history.json")?;

    let config = test_config(root);

    let result = run_undo(&config, false, &NoopReporter);
    assert!(
        result.is_ok(),
        "Should skip missing destination files without failing"
    );
    assert!(
        !src.exists(),
        "Source should not be created if destination was missing"
    );
    Ok(())
}

#[test]
fn test_undo_massive_batch_500() -> anyhow::Result<()> {
    // Stress test with large number of files
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let mut moves = Vec::new();

    for i in 0..500 {
        let src = root.join(format!("batch/src_{}.dat", i));
        let dst = root.join(format!("batch/dst_{}.dat", i));
        fs::create_dir_all(dst.parent().unwrap())?;
        fs::write(&dst, "binary data")?;
        moves.push(MoveRecord {
            source: src,
            destination: dst,
            timestamp: chrono::Utc::now(),
        });
    }

    let session = Session { moves };
    session.save(root, ".history.json")?;

    let config = test_config(root);

    run_undo(&config, false, &NoopReporter)?;
    assert_eq!(
        fs::read_dir(root.join("batch"))?.count(),
        500,
        "All 500 files should be back in source folder level"
    );
    Ok(())
}

#[test]
fn test_undo_unicode_paths() -> anyhow::Result<()> {
    // Ensure non-ASCII paths are handled correctly
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let src = root.join("เอกสาร.pdf");
    let dst = root.join("จัดเก็บ/เอกสาร.pdf");

    fs::create_dir_all(dst.parent().unwrap())?;
    fs::write(&dst, "unicode content")?;

    let session = Session {
        moves: vec![MoveRecord {
            source: src.clone(),
            destination: dst.clone(),
            timestamp: chrono::Utc::now(),
        }],
    };
    session.save(root, "history.json")?;

    let mut config = test_config(root);
    config.history_file = "history.json".to_string();

    run_undo(&config, false, &NoopReporter)?;
    assert!(
        src.exists(),
        "Unicode filename should be restored correctly"
    );
    Ok(())
}

#[test]
fn test_undo_spaces_and_special_chars() -> anyhow::Result<()> {
    // Handle tricky filenames
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let src = root.join("my file @ # $ % ^.txt");
    let dst = root.join("dest path/my file @ # $ % ^.txt");

    fs::create_dir_all(dst.parent().unwrap())?;
    fs::write(&dst, "special chars")?;

    let session = Session {
        moves: vec![MoveRecord {
            source: src.clone(),
            destination: dst.clone(),
            timestamp: chrono::Utc::now(),
        }],
    };
    session.save(root, "history.json")?;

    let mut config = test_config(root);
    config.history_file = "history.json".to_string();

    run_undo(&config, false, &NoopReporter)?;
    assert!(
        src.exists(),
        "Filename with spaces and symbols should be restored"
    );
    Ok(())
}

#[test]
fn test_undo_deeply_nested_revert() -> anyhow::Result<()> {
    // Revert file from deep folder structure
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let src = root.join("a/b/c/d/e/file.txt");
    let dst = root.join("final/file.txt");

    fs::create_dir_all(dst.parent().unwrap())?;
    fs::write(&dst, "deep")?;

    let session = Session {
        moves: vec![MoveRecord {
            source: src.clone(),
            destination: dst.clone(),
            timestamp: chrono::Utc::now(),
        }],
    };
    session.save(root, "history.json")?;

    let mut config = test_config(root);
    config.history_file = "history.json".to_string();

    run_undo(&config, false, &NoopReporter)?;
    assert!(
        src.exists(),
        "Deeply nested source path should be recreated and file restored"
    );
    Ok(())
}

#[test]
fn test_undo_history_cleanup_after_success() -> anyhow::Result<()> {
    // Ensure history file is deleted only on real run
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let h_file = "cleanup.json";
    let session = Session { moves: vec![] };
    session.save(root, h_file)?;

    let mut config = test_config(root);
    config.history_file = h_file.to_string();

    run_undo(&config, false, &NoopReporter)?;
    assert!(
        !root.join(h_file).exists(),
        "History file must be deleted after successful operation"
    );
    Ok(())
}

#[test]
fn test_undo_partial_success_mixed() -> anyhow::Result<()> {
    // Undo some files while others are missing
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let src1 = root.join("exist.txt");
    let dst1 = root.join("exist_dst.txt");
    let src2 = root.join("missing.txt");
    let dst2 = root.join("missing_dst.txt");

    fs::write(&dst1, "i exist")?;

    let session = Session {
        moves: vec![
            MoveRecord {
                source: src1.clone(),
                destination: dst1.clone(),
                timestamp: chrono::Utc::now(),
            },
            MoveRecord {
                source: src2.clone(),
                destination: dst2.clone(),
                timestamp: chrono::Utc::now(),
            },
        ],
    };
    session.save(root, "mixed.json")?;

    let mut config = test_config(root);
    config.history_file = "mixed.json".to_string();

    run_undo(&config, false, &NoopReporter)?;
    assert!(src1.exists(), "Existing file should be reverted");
    assert!(!src2.exists(), "Missing file should still be missing");
    Ok(())
}

#[test]
fn test_undo_source_parent_creation() -> anyhow::Result<()> {
    // Recreate parent directory of source if it was deleted
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let src = root.join("new_parent/file.txt");
    let dst = root.join("destination/file.txt");

    fs::create_dir_all(dst.parent().unwrap())?;
    fs::write(&dst, "data")?;

    let session = Session {
        moves: vec![MoveRecord {
            source: src.clone(),
            destination: dst.clone(),
            timestamp: chrono::Utc::now(),
        }],
    };
    session.save(root, "history.json")?;

    let mut config = test_config(root);
    config.history_file = "history.json".to_string();

    run_undo(&config, false, &NoopReporter)?;
    assert!(
        src.parent().unwrap().exists(),
        "Source parent directory should be automatically recreated"
    );
    assert!(
        src.exists(),
        "File should be restored inside recreated directory"
    );
    Ok(())
}

#[test]
fn test_undo_revert_from_trash() -> anyhow::Result<()> {
    // Revert files that went to trash
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let src = root.join("photo.jpg");
    let trash = root.join(".trash/photo.jpg");

    fs::create_dir_all(trash.parent().unwrap())?;
    fs::write(&trash, "trash content")?;

    let session = Session {
        moves: vec![MoveRecord {
            source: src.clone(),
            destination: trash.clone(),
            timestamp: chrono::Utc::now(),
        }],
    };
    session.save(root, "history.json")?;

    let mut config = test_config(root);
    config.history_file = "history.json".to_string();

    run_undo(&config, false, &NoopReporter)?;
    assert!(
        src.exists(),
        "File should be moved back from trash directory"
    );
    Ok(())
}

#[test]
fn test_undo_revert_from_unknown() -> anyhow::Result<()> {
    // Revert files that went to unknown dir
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let src = root.join("mystery.bin");
    let unknown = root.join(".unknown/mystery.bin");

    fs::create_dir_all(unknown.parent().unwrap())?;
    fs::write(&unknown, "mystery")?;

    let session = Session {
        moves: vec![MoveRecord {
            source: src.clone(),
            destination: unknown.clone(),
            timestamp: chrono::Utc::now(),
        }],
    };
    session.save(root, "history.json")?;

    let mut config = test_config(root);
    config.history_file = "history.json".to_string();

    run_undo(&config, false, &NoopReporter)?;
    assert!(
        src.exists(),
        "File should be moved back from unknown directory"
    );
    Ok(())
}

#[test]
fn test_undo_idempotency() -> anyhow::Result<()> {
    // Running undo twice shouldn't fail even if history is gone
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let h_file = "idemp.json";
    let src = root.join("a.txt");
    let dst = root.join("b.txt");
    fs::write(&dst, "content")?;

    let session = Session {
        moves: vec![MoveRecord {
            source: src.clone(),
            destination: dst.clone(),
            timestamp: chrono::Utc::now(),
        }],
    };
    session.save(root, h_file)?;

    let mut config = test_config(root);
    config.history_file = h_file.to_string();

    run_undo(&config, false, &NoopReporter)?; // First run
    assert!(src.exists(), "First undo should succeed");

    let result = run_undo(&config, false, &NoopReporter); // Second run
    assert!(
        result.is_ok(),
        "Second undo run with missing history should still be Ok"
    );
    Ok(())
}

#[test]
fn test_undo_large_filename() -> anyhow::Result<()> {
    // Handle extremely long filenames
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let long_name = "a".repeat(200) + ".txt";
    let src = root.join(&long_name);
    let dst = root.join("dst").join(&long_name);

    fs::create_dir_all(dst.parent().unwrap())?;
    fs::write(&dst, "long")?;

    let session = Session {
        moves: vec![MoveRecord {
            source: src.clone(),
            destination: dst.clone(),
            timestamp: chrono::Utc::now(),
        }],
    };
    session.save(root, "h.json")?;

    let mut config = test_config(root);
    config.history_file = "h.json".to_string();

    run_undo(&config, false, &NoopReporter)?;
    assert!(src.exists(), "Extreme filename length should be handled");
    Ok(())
}

#[test]
fn test_undo_move_record_timestamps() -> anyhow::Result<()> {
    // Verify that timestamps don't affect undo logic
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let src = root.join("old.txt");
    let dst = root.join("new.txt");
    fs::write(&dst, "data")?;

    let old_time = chrono::Utc::now() - chrono::Duration::days(30);
    let session = Session {
        moves: vec![MoveRecord {
            source: src.clone(),
            destination: dst.clone(),
            timestamp: old_time,
        }],
    };
    session.save(root, "history.json")?;

    let mut config = test_config(root);
    config.history_file = "history.json".to_string();

    run_undo(&config, false, &NoopReporter)?;
    assert!(src.exists(), "Even old history records should be undoable");
    Ok(())
}

#[test]
fn test_undo_directory_recreation_on_revert() -> anyhow::Result<()> {
    // Ensure nested directories are recreated if source path needs them
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let src = root.join("x/y/z/file.txt");
    let dst = root.join("out/file.txt");
    fs::create_dir_all(dst.parent().unwrap())?;
    fs::write(&dst, "content")?;

    let session = Session {
        moves: vec![MoveRecord {
            source: src.clone(),
            destination: dst.clone(),
            timestamp: chrono::Utc::now(),
        }],
    };
    session.save(root, "h.json")?;

    let mut config = test_config(root);
    config.history_file = "h.json".to_string();

    run_undo(&config, false, &NoopReporter)?;
    assert!(
        src.exists(),
        "Files should be restored to deeply nested paths correctly"
    );
    Ok(())
}

#[test]
fn test_undo_unmapped_history_file() -> anyhow::Result<()> {
    // Handled case where history file name in config changed
    let tmp = TempDir::new()?;
    let mut config = test_config(tmp.path());
    config.source_dir = tmp.path().to_path_buf();
    config.history_file = "wrong_name.json".to_string();

    let result = run_undo(&config, false, &NoopReporter);
    assert!(
        result.is_ok(),
        "Changing history file name should just result in empty undo, not failure"
    );
    Ok(())
}
