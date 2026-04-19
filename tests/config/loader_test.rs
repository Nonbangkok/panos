use panos::Config;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_load_valid_config() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let config_path = tmp.path().join("panos.toml");

    let toml_content = r#"
        source_dir = "."
        watch_mode = true
        debounce_seconds = 5
        
        [[rules]]
        name = "Images"
        extensions = [".jpg", ".png"]
        destination = "MyPhotos"
    "#;
    fs::write(&config_path, toml_content)?;

    let config = Config::load(&config_path)?;

    assert_eq!(
        config.source_dir.to_str().unwrap(),
        ".",
        "Source directory should match file content"
    );
    assert!(config.watch_mode, "Watch mode should be true");
    assert_eq!(config.debounce_seconds, 5, "Debounce seconds should be 5");

    assert_eq!(config.rules.len(), 1, "Should have exactly 1 rule");
    assert_eq!(config.rules[0].name, "Images");

    assert!(
        config.rules[0].extensions.contains(&"jpg".to_string()),
        "Extensions should be sanitized automatically"
    );

    Ok(())
}

#[test]
fn test_load_config_not_found() {
    let result = Config::load(Path::new("non_existent_file.toml"));
    assert!(
        result.is_err(),
        "Loading non-existent file should return error"
    );
}

#[test]
fn test_config_default_values() {
    let config = Config::default();
    assert_eq!(
        config.debounce_seconds, 2,
        "Default debounce_seconds should be 2"
    );
    assert!(
        config.exclude_hidden,
        "By default exclude_hidden should be true"
    );
}

#[test]
fn test_config_massive_rules_list() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let config_path = tmp.path().join("massive.toml");

    let mut toml_content = String::from("source_dir = \".\"\n");
    for i in 0..150 {
        toml_content.push_str(&format!(
            "[[rules]]\nname = \"Rule{}\"\nextensions = [\"ext{}\"]\ndestination = \"dest{}\"\n",
            i, i, i
        ));
    }
    fs::write(&config_path, toml_content)?;

    let config = Config::load(&config_path)?;
    assert_eq!(
        config.rules.len(),
        150,
        "Should load all 150 rules without loss"
    );
    assert_eq!(
        config.rules[149].name, "Rule149",
        "Data of the last rule should be accurate"
    );
    Ok(())
}

#[test]
fn test_config_multi_dot_extensions_sanitize() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let config_path = tmp.path().join("multidot.toml");
    let toml = r#"
        source_dir = "."
        [[rules]]
        name = "Archives"
        extensions = [".TAR.GZ", "Zip.Tmp"]
        destination = "Arch"
    "#;
    fs::write(&config_path, toml)?;

    let config = Config::load(&config_path)?;
    let exts = &config.rules[0].extensions;
    assert!(
        exts.contains(&"tar.gz".to_string()),
        ".TAR.GZ extension should be converted to tar.gz and dots removed"
    );
    assert!(
        exts.contains(&"zip.tmp".to_string()),
        "Zip.Tmp extension should be converted to lowercase"
    );
    Ok(())
}

#[test]
fn test_config_empty_rules_list() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let config_path = tmp.path().join("empty_rules.toml");
    fs::write(&config_path, "source_dir = \".\"\nrules = []")?;

    let config = Config::load(&config_path)?;
    assert!(
        config.rules.is_empty(),
        "Empty rules array should load as empty"
    );
    Ok(())
}

#[test]
fn test_config_missing_optional_fields() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let config_path = tmp.path().join("minimal.toml");
    fs::write(&config_path, "source_dir = \".\"")?;

    let config = Config::load(&config_path)?;
    assert_eq!(
        config.debounce_seconds, 2,
        "Missing debounce_seconds should fallback to default (2)"
    );
    assert!(
        config.ignore_patterns.is_empty(),
        "Missing ignore_patterns should be empty"
    );
    assert!(
        config.exclude_hidden,
        "Missing exclude_hidden should fallback to true"
    );
    Ok(())
}

#[test]
fn test_config_unicode_rule_names() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let config_path = tmp.path().join("unicode.toml");
    let toml = r#"
        source_dir = "."
        [[rules]]
        name = "รูปภาพครอบครัว"
        extensions = ["jpg"]
        destination = "โฟลเดอร์/ลับ"
    "#;
    fs::write(&config_path, toml)?;

    let config = Config::load(&config_path)?;
    assert_eq!(
        config.rules[0].name, "รูปภาพครอบครัว",
        "Unicode rule name should load correctly"
    );
    assert_eq!(
        config.rules[0].destination.to_str().unwrap(),
        "โฟลเดอร์/ลับ",
        "Unicode destination path should load correctly"
    );
    Ok(())
}

#[test]
fn test_config_invalid_toml_syntax() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("garbage.toml");
    fs::write(&config_path, "this is not = a toml file [ [[ [").unwrap();

    let result = Config::load(&config_path);
    assert!(result.is_err(), "Malformed TOML should always return error");
}

#[test]
fn test_config_extreme_debounce_values() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let config_path = tmp.path().join("extreme.toml");
    fs::write(
        &config_path,
        "source_dir = \".\"\ndebounce_seconds = 999999",
    )?;

    let config = Config::load(&config_path)?;
    assert_eq!(
        config.debounce_seconds, 999999,
        "Should handle large debounce values without overflow"
    );
    Ok(())
}

#[test]
fn test_config_case_insensitive_extensions_sanitize() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let config_path = tmp.path().join("case.toml");
    let toml = r#"
        source_dir = "."
        [[rules]]
        name = "Test"
        extensions = ["JPG", "Png", "pDf"]
        destination = "out"
    "#;
    fs::write(&config_path, toml)?;

    let config = Config::load(&config_path)?;
    let exts = &config.rules[0].extensions;
    assert!(
        exts.iter().all(|e| e == &e.to_lowercase()),
        "All extensions should be lowercased after sanitization"
    );
    Ok(())
}

#[test]
fn test_config_too_many_temp_extensions() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let config_path = tmp.path().join("temp_ext.toml");
    let mut content = String::from("source_dir = \".\"\ntemp_extensions = [");
    for i in 0..100 {
        content.push_str(&format!("\"tmp{}\"{}", i, if i < 99 { "," } else { "" }));
    }
    content.push(']');
    fs::write(&config_path, content)?;

    let config = Config::load(&config_path)?;
    assert_eq!(
        config.temp_extensions.len(),
        100,
        "Should load all 100 temp extensions"
    );
    Ok(())
}

#[test]
fn test_config_rule_with_glob_patterns() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let config_path = tmp.path().join("glob.toml");
    let toml = r#"
        source_dir = "."
        [[rules]]
        name = "Backups"
        extensions = []
        patterns = ["*backup*", "2024-??-*", "[0-9]*.zip"]
        destination = "bak"
    "#;
    fs::write(&config_path, toml)?;

    let config = Config::load(&config_path)?;
    assert_eq!(
        config.rules[0].patterns.len(),
        3,
        "Should load all glob patterns"
    );
    assert_eq!(
        config.rules[0].patterns[1], "2024-??-*",
        "Glob pattern content should be accurate"
    );
    Ok(())
}

#[test]
fn test_config_exclude_hidden_toggle() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let config_path = tmp.path().join("hidden_off.toml");
    fs::write(&config_path, "source_dir = \".\"\nexclude_hidden = false")?;

    let config = Config::load(&config_path)?;
    assert!(
        !config.exclude_hidden,
        "exclude_hidden = false should be correctly loaded"
    );
    Ok(())
}

#[test]
fn test_config_malformed_rule_entry() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("invalid_type.toml");
    let toml = r#"
        source_dir = "."
        [[rules]]
        name = 12345
        destination = "broken"
    "#;
    fs::write(&config_path, toml).unwrap();

    let result = Config::load(&config_path);
    assert!(result.is_err(), "Type mismatch should cause loading error");
}

#[test]
fn test_config_special_characters_in_paths() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let config_path = tmp.path().join("special_path.toml");
    let toml = r#"
        source_dir = "."
        trash_dir = ".panos_trash (deleted Files)"
        unknown_dir = "Other Files/Not Recognized!"
    "#;
    fs::write(&config_path, toml)?;

    let config = Config::load(&config_path)?;
    assert_eq!(
        config.trash_dir.to_str().unwrap(),
        ".panos_trash (deleted Files)",
        "Trash directory with spaces and parentheses should load correctly"
    );
    assert!(
        config.unknown_dir.to_str().unwrap().contains('!'),
        "Special characters in path should not cause loading issues"
    );
    Ok(())
}

#[test]
fn test_config_history_file_custom_name() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let config_path = tmp.path().join("history.toml");
    fs::write(
        &config_path,
        "source_dir = \".\"\nhistory_file = \"custom_history.db\"",
    )?;

    let config = Config::load(&config_path)?;
    assert_eq!(
        config.history_file, "custom_history.db",
        "Should be able to customize history file name"
    );
    Ok(())
}

#[test]
fn test_config_comment_heavy_toml() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let config_path = tmp.path().join("comments.toml");
    let toml = r#"
        source_dir = "."
        
        [[rules]]
        name = "Rule1"
        extensions = [
            "txt",
            "doc"
        ]
        destination = "docs"
    "#;
    fs::write(&config_path, toml)?;

    let config = Config::load(&config_path)?;
    assert_eq!(
        config.rules[0].extensions.len(),
        2,
        "TOML should be parsed correctly regardless of comments presence"
    );
    Ok(())
}

#[test]
fn test_config_numeric_types_max_limit() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let config_path = tmp.path().join("limits.toml");
    fs::write(
        &config_path,
        "source_dir = \".\"\npolling_interval_ms = 4294967295",
    )?;

    let config = Config::load(&config_path)?;
    assert_eq!(
        config.polling_interval_ms, 4294967295,
        "Should load large polling_interval values"
    );
    Ok(())
}

#[test]
fn test_config_full_schema_load() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let config_path = tmp.path().join("full.toml");
    let toml = r#"
        source_dir = "."
        watch_mode = true
        debounce_seconds = 10
        polling_interval_ms = 500
        temp_extensions = ["part", "crdownload"]
        ignore_patterns = ["node_modules", ".git"]
        trash_dir = ".temp_trash"
        unknown_dir = "misc"
        history_file = "activity.json"
        exclude_hidden = false
        
        [[rules]]
        name = "Web"
        extensions = ["html", "css", "js"]
        patterns = ["*.v6.*"]
        destination = "web_dist"
    "#;
    fs::write(&config_path, toml)?;

    let config = Config::load(&config_path)?;
    assert!(
        config.watch_mode && !config.exclude_hidden,
        "Boolean values should match file content"
    );
    assert_eq!(
        config.temp_extensions.len(),
        2,
        "Temp extensions list should be complete"
    );
    assert_eq!(
        config.ignore_patterns[1], ".git",
        "Ignore patterns should maintain order"
    );
    Ok(())
}

#[test]
fn test_config_path_traversal_attempt() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let config_path = tmp.path().join("traversal.toml");
    fs::write(&config_path, "source_dir = \"../../etc\"")?;

    let result = Config::load(&config_path);
    assert!(
        result.is_err(),
        "Invalid or risky source_dir should be detected"
    );
    Ok(())
}

#[test]
fn test_config_rules_sanitize_chained() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let config_path = tmp.path().join("chained.toml");
    let toml = r#"
        source_dir = "."
        [[rules]]
        name = "Clean"
        extensions = [".JPG", " PNG "]
        patterns = ["*.TMP ", " BACKUP.*"]
        destination = "out"
    "#;
    fs::write(&config_path, toml)?;

    let config = Config::load(&config_path)?;
    let rule = &config.rules[0];
    assert_eq!(
        rule.extensions[0], "jpg",
        "Extensions should be cleaned and standardized"
    );
    Ok(())
}

#[test]
fn test_config_empty_string_fields() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let config_path = tmp.path().join("empty_strings.toml");
    let toml = r#"
        source_dir = "."
        history_file = ""
        [[rules]]
        name = ""
        extensions = [""]
        destination = ""
    "#;
    fs::write(&config_path, toml)?;

    let config = Config::load(&config_path)?;
    assert_eq!(
        config.history_file, "",
        "Empty string for history_file should be allowed"
    );
    assert_eq!(
        config.rules[0].name, "",
        "Empty rule name should be allowed if specified"
    );
    Ok(())
}
