//! Empty directory removing and handling operations

use walkdir::WalkDir;
use anyhow::Result;
use tracing::info;

pub fn remove_empty_dirs(root: &std::path::Path, dry_run: bool) -> Result<()> {

    for entry in WalkDir::new(root)
        .contents_first(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_dir() || entry.path() == root {
            continue;
        }

        let path: &std::path::Path = entry.path();

        let is_empty: bool = match std::fs::read_dir(path) {
            Ok(mut entries) => entries.next().is_none(),
            Err(e) => {
                tracing::error!("Could not read directory {:?}: {}", path, e);
                false
            }
        };

        if is_empty {
            info!("Removing empty directory: {:?}", path);

            if !dry_run {
                std::fs::remove_dir(path)?;
            }
        }
    }

    Ok(())
}
