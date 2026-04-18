//! File moving and handling operations

use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

/// Move a file to destination with conflict resolution
pub fn move_file(
    source: &std::path::Path,
    dest_dir: &std::path::Path,
    dry_run: bool,
) -> Result<()> {
    let source_parent = source.parent().ok_or_else(|| anyhow::anyhow!("Could not get parent directory"))?;
    if source_parent == dest_dir {
        // File is already in the target directory, nothing to do
        return Ok(());
    }

    let file_name: &std::ffi::OsStr = source
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Could not get file name"))?;
    let mut dest_path: PathBuf = dest_dir.join(file_name);

    if !dry_run {
        std::fs::create_dir_all(dest_dir)?;
    }

    // Handle conflict
    if dest_path.exists() {
        let stem: &str = source
            .file_stem()
            .and_then(|s: &std::ffi::OsStr| s.to_str())
            .unwrap_or("file");
        let extension: &str = source
            .extension()
            .and_then(|e: &std::ffi::OsStr| e.to_str())
            .unwrap_or("");

        let mut count: i32 = 1;
        while dest_path.exists() {
            let new_name: String = if extension.is_empty() {
                format!("{}_{}", stem, count)
            } else {
                format!("{}_{}.{}", stem, count, extension)
            };
            dest_path = dest_dir.join(new_name);
            count += 1;
        }
    }

    info!("Moving {:?} to {:?}", source, dest_path);

    if !dry_run {
        std::fs::rename(source, dest_path)?;
    }

    Ok(())
}
