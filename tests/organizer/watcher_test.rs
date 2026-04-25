use panos::organizer::watcher::watch_mode;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

use crate::common::{test_config, test_rule};

#[test]
fn test_watcher_moves_new_file() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();

    let mut config = test_config(root);
    config.rules = vec![test_rule("Images", vec!["jpg"], vec![])];
    config.debounce_seconds = 1;

    let config_clone = config.clone();
    thread::spawn(move || {
        let mut ai = None;
        let _ = watch_mode(&config_clone, false, &mut ai);
    });

    thread::sleep(Duration::from_millis(500));

    let test_file = root.join("photo.jpg");
    fs::write(&test_file, "image data")?;

    thread::sleep(Duration::from_secs(3));

    let expected_path = root.join("Images/photo.jpg");
    assert!(
        expected_path.exists(),
        "Watcher should have moved the file to Images/"
    );
    assert!(!test_file.exists(), "Original file should be gone");

    Ok(())
}

#[test]
fn test_watcher_batch_processing_and_conflicts() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();

    let mut config = test_config(root);
    config.rules = vec![
        test_rule("Docs", vec!["pdf", "txt"], vec![]),
        test_rule("Images", vec!["png"], vec![]),
    ];
    config.debounce_seconds = 1;

    let docs_dir = root.join("Docs");
    fs::create_dir_all(&docs_dir)?;
    fs::write(docs_dir.join("report.pdf"), "old report")?;
    let config_clone = config.clone();
    thread::spawn(move || {
        let mut ai = None;
        let _ = watch_mode(&config_clone, false, &mut ai);
    });
    thread::sleep(Duration::from_millis(500));

    let subfolder = root.join("new_project");
    fs::create_dir_all(&subfolder)?;

    fs::write(subfolder.join("report.pdf"), "new report content")?;
    fs::write(subfolder.join("note.txt"), "some notes")?;
    fs::write(root.join("logo.png"), "image data")?;

    thread::sleep(Duration::from_secs(3));

    assert!(root.join("Docs/report.pdf").exists());
    assert!(
        root.join("Docs/report_1.pdf").exists(),
        "Conflict file should be renamed to report_1.pdf"
    );
    assert!(root.join("Docs/note.txt").exists());
    assert!(root.join("Images/logo.png").exists());

    assert!(
        !subfolder.exists(),
        "The empty subfolder should have been cleaned up"
    );

    Ok(())
}

#[test]
fn test_watcher_basic_flow_simple() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let mut config = test_config(root);
    config.rules = vec![test_rule("Docs", vec!["pdf"], vec![])];
    config.debounce_seconds = 1;

    let config_clone = config.clone();
    thread::spawn(move || {
        let mut ai = None;
        let _ = watch_mode(&config_clone, false, &mut ai);
    });
    thread::sleep(Duration::from_millis(500));

    let f = root.join("test.pdf");
    fs::write(&f, "pdf data")?;
    thread::sleep(Duration::from_secs(3));

    assert!(
        root.join("Docs/test.pdf").exists(),
        "PDF file should be moved to Docs folder"
    );
    assert!(
        !f.exists(),
        "Original file should be deleted after successful move"
    );
    Ok(())
}

#[test]
fn test_watcher_batch_and_conflict_resolution() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let mut config = test_config(root);
    config.rules = vec![test_rule("Images", vec!["png"], vec![])];
    config.debounce_seconds = 1;

    let dest = root.join("Images");
    fs::create_dir_all(&dest)?;
    fs::write(dest.join("logo.png"), "old logo")?;

    let config_clone = config.clone();
    thread::spawn(move || {
        let mut ai = None;
        let _ = watch_mode(&config_clone, false, &mut ai);
    });
    thread::sleep(Duration::from_millis(500));

    fs::write(root.join("logo.png"), "new logo")?;
    thread::sleep(Duration::from_secs(3));

    assert!(
        root.join("Images/logo.png").exists(),
        "Original logo file should still exist"
    );
    assert!(
        root.join("Images/logo_1.png").exists(),
        "New file with same name should be renamed to logo_1.png to prevent data loss"
    );
    Ok(())
}

#[test]
fn test_watcher_unicode_and_special_names() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let mut config = test_config(root);
    config.rules = vec![test_rule("เอกสาร", vec!["docx"], vec![])];
    config.debounce_seconds = 1;

    let config_clone = config.clone();
    thread::spawn(move || {
        let mut ai = None;
        let _ = watch_mode(&config_clone, false, &mut ai);
    });
    thread::sleep(Duration::from_millis(500));

    let thai_file = root.join("รายงาน_สรุป #2024 🔥.docx");
    fs::write(&thai_file, "ข้อมูุล")?;
    thread::sleep(Duration::from_secs(3));

    assert!(
        root.join("เอกสาร/รายงาน_สรุป #2024 🔥.docx").exists(),
        "Watcher should support Thai and emoji filenames correctly"
    );
    Ok(())
}

#[test]
fn test_watcher_should_not_move_ignored_files() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let mut config = test_config(root);
    config.ignore_patterns = vec!["keep.me".to_string(), "ignore_this.txt".to_string()];
    config.rules = vec![test_rule("Docs", vec!["txt"], vec![])];
    config.debounce_seconds = 1;

    let config_clone = config.clone();
    thread::spawn(move || {
        let mut ai = None;
        let _ = watch_mode(&config_clone, false, &mut ai);
    });
    thread::sleep(Duration::from_millis(500));

    let ignored = root.join("ignore_this.txt");
    fs::write(&ignored, "don't move")?;
    thread::sleep(Duration::from_secs(3));

    assert!(ignored.exists(), "File in ignore list should not be moved");
    assert!(
        !root.join("Docs/ignore_this.txt").exists(),
        "Ignored file should not appear in destination folder"
    );
    Ok(())
}
#[test]
fn test_watcher_massive_burst_100_files() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let mut config = test_config(root);
    config.rules = vec![test_rule("Data", vec!["csv"], vec![])];
    config.debounce_seconds = 1;

    let config_clone = config.clone();
    thread::spawn(move || {
        let mut ai = None;
        let _ = watch_mode(&config_clone, false, &mut ai);
    });
    thread::sleep(Duration::from_millis(500));

    for i in 0..100 {
        fs::write(root.join(format!("file_{}.csv", i)), format!("data {}", i))?;
    }
    thread::sleep(Duration::from_secs(3));

    let count = fs::read_dir(root.join("Data"))?.count();
    assert_eq!(
        count, 100,
        "System should be able to collect and move 100 files in a single scan"
    );
    Ok(())
}

#[test]
fn test_watcher_nested_recursive_organization() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let mut config = test_config(root);
    config.rules = vec![test_rule("Code", vec!["rs"], vec![])];
    config.debounce_seconds = 1;

    let config_clone = config.clone();
    thread::spawn(move || {
        let mut ai = None;
        let _ = watch_mode(&config_clone, false, &mut ai);
    });
    thread::sleep(Duration::from_millis(500));

    let deep_dir = root.join("a/b/c/d/e/f");
    fs::create_dir_all(&deep_dir)?;
    fs::write(deep_dir.join("main.rs"), "fn main() {}")?;
    thread::sleep(Duration::from_secs(3));

    assert!(
        root.join("Code/main.rs").exists(),
        "File in deeply nested folder should be found and moved to destination folder"
    );
    Ok(())
}

#[test]
fn test_watcher_unknown_file_type_handling() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let mut config = test_config(root);
    config.unknown_dir = PathBuf::from("Etc");
    config.debounce_seconds = 1;

    let config_clone = config.clone();
    thread::spawn(move || {
        let mut ai = None;
        let _ = watch_mode(&config_clone, false, &mut ai);
    });
    thread::sleep(Duration::from_millis(500));

    let mystery = root.join("something.weird");
    fs::write(&mystery, "unknown data")?;
    thread::sleep(Duration::from_secs(3));

    assert!(
        root.join("Etc/something.weird").exists(),
        "File that doesn't match any rule should be moved to Unknown (Etc) folder"
    );
    Ok(())
}

#[test]
fn test_watcher_empty_directory_cleanup_chain() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let mut config = test_config(root);
    config.rules = vec![test_rule("Docs", vec!["txt"], vec![])];
    config.debounce_seconds = 1;

    let config_clone = config.clone();
    thread::spawn(move || {
        let mut ai = None;
        let _ = watch_mode(&config_clone, false, &mut ai);
    });
    thread::sleep(Duration::from_millis(500));

    let nested_empty = root.join("level1/level2/level3");
    fs::create_dir_all(&nested_empty)?;
    fs::write(nested_empty.join("task.txt"), "todo")?;
    thread::sleep(Duration::from_secs(3));

    assert!(
        root.join("Docs/task.txt").exists(),
        "File must be moved out"
    );
    assert!(
        !root.join("level1").exists(),
        "All empty source folders must be deleted (Cleanup Chain)"
    );
    Ok(())
}
#[test]
fn test_watcher_massive_filename_length() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let mut config = test_config(root);
    config.rules = vec![test_rule("Overflow", vec!["bin"], vec![])];
    config.debounce_seconds = 1;

    let config_clone = config.clone();
    thread::spawn(move || {
        let mut ai = None;
        let _ = watch_mode(&config_clone, false, &mut ai);
    });
    thread::sleep(Duration::from_millis(500));

    let long_name = "a".repeat(200) + ".bin";
    let big_path = root.join(&long_name);
    fs::write(&big_path, "binary")?;
    thread::sleep(Duration::from_secs(3));

    assert!(
        root.join("Overflow").join(long_name).exists(),
        "File with abnormal length of 200+ characters should be moved without crashing the system"
    );
    Ok(())
}

#[test]
fn test_watcher_exclude_hidden_files() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let mut config = test_config(root);
    config.exclude_hidden = true;
    config.rules = vec![test_rule("Protected", vec!["config"], vec![])];
    config.debounce_seconds = 1;

    let config_clone = config.clone();
    thread::spawn(move || {
        let mut ai = None;
        let _ = watch_mode(&config_clone, false, &mut ai);
    });
    thread::sleep(Duration::from_millis(500));

    let hidden_file = root.join(".my_config.config");
    fs::write(&hidden_file, "secret")?;
    thread::sleep(Duration::from_secs(3));

    assert!(
        hidden_file.exists(),
        "Hidden files starting with dot (.) must not be moved, even if their extension matches the rule"
    );
    assert!(
        !root.join("Protected/.my_config.config").exists(),
        "Hidden files must not appear in the organized folder"
    );
    Ok(())
}
