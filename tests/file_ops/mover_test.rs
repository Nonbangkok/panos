use panos::file_ops::mover::move_file;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_mover_basic_move_success() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src = tmp.path().join("source.txt");
    let dest = tmp.path().join("target.txt");
    fs::write(&src, "content")?;

    let record = move_file(&src, &dest, false)?;
    assert!(
        record.is_some(),
        "Successful move should return a MoveRecord"
    );
    assert!(dest.exists(), "Destination file must exist after moving");
    assert!(!src.exists(), "Source file must not exist after moving");
    Ok(())
}

#[test]
fn test_mover_dry_run_no_actual_move() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src = tmp.path().join("source.txt");
    let dest = tmp.path().join("target.txt");
    fs::write(&src, "content")?;

    let record = move_file(&src, &dest, true)?;
    assert!(record.is_some(), "Dry run should still return a MoveRecord");
    assert!(
        src.exists(),
        "Source file must still exist after dry run move"
    );
    assert!(
        !dest.exists(),
        "Destination file must not be created during dry run"
    );
    Ok(())
}

#[test]
fn test_mover_same_source_and_destination() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src = tmp.path().join("same.txt");
    fs::write(&src, "content")?;

    let record = move_file(&src, &src, false)?;
    assert!(
        record.is_none(),
        "Moving to the same path should return None"
    );
    Ok(())
}

#[test]
fn test_mover_conflict_resolution_level_1() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src_dir = tmp.path().join("src");
    let dst_dir = tmp.path().join("dst");
    fs::create_dir(&src_dir)?;
    fs::create_dir(&dst_dir)?;

    let src = src_dir.join("photo.jpg");
    let dest = dst_dir.join("photo.jpg");
    fs::write(&src, "new image")?;
    fs::write(&dest, "old image")?;

    let record = move_file(&src, &dest, false)?;
    let final_dest = record.unwrap().destination;
    assert_eq!(
        final_dest.file_name().unwrap(),
        "photo_1.jpg",
        "Should resolve conflict by appending _1 based on source name"
    );
    assert!(
        final_dest.exists(),
        "Resolved destination file should exist"
    );
    assert!(
        dest.exists(),
        "Original destination file should remain preserved"
    );
    Ok(())
}

#[test]
fn test_mover_conflict_resolution_level_many() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src_dir = tmp.path().join("src");
    let dst_dir = tmp.path().join("dst");
    fs::create_dir(&src_dir)?;
    fs::create_dir(&dst_dir)?;

    let src = src_dir.join("doc.pdf");
    fs::write(&src, "new doc")?;

    let base_dest = dst_dir.join("doc.pdf");
    fs::write(&base_dest, "v0")?;
    for i in 1..11 {
        fs::write(dst_dir.join(format!("doc_{}.pdf", i)), format!("v{}", i))?;
    }

    let record = move_file(&src, &base_dest, false)?;
    let final_dest = record.unwrap().destination;
    assert_eq!(
        final_dest.file_name().unwrap(),
        "doc_11.pdf",
        "Should resolve conflict even with many existing files"
    );
    Ok(())
}

#[test]
fn test_mover_create_nested_destination_dir() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src = tmp.path().join("file.txt");
    fs::write(&src, "data")?;
    let dest = tmp.path().join("deeply/nested/dir/file.txt");

    move_file(&src, &dest, false)?;
    assert!(
        dest.exists(),
        "Mover should create intermediate directories automatically"
    );
    Ok(())
}

#[test]
fn test_mover_file_with_no_extension() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src_dir = tmp.path().join("src");
    let dst_dir = tmp.path().join("dst");
    fs::create_dir(&src_dir)?;
    fs::create_dir(&dst_dir)?;

    let src = src_dir.join("README");
    let dest = dst_dir.join("README");
    fs::write(&src, "src")?;
    fs::write(&dest, "dest")?;

    let record = move_file(&src, &dest, false)?;
    assert_eq!(
        record.unwrap().destination.file_name().unwrap(),
        "README_1",
        "Should handle extensionless files correctly during conflict"
    );
    Ok(())
}

#[test]
fn test_mover_file_with_dots_in_name() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src_dir = tmp.path().join("src");
    let dst_dir = tmp.path().join("dst");
    fs::create_dir(&src_dir)?;
    fs::create_dir(&dst_dir)?;

    let src = src_dir.join("archive.tar.gz");
    let dest = dst_dir.join("archive.tar.gz");
    fs::write(&src, "new")?;
    fs::write(&dest, "old")?;

    let record = move_file(&src, &dest, false)?;
    assert_eq!(
        record.unwrap().destination.file_name().unwrap(),
        "archive.tar_1.gz",
        "Should handle multiple dots correctly"
    );
    Ok(())
}

#[test]
fn test_mover_unicode_and_emojis() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let name = "เอกสาร_🚀.txt";
    let src = tmp.path().join(name);
    let dest = tmp.path().join("output").join(name);
    fs::write(&src, "unicode content")?;

    move_file(&src, &dest, false)?;
    assert!(
        dest.exists(),
        "Mover must support Unicode and Emoji filenames"
    );
    Ok(())
}

#[test]
fn test_mover_hidden_file() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src = tmp.path().join(".env");
    let dest = tmp.path().join("secret/.env");
    fs::write(&src, "KEY=VALUE")?;

    move_file(&src, &dest, false)?;
    assert!(
        dest.exists(),
        "Moving hidden files should be handled correctly"
    );
    Ok(())
}

#[test]
fn test_mover_empty_file_move() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src = tmp.path().join("empty.txt");
    let dest = tmp.path().join("empty_dest.txt");
    fs::File::create(&src)?;

    move_file(&src, &dest, false)?;
    assert!(dest.exists(), "Should allow moving empty (0-byte) files");
    assert_eq!(
        fs::metadata(&dest)?.len(),
        0,
        "File size must remain 0 after move"
    );
    Ok(())
}

#[test]
fn test_mover_special_characters_in_filename() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let name = "!@#$%^&()_+ {}.txt";
    let src = tmp.path().join(name);
    let dest = tmp.path().join("special").join(name);
    fs::write(&src, "safe")?;

    move_file(&src, &dest, false)?;
    assert!(
        dest.exists(),
        "Standard special characters in filenames should be supported"
    );
    Ok(())
}

#[test]
fn test_mover_long_path_move() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let mut path = tmp.path().to_path_buf();
    for _ in 0..10 {
        path.push("very_long_directory_name_to_test_path_limits");
    }
    let src = tmp.path().join("short.txt");
    let dest = path.join("file.txt");
    fs::write(&src, "content")?;

    move_file(&src, &dest, false)?;
    assert!(dest.exists(), "Should handle deeply nested long path names");
    Ok(())
}

#[test]
fn test_mover_record_metadata_accuracy() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src = tmp.path().join("meta.txt");
    let dest = tmp.path().join("meta_dest.txt");
    fs::write(&src, "data")?;

    let record = move_file(&src, &dest, false)?.unwrap();
    assert_eq!(
        record.source, src,
        "MoveRecord source should match original input"
    );
    assert_eq!(
        record.destination, dest,
        "MoveRecord destination should match final path"
    );
    let now = chrono::Utc::now();
    assert!(
        record.timestamp <= now,
        "Timestamp should be captured accurately"
    );
    Ok(())
}

#[test]
fn test_mover_non_existent_source_fails() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src = tmp.path().join("ghost.txt");
    let dest = tmp.path().join("grave.txt");

    let result = move_file(&src, &dest, false);
    assert!(
        result.is_err(),
        "Moving a non-existent file must result in an error"
    );
    Ok(())
}

#[test]
fn test_mover_overwrite_not_allowed_without_rename() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src_dir = tmp.path().join("src");
    let dst_dir = tmp.path().join("dst");
    fs::create_dir(&src_dir)?;
    fs::create_dir(&dst_dir)?;

    let src = src_dir.join("keep.txt");
    let dest = dst_dir.join("keep.txt");
    fs::write(&src, "new")?;
    fs::write(&dest, "old")?;

    move_file(&src, &dest, false)?;
    assert!(dest.exists(), "Original target must not be deleted");
    assert_eq!(
        fs::read_to_string(&dest)?,
        "old",
        "Content of original target must be preserved"
    );
    Ok(())
}

#[test]
fn test_mover_collision_with_dot_only_name() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src_dir = tmp.path().join("src");
    let dst_dir = tmp.path().join("dst");
    fs::create_dir(&src_dir)?;
    fs::create_dir(&dst_dir)?;

    let src = src_dir.join(".config");
    let dest = dst_dir.join(".config");
    fs::write(&src, "new")?;
    fs::write(&dest, "old")?;

    let record = move_file(&src, &dest, false)?;
    assert_eq!(
        record.unwrap().destination.file_name().unwrap(),
        ".config_1",
        "Should handle files starting with dot correctly"
    );
    Ok(())
}

#[test]
fn test_mover_spaces_in_path_handling() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src_dir = tmp.path().join("my source dir");
    let dest_dir = tmp.path().join("my destination dir");
    fs::create_dir(&src_dir)?;
    let src = src_dir.join("my file.txt");
    let dest = dest_dir.join("my file.txt");
    fs::write(&src, "txt")?;

    move_file(&src, &dest, false)?;
    assert!(
        dest.exists(),
        "Should correctly handle paths containing spaces"
    );
    Ok(())
}

#[test]
fn test_mover_heavy_sequential_load() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src_dir = tmp.path().join("src");
    let dest_dir = tmp.path().join("dst");
    fs::create_dir(&src_dir)?;

    for i in 0..100 {
        let f = src_dir.join(format!("{}.txt", i));
        fs::write(&f, "data")?;
        move_file(&f, &dest_dir.join(format!("{}.txt", i)), false)?;
    }

    assert_eq!(
        fs::read_dir(dest_dir)?.count(),
        100,
        "Should successfully handle 100 sequential moves"
    );
    Ok(())
}

#[test]
fn test_mover_dots_as_directory_name() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src = tmp.path().join("file.txt");
    let dest = tmp.path().join("...").join("file.txt");
    fs::write(&src, "data")?;

    move_file(&src, &dest, false)?;
    assert!(
        dest.exists(),
        "Should handle directory names consisting only of dots"
    );
    Ok(())
}

#[test]
fn test_mover_source_is_directory_success() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src = tmp.path().join("dir_to_move");
    fs::create_dir(&src)?;
    fs::write(src.join("inside.txt"), "data")?;
    let dest = tmp.path().join("target_dir");

    move_file(&src, &dest, false)?;
    assert!(
        dest.is_dir(),
        "Renaming a directory should work as it uses fs::rename internally"
    );
    assert!(
        dest.join("inside.txt").exists(),
        "Directory content should be moved together"
    );
    Ok(())
}

#[test]
fn test_mover_extremely_high_conflict_count() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src_dir = tmp.path().join("src");
    let dst_dir = tmp.path().join("dst");
    fs::create_dir(&src_dir)?;
    fs::create_dir(&dst_dir)?;

    let base = dst_dir.join("clash.txt");
    fs::write(&base, "origin")?;
    for i in 1..25 {
        fs::write(dst_dir.join(format!("clash_{}.txt", i)), "clash")?;
    }

    let src = src_dir.join("clash.txt");
    fs::write(&src, "new")?;

    let record = move_file(&src, &base, false)?;
    assert_eq!(
        record.unwrap().destination.file_name().unwrap(),
        "clash_25.txt",
        "Should correctly increment until 25"
    );
    Ok(())
}

#[test]
fn test_mover_extension_case_preservation() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src_dir = tmp.path().join("src");
    let dst_dir = tmp.path().join("dst");
    fs::create_dir(&src_dir)?;
    fs::create_dir(&dst_dir)?;

    let src = src_dir.join("image.JPG");
    let dest = dst_dir.join("image.JPG");
    fs::write(&src, "new")?;
    fs::write(&dest, "old")?;

    let record = move_file(&src, &dest, false)?;
    assert_eq!(
        record.unwrap().destination.file_name().unwrap(),
        "image_1.JPG",
        "Should preserve the case of filename extensions"
    );
    Ok(())
}

#[test]
fn test_mover_filename_with_leading_spaces() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src = tmp.path().join("  padded.txt");
    let dest = tmp.path().join("target").join("  padded.txt");
    fs::write(&src, "content")?;

    move_file(&src, &dest, false)?;
    assert!(
        dest.exists(),
        "Should handle filenames with leading spaces correctly"
    );
    Ok(())
}

#[test]
fn test_mover_large_filename_stress() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let long_name = "n".repeat(200) + ".txt";
    let src = tmp.path().join(&long_name);
    let dest = tmp.path().join("out").join(&long_name);
    fs::write(&src, "data")?;

    move_file(&src, &dest, false)?;
    assert!(
        dest.exists(),
        "Should handle moderately long filenames correctly"
    );
    Ok(())
}

#[test]
fn test_mover_rename_failure_fallback_copy() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src = tmp.path().join("src.txt");
    let dest = tmp.path().join("dest.txt");
    fs::write(&src, "important data")?;

    let record = move_file(&src, &dest, false)?;
    assert!(
        record.is_some(),
        "Move should succeed even if rename logic is tested on same filesystem"
    );
    assert_eq!(
        fs::read_to_string(dest)?,
        "important data",
        "File content must be identical after move"
    );
    Ok(())
}

#[test]
fn test_mover_multiple_consecutive_dots() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src = tmp.path().join("odd..name...txt");
    let dest = tmp.path().join("target..txt");
    fs::write(&src, "weird")?;

    move_file(&src, &dest, false)?;
    assert!(
        dest.exists(),
        "Should handle filenames with multiple consecutive dots"
    );
    Ok(())
}

#[test]
fn test_mover_very_deep_directory_nesting_creation() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src = tmp.path().join("file.txt");
    fs::write(&src, "data")?;

    let mut dest = tmp.path().to_path_buf();
    for i in 0..20 {
        dest.push(format!("level_{}", i));
    }
    dest.push("final.txt");

    move_file(&src, &dest, false)?;
    assert!(
        dest.exists(),
        "Should successfully create deeply nested directory structures"
    );
    Ok(())
}

#[test]
fn test_mover_collision_on_extensionless_hidden_file() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src_dir = tmp.path().join("src");
    let dst_dir = tmp.path().join("dst");
    fs::create_dir(&src_dir)?;
    fs::create_dir(&dst_dir)?;

    let src = src_dir.join(".noext");
    let dest = dst_dir.join(".noext");
    fs::write(&src, "a")?;
    fs::write(&dest, "b")?;

    let record = move_file(&src, &dest, false)?;
    assert_eq!(
        record.unwrap().destination.file_name().unwrap(),
        ".noext_1",
        "Should handle extensionless hidden files during collision"
    );
    Ok(())
}

#[test]
fn test_mover_dry_run_with_conflict_predicts_correct_name() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src_dir = tmp.path().join("src");
    let dst_dir = tmp.path().join("dst");
    fs::create_dir(&src_dir)?;
    fs::create_dir(&dst_dir)?;

    let src = src_dir.join("test.txt");
    let dest = dst_dir.join("test.txt");
    fs::write(&src, "new")?;
    fs::write(&dest, "old")?;

    let record = move_file(&src, &dest, true)?;
    assert_eq!(
        record.unwrap().destination.file_name().unwrap(),
        "test_1.txt",
        "Dry run should accurately predict the final resolved filename"
    );
    assert!(
        !dst_dir.join("test_1.txt").exists(),
        "Dry run must not create the predicted file"
    );
    Ok(())
}

#[test]
fn test_mover_vulnerability_trailing_dot_loss() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let src_dir = tmp.path().join("src");
    let dst_dir = tmp.path().join("dst");
    fs::create_dir_all(&src_dir)?;
    fs::create_dir_all(&dst_dir)?;

    let src = src_dir.join("data.txt.");
    let dest = dst_dir.join("data.txt.");
    fs::write(&src, "new")?;
    fs::write(&dest, "old")?;

    let record = move_file(&src, &dest, false)?;
    let final_dest = record.unwrap().destination;

    let final_name = final_dest.file_name().unwrap().to_str().unwrap();
    assert!(
        final_name.contains("._"),
        "VULNERABILITY: Trailing dot lost! Expected something like data.txt._1 but got {:?}",
        final_name
    );
    Ok(())
}
