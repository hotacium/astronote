use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const CONFIG_FILE_NAME: &str = ".config/astronote.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub database_path: String,
    pub editor_command: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_path: String::from("./.astronote.db"),
            editor_command: String::from("vim"),
        }
    }
}

impl Config {
    pub fn try_new() -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let builder = build_config()?;
        let config: Self = builder.extract()?;
        Ok(config)
    }
}

pub fn build_config() -> Result<Figment, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let current_path = std::env::current_dir()?;
    let builder = Figment::new().merge(Serialized::defaults(Config::default()));
    Ok(merge_config_file(builder, current_path))
}

// merge config file from root directory to current directory
fn merge_config_file(mut builder: Figment, dir_path: PathBuf) -> Figment {
    let config_path = dir_path.join(CONFIG_FILE_NAME);
    // recursively merge config file from root
    // **merge parent directory first**
    if let Some(parent) = dir_path.parent() {
        builder = merge_config_file(builder, parent.to_path_buf());
    }
    // overwrite config builder with current directory
    if let Some(config_path) = config_path.to_str() {
        builder = builder.merge(Toml::file(config_path));
    }
    builder
}
