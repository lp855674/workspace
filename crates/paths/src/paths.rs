use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub root: PathBuf,
    pub settings_file: PathBuf,
    pub keymap_file: PathBuf,
    pub database_file: PathBuf,
}

impl AppPaths {
    pub fn for_root(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();

        Self {
            settings_file: root.join("settings.json"),
            keymap_file: root.join("keymap.json"),
            database_file: root.join("framework.db"),
            root,
        }
    }
}
