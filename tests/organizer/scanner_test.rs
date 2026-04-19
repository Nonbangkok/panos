use panos::organizer::scanner::organize;
use panos::ui::NoopReporter;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use crate::common::{test_config, test_rule};

#[test]
fn test_scanner_empty_dir() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let config = test_config(tmp.path());

    let history = organize(&config, true, &NoopReporter)?;
    assert_eq!(history.len(), 0);
    Ok(())
}

#[test]
fn test_scanner_deep_nesting() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let deep_dir = root.join("level1/level2/level3");
    fs::create_dir_all(&deep_dir)?;
    fs::write(deep_dir.join("deep.jpg"), "")?;

    let mut config = test_config(root);
    config.rules = vec![test_rule("Images", vec!["jpg"], vec![])];

    let history = organize(&config, true, &NoopReporter)?;
    assert_eq!(history.len(), 1);
    assert!(history[0].source.ends_with("deep.jpg"));
    Ok(())
}

#[test]
fn test_scanner_ignore_patterns_exact() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    fs::write(root.join("ignore_me.txt"), "")?;
    fs::write(root.join("keep_me.txt"), "")?;

    let mut config = test_config(root);
    config.ignore_patterns = vec!["ignore_me.txt".to_string()];
    config.rules = vec![test_rule("Docs", vec!["txt"], vec![])];

    let history = organize(&config, true, &NoopReporter)?;
    assert_eq!(history.len(), 1);
    assert!(history[0].source.ends_with("keep_me.txt"));
    Ok(())
}

#[test]
fn test_scanner_internal_dirs_protection() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();

    // Internal dirs
    let trash = root.join(".trash");
    fs::create_dir(&trash)?;
    fs::write(trash.join("deleted.jpg"), "")?;

    let unknown = root.join(".unknown");
    fs::create_dir(&unknown)?;
    fs::write(unknown.join("mystery.jpg"), "")?;

    let mut config = test_config(root);
    config.rules = vec![test_rule("Images", vec!["jpg"], vec![])];

    let history = organize(&config, true, &NoopReporter)?;
    assert_eq!(history.len(), 0);
    Ok(())
}

#[test]
fn test_scanner_destination_dirs_protection() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();

    let dest = root.join("Images");
    fs::create_dir(&dest)?;
    fs::write(dest.join("already_there.jpg"), "")?;
    fs::write(root.join("new.jpg"), "")?;

    let mut config = test_config(root);
    config.rules = vec![test_rule("Images", vec!["jpg"], vec![])];

    let history = organize(&config, true, &NoopReporter)?;
    assert_eq!(history.len(), 1);
    assert!(history[0].source.ends_with("new.jpg"));
    Ok(())
}

#[test]
fn test_scanner_recursive_hidden_exclusion() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();

    let hidden_dir = root.join(".secret_dir");
    fs::create_dir(&hidden_dir)?;
    fs::write(hidden_dir.join("secret.jpg"), "")?;
    fs::write(root.join("public.jpg"), "")?;

    let mut config = test_config(root);
    config.exclude_hidden = true;
    config.rules = vec![test_rule("Images", vec!["jpg"], vec![])];

    let history = organize(&config, true, &NoopReporter)?;
    assert_eq!(history.len(), 1);
    assert!(history[0].source.ends_with("public.jpg"));
    Ok(())
}

#[test]
fn test_scanner_filtering_logic() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();

    fs::write(root.join("valid_photo.jpg"), "")?;
    fs::write(root.join(".hidden_file.txt"), "")?;

    let img_dir = root.join("Images");
    fs::create_dir(&img_dir)?;
    fs::write(img_dir.join("existing_photo.jpg"), "")?;

    let mut config = test_config(root);
    config.exclude_hidden = true;
    config.rules = vec![test_rule("Images", vec!["jpg"], vec![])];

    let history = organize(&config, true, &NoopReporter)?;
    assert_eq!(history.len(), 1);
    assert!(history[0].source.ends_with("valid_photo.jpg"));

    Ok(())
}

#[test]
fn test_scanner_mixed_rules_priority() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    fs::write(root.join("test.pdf"), "")?;

    let mut config = test_config(root);

    config.rules = vec![
        test_rule("Docs", vec!["pdf"], vec![]),
        test_rule("Archives", vec!["pdf"], vec![]),
    ];

    let history = organize(&config, true, &NoopReporter)?;
    assert_eq!(history.len(), 1);
    // Check the destination folder name (parent of the destination file)
    let dest_folder = history[0]
        .destination
        .parent()
        .unwrap()
        .file_name()
        .unwrap();
    assert_eq!(dest_folder, "Docs");
    Ok(())
}

#[test]
fn test_scanner_case_insensitivity() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    fs::write(root.join("IMAGE.JPG"), "")?;

    let mut config = test_config(root);
    config.rules = vec![test_rule("Images", vec!["jpg"], vec![])];

    let history = organize(&config, true, &NoopReporter)?;
    assert_eq!(history.len(), 1);
    assert!(history[0].source.ends_with("IMAGE.JPG"));
    Ok(())
}

#[test]
fn test_scanner_glob_patterns() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    fs::write(root.join("invoice_2024.pdf"), "")?;
    fs::write(root.join("receipt_2024.pdf"), "")?;

    let mut config = test_config(root);
    config.rules = vec![test_rule("Invoices", vec![], vec!["invoice_*.pdf"])];
    config.unknown_dir = PathBuf::from(".unknown");

    let history = organize(&config, true, &NoopReporter)?;
    assert_eq!(history.len(), 2); // 1 match + 1 unknown

    // Find the move record for the invoice
    let invoice_record = history
        .iter()
        .find(|r| r.source.ends_with("invoice_2024.pdf"))
        .expect("Invoice should be in history");

    assert!(
        invoice_record
            .destination
            .to_str()
            .unwrap()
            .contains("Invoices")
    );
    Ok(())
}

#[test]
fn test_scanner_no_extension_files() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    fs::write(root.join("README"), "")?;

    let mut config = test_config(root);
    config.rules = vec![test_rule("Docs", vec!["txt"], vec![])];
    config.unknown_dir = PathBuf::from("Other");

    let history = organize(&config, true, &NoopReporter)?;
    assert_eq!(history.len(), 1);
    assert!(history[0].destination.to_str().unwrap().contains("Other"));
    Ok(())
}

#[test]
fn test_scanner_large_extension_list() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let extensions = vec![
        "jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp", "heic", "avif",
    ];
    for ext in &extensions {
        fs::write(root.join(format!("test.{}", ext)), "")?;
    }

    let mut config = test_config(root);
    config.rules = vec![test_rule("Images", extensions, vec![])];

    let history = organize(&config, true, &NoopReporter)?;
    assert_eq!(history.len(), 9);
    Ok(())
}

#[test]
fn test_scanner_overlapping_rules_glob_vs_ext() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    fs::write(root.join("special_report.pdf"), "")?;

    let mut config = test_config(root);
    // Glob should take priority if listed first
    config.rules = vec![
        test_rule("Special", vec![], vec!["special_*.pdf"]),
        test_rule("Docs", vec!["pdf"], vec![]),
    ];

    let history = organize(&config, true, &NoopReporter)?;
    assert_eq!(history.len(), 1);
    let dest_folder = history[0]
        .destination
        .parent()
        .unwrap()
        .file_name()
        .unwrap();
    assert_eq!(dest_folder, "Special");
    Ok(())
}

#[test]
fn test_scanner_rule_with_dots_in_extension() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    fs::write(root.join("archive.tar.gz"), "")?;

    let mut config = test_config(root);
    config.rules = vec![test_rule("Archives", vec!["tar.gz"], vec![])];

    let history = organize(&config, true, &NoopReporter)?;
    assert_eq!(history.len(), 1);
    assert!(history[0].source.ends_with("archive.tar.gz"));
    Ok(())
}

#[test]
fn test_scanner_temp_file_cleanup() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    fs::write(root.join("data.tmp"), "")?;
    fs::write(root.join("data.part"), "")?;

    let mut config = test_config(root);
    config.temp_extensions = vec!["tmp".to_string(), "part".to_string()];
    config.trash_dir = PathBuf::from("TrashBin");

    let history = organize(&config, true, &NoopReporter)?;
    assert_eq!(history.len(), 2);
    for h in history {
        assert!(h.destination.to_str().unwrap().contains("TrashBin"));
    }
    Ok(())
}

#[test]
fn test_scanner_unicode_and_special_chars() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let names = vec!["รูปภาพ.jpg", "space file.png", "special!@#$%^&()_+.txt"];
    for name in &names {
        fs::write(root.join(name), "")?;
    }

    let mut config = test_config(root);
    config.rules = vec![
        test_rule("Images", vec!["jpg", "png"], vec![]),
        test_rule("Docs", vec!["txt"], vec![]),
    ];

    let history = organize(&config, true, &NoopReporter)?;
    assert_eq!(history.len(), 3);
    Ok(())
}

#[test]
fn test_scanner_massive_file_count_batch() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    // Simulate 200 files (sufficient for batching but fast)
    for i in 0..200 {
        fs::write(root.join(format!("file_{}.jpg", i)), "")?;
    }

    let mut config = test_config(root);
    config.rules = vec![test_rule("Images", vec!["jpg"], vec![])];

    let history = organize(&config, true, &NoopReporter)?;
    assert_eq!(history.len(), 200);
    Ok(())
}

#[test]
fn test_scanner_dry_run_state_consistency() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let file_path = root.join("test.jpg");
    fs::write(&file_path, "original")?;

    let mut config = test_config(root);
    config.rules = vec![test_rule("Images", vec!["jpg"], vec![])];

    // Dry run
    let history = organize(&config, true, &NoopReporter)?;
    assert_eq!(history.len(), 1);

    // Verify file still exists in original location
    assert!(file_path.exists());
    assert_eq!(fs::read_to_string(file_path)?, "original");

    // Verify destination dir was NOT created
    assert!(!root.join("Images").exists());
    Ok(())
}

#[test]
fn test_scanner_duplicate_filenames_different_folders() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();
    let dir_a = root.join("folder_a");
    let dir_b = root.join("folder_b");
    fs::create_dir(&dir_a)?;
    fs::create_dir(&dir_b)?;
    fs::write(dir_a.join("same_name.jpg"), "")?;
    fs::write(dir_b.join("same_name.jpg"), "")?;

    let mut config = test_config(root);
    config.rules = vec![test_rule("Images", vec!["jpg"], vec![])];

    let history = organize(&config, true, &NoopReporter)?;
    assert_eq!(history.len(), 2);
    Ok(())
}

#[test]
fn test_scanner_extremely_long_path() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let root = tmp.path();

    // Create a very long subfolder chain
    let mut current = root.to_path_buf();
    for i in 0..20 {
        current = current.join(format!("long_folder_name_{}", i));
    }
    fs::create_dir_all(&current)?;
    fs::write(current.join("deep_file.jpg"), "")?;

    let mut config = test_config(root);
    config.rules = vec![test_rule("Images", vec!["jpg"], vec![])];

    let history = organize(&config, true, &NoopReporter)?;
    assert_eq!(history.len(), 1);
    assert!(history[0].source.ends_with("deep_file.jpg"));
    Ok(())
}
