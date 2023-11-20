use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub database_path: String,
    pub editor_command: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_path: String::from("astronote.db"),
            editor_command: String::from("vim"),
        }
    }
}

pub fn find_config(current_path: &PathBuf) -> Option<PathBuf> {
    let mut config_path = current_path.clone();
    config_path.push(".astronote.toml");
    if !config_path.exists() {
        return match current_path.parent() {
            Some(p) => find_config(&p.to_path_buf()),
            None => return None,
        };
    }
    Some(config_path)
}
