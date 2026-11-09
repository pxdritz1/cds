use std::path::PathBuf;

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
            config: base.config_dir().join("chloe-dynamic-shell"),
            cache: base.cache_dir().join("chloe-dynamic-shell"),
            data: base.data_dir().join("chloe-dynamic-shell"),
            socket: runtime.join("cds.sock"),
        })
    }

    pub fn create_user_directories(&self) -> Result<()> {
        std::fs::create_dir_all(&self.config)?;
        std::fs::create_dir_all(&self.cache)?;
        std::fs::create_dir_all(&self.data)?;
        Ok(())
    }
}

pub fn socket_path() -> Result<PathBuf> {
    Ok(Paths::discover()?.socket)
}
