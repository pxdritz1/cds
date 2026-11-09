use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use directories::BaseDirs;

pub struct Paths {
    pub config: PathBuf,
    pub cache: PathBuf,
    pub data: PathBuf,
    pub socket: PathBuf,
}

impl Paths {
    pub fn discover() -> Result<Self> {
        let base = BaseDirs::new().context("could not resolve the user's XDG directories")?;
        let runtime = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .context("XDG_RUNTIME_DIR is not set; CDS needs a user runtime directory")?;

        Ok(Self {
            config: base.config_dir().join("cds"),
            cache: base.cache_dir().join("cds"),
            data: base.data_dir().join("cds"),
            socket: runtime.join("cds.sock"),
        })
    }

    pub fn create_user_directories(&self) -> Result<()> {
        // Keep the old Amane configuration usable: copy only files missing from
        // the new CDS directories and never delete or overwrite the old data.
        migrate_legacy_directory(
            &self.config,
            &self.config.parent().unwrap_or(&self.config).join("chloe-dynamic-shell"),
        )?;
        migrate_legacy_directory(
            &self.cache,
            &self.cache.parent().unwrap_or(&self.cache).join("chloe-dynamic-shell"),
        )?;
        migrate_legacy_directory(
            &self.data,
            &self.data.parent().unwrap_or(&self.data).join("chloe-dynamic-shell"),
        )?;

        std::fs::create_dir_all(&self.config)?;
        std::fs::create_dir_all(&self.cache)?;
        std::fs::create_dir_all(&self.data)?;
        Ok(())
    }
}

fn migrate_legacy_directory(destination: &Path, legacy: &Path) -> Result<()> {
    if !legacy.is_dir() {
        return Ok(());
    }

    copy_missing_tree(legacy, destination)
        .with_context(|| format!("could not import legacy CDS data from {}", legacy.display()))
}

fn copy_missing_tree(source: &Path, destination: &Path) -> Result<()> {
    std::fs::create_dir_all(destination)?;

    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry.file_type()?;

        // Skip symlinks: importing them could make the new configuration
        // point outside the user's XDG directories.
        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir() {
            copy_missing_tree(&source_path, &destination_path)?;
        } else if file_type.is_file() && !destination_path.exists() {
            std::fs::copy(&source_path, &destination_path)?;
        }
    }

    Ok(())
}

pub fn socket_path() -> Result<PathBuf> {
    Ok(Paths::discover()?.socket)
}

#[cfg(test)]
mod tests {
    use super::copy_missing_tree;

    #[test]
    fn migration_copies_missing_files_without_overwriting_existing_files() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let old = temporary.path().join("old");
        let new = temporary.path().join("new");
        std::fs::create_dir_all(old.join("nested")).expect("old nested directory");
        std::fs::create_dir_all(new.join("nested")).expect("new nested directory");
        std::fs::write(old.join("settings.conf"), "old settings").expect("old settings");
        std::fs::write(old.join("nested/cache.bin"), "old cache").expect("old cache");
        std::fs::write(new.join("settings.conf"), "new settings").expect("new settings");

        copy_missing_tree(&old, &new).expect("migration");

        assert_eq!(
            std::fs::read_to_string(new.join("settings.conf")).unwrap(),
            "new settings"
        );
        assert_eq!(
            std::fs::read_to_string(new.join("nested/cache.bin")).unwrap(),
            "old cache"
        );
        assert_eq!(
            std::fs::read_to_string(old.join("settings.conf")).unwrap(),
            "old settings"
        );
    }
}
