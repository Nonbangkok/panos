use panos::{
    file_ops::{Session, remove_empty_dirs},
    organizer::{organize, run_undo},
    ui::NoopReporter,
};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

mod common;
use common::{test_config, test_rule};

#[test]
fn test_full_organization_flow() -> anyhow::Result<()> {
    // 1. Setup temporary workspace
    let tmp_dir = TempDir::new()?;
    let source_path = tmp_dir.path().to_path_buf();

    // Create subdirectories
    let images_dir = "images";
    let docs_dir = "docs";
    let trash_dir = "trash";
    let unknown_dir = "unknown";

    fs::create_dir(source_path.join(images_dir))?;
    fs::create_dir(source_path.join(docs_dir))?;
    fs::create_dir(source_path.join(trash_dir))?;
    fs::create_dir(source_path.join(unknown_dir))?;

    // 2. Create test files
    let file1 = source_path.join("image1.jpg");
    let file2 = source_path.join("document1.pdf");
    let file3 = source_path.join(".tmp");
    let file4 = source_path.join("unknown.txt");
    let file5 = source_path.join("subfolder/image2.jpg");

    fs::create_dir(source_path.join("subfolder"))?;
    fs::write(&file1, "jpg content 1")?;
    fs::write(&file2, "pdf content 1")?;
    fs::write(&file3, "temp content 1")?;
    fs::write(&file4, "txt content 1")?;
    fs::write(&file5, "jpg content 2")?;

    // 3. Setup configuration
    let mut config = test_config(&source_path);
    config.rules = vec![test_rule("Images", vec!["jpg", "png"], vec![]), {
        let mut r = test_rule("Documents", vec!["pdf"], vec![]);
        r.destination = PathBuf::from(docs_dir);
        r
    }];
    config.debounce_seconds = 1;
    config.temp_extensions = vec!["tmp".to_string()];
    config.trash_dir = PathBuf::from(trash_dir);
    config.unknown_dir = PathBuf::from(unknown_dir);
    config.history_file = ".panos_history.json".to_string();
    config.exclude_hidden = false;

    // 4. Run organization
    let mut ai = None;
    let history = organize(&config, false, &NoopReporter, &mut ai)?;
    remove_empty_dirs(&source_path, false, &history, &NoopReporter)?;

    // 5. Assertions
    // Images moved
    assert!(source_path.join(images_dir).join("image1.jpg").exists());
    assert!(source_path.join(images_dir).join("image2.jpg").exists());

    // Documents moved
    assert!(source_path.join(docs_dir).join("document1.pdf").exists());

    // Temp file moved to trash
    assert!(source_path.join(trash_dir).join(".tmp").exists());

    // Unknown file moved to unknown
    assert!(source_path.join(unknown_dir).join("unknown.txt").exists());

    // Original files gone
    assert!(!file1.exists());
    assert!(!file2.exists());
    assert!(!file3.exists());
    assert!(!file4.exists());
    assert!(!file5.exists());

    // Subfolder should be removed (it was empty after moving image2.jpg)
    assert!(!source_path.join("subfolder").exists());

    Ok(())
}

#[test]
fn test_dry_run_does_not_move_files() -> anyhow::Result<()> {
    let tmp_dir = TempDir::new()?;
    let source_path = tmp_dir.path().to_path_buf();

    let file1 = source_path.join("test.jpg");
    fs::write(&file1, "content")?;

    let mut config = test_config(&source_path);
    config.rules = vec![test_rule("Images", vec!["jpg"], vec![])];
    config.debounce_seconds = 1;
    config.trash_dir = PathBuf::from("trash");
    config.unknown_dir = PathBuf::from("unknown");
    config.history_file = ".panos_history.json".to_string();
    config.exclude_hidden = false;

    // Run organization in dry_run mode
    let mut ai = None;
    organize(&config, true, &NoopReporter, &mut ai)?;

    // File should still be in source
    assert!(file1.exists());
    // Destination directory should not have the file (it might not even exist)
    assert!(!source_path.join("images").join("test.jpg").exists());

    Ok(())
}

#[test]
fn test_comprehensive_scenario() -> anyhow::Result<()> {
    let tmp_dir = TempDir::new()?;
    let source_path = tmp_dir.path().to_path_buf();

    // 1. Setup structure
    let folders = [
        "images",
        "docs",
        "archives",
        "trash",
        "unknown",
        "work/projects",
        "work/temp",
    ];
    for folder in &folders {
        fs::create_dir_all(source_path.join(folder))?;
    }

    // 2. Create various files
    // Extension matches
    fs::write(source_path.join("photo.jpg"), "photo content")?;
    fs::write(source_path.join("vacation.png"), "vacation content")?;
    fs::write(source_path.join("resume.pdf"), "resume content")?;
    fs::write(source_path.join("archive.tar.gz"), "archive content")?;

    // Pattern matches
    fs::write(source_path.join("backup_2024.zip"), "unique zip backup")?;
    fs::write(source_path.join("old_notes.zip"), "unique zip notes")?;
    fs::write(
        source_path.join("IMPORTANT_DOC.txt"),
        "unique pattern match",
    )?;

    // Nested files
    fs::write(
        source_path.join("work/projects/report.pdf"),
        "report content",
    )?;

    // Temp files
    fs::write(source_path.join("data.tmp"), "tmp")?;
    fs::write(source_path.join(".cache"), "hidden_tmp")?;
    fs::write(source_path.join("~WRL0001.tmp"), "office temp")?;

    // Unknown files
    fs::write(source_path.join("README"), "no extension")?;
    fs::write(source_path.join("script.sh"), "sh")?;

    // Conflict scenario: file already exists in destination
    fs::write(source_path.join("images/profile.jpg"), "existing")?;
    fs::write(source_path.join("profile.jpg"), "new")?;

    // Ignore patterns
    fs::write(source_path.join("node_modules"), "should be ignored folder")?;
    fs::write(source_path.join("target"), "should be ignored")?;

    // 3. Configuration
    let mut config = test_config(&source_path);
    config.rules = vec![
        {
            let mut r = test_rule("Photos", vec!["jpg", "png"], vec![]);
            r.destination = PathBuf::from("images");
            r
        },
        {
            let mut r = test_rule("Docs", vec!["pdf"], vec!["*.tar.gz"]);
            r.destination = PathBuf::from("docs");
            r
        },
        test_rule("Archives", vec![], vec!["*backup*", "old_*", "IMPORTANT*"]),
    ];
    config.debounce_seconds = 1;
    config.temp_extensions = vec!["tmp".to_string(), "cache".to_string()];
    config.ignore_patterns = vec!["node_modules".to_string(), "target".to_string()];
    config.trash_dir = PathBuf::from("trash");
    config.unknown_dir = PathBuf::from("unknown");
    config.history_file = ".panos_history.json".to_string();
    config.exclude_hidden = false;

    // 4. Run
    let mut ai = None;
    organize(&config, false, &NoopReporter, &mut ai)?;

    // 5. Verification
    assert!(source_path.join("images/photo.jpg").exists());
    assert!(source_path.join("images/vacation.png").exists());
    assert!(source_path.join("docs/resume.pdf").exists());
    assert!(source_path.join("docs/archive.tar.gz").exists());

    assert!(source_path.join("archives/backup_2024.zip").exists());
    assert!(source_path.join("archives/old_notes.zip").exists());
    assert!(source_path.join("archives/IMPORTANT_DOC.txt").exists());

    assert!(source_path.join("docs/report.pdf").exists());

    assert!(source_path.join("trash/data.tmp").exists());
    assert!(source_path.join("trash/.cache").exists());
    assert!(source_path.join("trash/~WRL0001.tmp").exists());

    // Unknown files moved to unknown
    assert!(source_path.join("unknown/README").exists());
    assert!(source_path.join("unknown/script.sh").exists());

    // Conflict handling (should rename to profile_1.jpg)
    assert!(source_path.join("images/profile.jpg").exists()); // the existing one
    assert!(source_path.join("images/profile_1.jpg").exists()); // the new one

    // Ignored files should still be in root
    assert!(source_path.join("node_modules").exists());
    assert!(source_path.join("target").exists());

    // 6. Cleanup empty dirs
    remove_empty_dirs(&source_path, false, &[], &NoopReporter)?;
    assert!(!source_path.join("work/projects").exists());
    assert!(!source_path.join("work/temp").exists());
    assert!(!source_path.join("work").exists());

    Ok(())
}

#[test]
fn test_undo_operation() -> anyhow::Result<()> {
    let tmp_dir = TempDir::new()?;
    let source_path = tmp_dir.path().to_path_buf();

    // 1. Setup
    fs::create_dir_all(source_path.join("images"))?;
    let file1 = source_path.join("photo.jpg");
    fs::write(&file1, "content")?;

    let mut config = test_config(&source_path);
    config.rules = vec![{
        let mut r = test_rule("Photos", vec!["jpg"], vec![]);
        r.destination = PathBuf::from("images");
        r
    }];
    config.debounce_seconds = 1;
    config.trash_dir = PathBuf::from("trash");
    config.unknown_dir = PathBuf::from("unknown");
    config.history_file = ".panos_history.json".to_string();
    config.exclude_hidden = false;

    // 2. Organize
    let mut ai = None;
    let history = organize(&config, false, &NoopReporter, &mut ai)?;
    assert!(!history.is_empty());

    // Save session manually (simulating main.rs behavior)
    let session = Session { moves: history };
    session.save(&config.source_dir, &config.history_file)?;

    // Verify file moved
    let moved_file = source_path.join("images/photo.jpg");
    assert!(moved_file.exists());
    assert!(!file1.exists());

    // 3. Undo
    run_undo(&config, false, &NoopReporter)?;

    // 4. Verify restored
    assert!(file1.exists());
    assert!(!moved_file.exists());

    // Verify history file deleted
    assert!(!source_path.join(&config.history_file).exists());

    Ok(())
}

#[test]
fn test_should_ignore_rigorous() -> anyhow::Result<()> {
    let tmp_dir = TempDir::new()?;
    let source_path = tmp_dir.path().to_path_buf();

    // Setup config
    let mut config = test_config(&source_path);
    config.rules = vec![test_rule("Test", vec!["jpg"], vec![])];
    config.watch_mode = true;
    config.debounce_seconds = 1;
    config.temp_extensions = vec!["tmp".to_string()];
    config.ignore_patterns = vec!["node_modules".to_string()];
    config.trash_dir = PathBuf::from("trash");
    config.unknown_dir = PathBuf::from("unknown");
    config.history_file = ".history".to_string();
    config.exclude_hidden = true;

    // Create the files so canonicalization works reliably in the test
    let trash_file = source_path.join("trash/file.txt");
    let ds_store = source_path.join(".DS_Store");
    let stray_file = source_path.join("trash/stray.txt");
    let valid_photo = source_path.join("valid_photo.jpg");
    let valid_pdf = source_path.join("new_file.pdf");

    fs::create_dir_all(source_path.join("trash"))?;
    fs::write(&trash_file, "")?;
    fs::write(&ds_store, "")?;
    fs::write(&stray_file, "")?;
    fs::write(&valid_photo, "")?;
    fs::write(&valid_pdf, "")?;

    // Create a mock event with MIXED paths
    // Case 1: All paths are noise -> should return true (ignore)
    let event_ignore =
        notify::Event::new(notify::EventKind::Modify(notify::event::ModifyKind::Any))
            .add_path(trash_file)
            .add_path(ds_store);

    assert!(panos::should_ignore(&event_ignore, &config));

    // Case 2: Mixed paths (one noise, one valid) -> should return false (DO NOT ignore)
    let event_mixed =
        notify::Event::new(notify::EventKind::Create(notify::event::CreateKind::File))
            .add_path(stray_file) // Noise
            .add_path(valid_photo); // Valid!

    // This is the bug we fixed! It should be false now because there's a valid path.
    assert!(!panos::should_ignore(&event_mixed, &config));

    // Case 3: All valid -> should return false
    let event_valid =
        notify::Event::new(notify::EventKind::Create(notify::event::CreateKind::File))
            .add_path(valid_pdf);

    assert!(!panos::should_ignore(&event_valid, &config));

    Ok(())
}

#[test]
fn test_watcher_stress_simulation() -> anyhow::Result<()> {
    let tmp_dir = TempDir::new()?;
    let source_path = tmp_dir.path().to_path_buf();

    // Create many subfolders
    for i in 0..10 {
        fs::create_dir_all(source_path.join(format!("folder_{}", i)))?;
    }

    let mut config = test_config(&source_path);
    config.rules = vec![
        {
            let mut r = test_rule("Photos", vec!["jpg"], vec![]);
            r.destination = PathBuf::from("images");
            r
        },
        {
            let mut r = test_rule("Docs", vec!["pdf"], vec![]);
            r.destination = PathBuf::from("docs");
            r
        },
    ];
    config.debounce_seconds = 1;
    config.temp_extensions = vec!["tmp".to_string()];
    config.trash_dir = PathBuf::from("trash");
    config.unknown_dir = PathBuf::from("unknown");
    config.history_file = ".history.json".to_string();
    config.exclude_hidden = false;

    // STRESS: Create 100 files across different folders
    for i in 0..100 {
        let folder = format!("folder_{}", i % 10);
        let ext = if i % 2 == 0 { "jpg" } else { "pdf" };
        let filename = format!("file_{}.{}", i, ext);
        let content = format!("unique content for file {}", i);
        fs::write(source_path.join(folder).join(filename), content)?;
    }

    // Run organization (as watch mode would trigger)
    let mut ai = None;
    let history = organize(&config, false, &NoopReporter, &mut ai)?;

    // Should have 100 moves
    assert_eq!(history.len(), 100);

    // Verify a random sample
    assert!(source_path.join("images/file_0.jpg").exists());
    assert!(source_path.join("docs/file_1.pdf").exists());
    assert!(source_path.join("images/file_98.jpg").exists());
    assert!(source_path.join("docs/file_99.pdf").exists());

    // Verify subfolders are empty (except the destination folders which are inside source)
    remove_empty_dirs(&source_path, false, &history, &NoopReporter)?;
    for i in 0..10 {
        assert!(!source_path.join(format!("folder_{}", i)).exists());
    }

    Ok(())
}

#[test]
fn test_dry_run_empty_dir_prediction() -> anyhow::Result<()> {
    let tmp_dir = TempDir::new()?;
    let source_path = tmp_dir.path().to_path_buf();

    // Setup nested structure that would be empty
    let nested = source_path.join("a/b/c");
    fs::create_dir_all(&nested)?;
    let file_path = nested.join("test.jpg");
    fs::write(&file_path, "content")?;

    let mut config = test_config(&source_path);
    config.rules = vec![test_rule("Images", vec!["jpg"], vec![])];
    config.debounce_seconds = 1;
    config.trash_dir = PathBuf::from("trash");
    config.unknown_dir = PathBuf::from("unknown");
    config.history_file = ".history.json".to_string();
    config.exclude_hidden = false;

    // Run in dry run
    let mut ai = None;
    let history = organize(&config, true, &NoopReporter, &mut ai)?;
    assert_eq!(history.len(), 1);

    // This should now detect that a/b/c, a/b, and a would be empty
    remove_empty_dirs(&source_path, true, &history, &NoopReporter)?;

    // Verify directories still exist (since it's dry run)
    assert!(source_path.join("a/b/c").exists());
    assert!(file_path.exists());

    Ok(())
}
