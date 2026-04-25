#![allow(dead_code)]

use panos::rules::ai::PanosAI;
use panos::{Config, Rule};
use std::path::{Path, PathBuf};

pub fn test_config(root: &Path) -> Config {
    Config {
        source_dir: root.to_path_buf(),
        ..Config::default()
    }
}

pub fn test_rule(name: &str, exts: Vec<&str>, patterns: Vec<&str>) -> Rule {
    let mut rule = Rule {
        name: name.to_string(),
        extensions: exts.into_iter().map(|s| s.to_string()).collect(),
        patterns: patterns.iter().map(|s| s.to_string()).collect(),
        destination: PathBuf::from(name),
        ..Rule::default()
    };
    rule.sanitize();
    rule
}

pub fn test_ai_engine(rules: &[Rule]) -> Option<PanosAI> {
    PanosAI::new("model_assets", rules).ok()
}

pub fn test_ai(filename: &str, label: &str) -> Option<String> {
    let config = test_config(Path::new("."));
    let mut rule = test_rule("Target", vec![], vec![]);
    rule.semantic_label = Some(label.to_string());
    let rules = vec![rule];

    if let Some(mut ai) = test_ai_engine(&rules) {
        return ai
            .determine_rule(filename, &config, &rules)
            .map(|r| r.name.clone());
    }
    None
}
