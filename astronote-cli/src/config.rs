use anyhow::{anyhow, Context, Result};
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const CONFIG_FILE_NAME: &str = ".astronote.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub database_path: String,
    pub editor_command: String,
    pub root: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_path: String::from("./.astronote.db"),
            editor_command: String::from("vim"),
            root: String::from("./"),
        }
    }
}

impl Config {
    pub fn try_new() -> Result<Self> {
        let builder = build_config()?;
        let mut config: Self = builder
            .extract()
            .with_context(|| "Failed to build config")?;
        config.root = find_config_root_absolute(&builder)?;
        Ok(config)
    }
}

pub fn build_config() -> Result<Figment> {
    let current_path = std::env::current_dir().with_context(|| "Failed to get current path")?;
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

pub fn find_config_root_absolute(builder: &Figment) -> Result<String> {
    let path_string = find_config_root(builder)?;
    let path = Path::new(&path_string)
        .canonicalize()
        .with_context(|| format!("Failed to make path absolute: {path_string}"))?;
    Ok(path.to_string_lossy().to_string())
}

fn find_config_root(builder: &Figment) -> Result<String> {
    // 1. If `root` is set in config file, then root is where the config file resides
    let root_metadata = builder
        .find_metadata("root")
        .ok_or(anyhow!("Unable to get metadata of root"))?;
    if root_metadata.name.starts_with("TOML") {
        let path_string = builder
            .find_value("root")?
            .into_string()
            .ok_or(anyhow!("`root` in config file is not string"))?;
        let path = Path::new(&path_string);
        // if the path is relative, process it as a relative path from where cofnig file exists
        let root = if path.is_relative() {
            let source = root_metadata.source.clone().unwrap().to_string();
            let config_root = source.strip_suffix(CONFIG_FILE_NAME).unwrap().to_string();
            // TODO: canonicalize doesn't expand some expression like `~`
            let config_root = Path::new(&config_root).join(&path).canonicalize()?;
            config_root
        } else {
            path.to_path_buf()
        };
        return Ok(root.to_string_lossy().to_string());
    }

    // 2. If `root` is not set in config file, then root is where the nearest config file exists
    let config_path = builder.metadata().fold(None, |parent, md| {
        // the newer, the nearer
        if md.name.starts_with("TOML") && md.source.is_some() {
            let source = md.source.clone().unwrap();
            Some(source.to_string())
        } else {
            parent
        }
    });
    if let Some(config_path) = config_path {
        let config_root = config_path
            .strip_suffix(CONFIG_FILE_NAME)
            .unwrap()
            .to_string();
        return Ok(config_root);
    }

    // 3. If there is no config file, then root is where astronote is running
    let current_dir = std::env::current_dir()
        .with_context(|| "Failed to get current directory")?
        .to_string_lossy()
        .to_string();
    return Ok(current_dir);
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempdir::TempDir;

    #[test]
    fn find_config_root_without_config_file() {
        let builder = Figment::new().merge(Serialized::defaults(Config::default()));
        let root = find_config_root_absolute(&builder).unwrap();
        let current_dir = std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .to_string();
        assert_eq!(current_dir, root)
    }

    #[test]
    fn find_config_root_with_config_file() {
        // create temporary config file with no content
        let dir = TempDir::new("test").unwrap();
        let file_path = dir.path().join(CONFIG_FILE_NAME);
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"").unwrap();
        file.flush().unwrap();

        let builder = Figment::new()
            .merge(Serialized::defaults(Config::default()))
            .merge(Toml::file(&file_path));
        let root = find_config_root_absolute(&builder).unwrap();
        assert_eq!(dir.path().to_string_lossy().to_string(), root)
    }

    #[test]
    fn find_config_root_with_config_file_2() {
        // create temporary config file with no content
        let dir = TempDir::new("").unwrap();
        let file_path = dir.path().join(CONFIG_FILE_NAME);
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"root = '../../usr/'").unwrap();
        file.flush().unwrap();

        let builder = Figment::new()
            .merge(Serialized::defaults(Config::default()))
            .merge(Toml::file(&file_path));
        let root = find_config_root_absolute(&builder).unwrap();

        let home_dir = dir.path().join("../../usr/").canonicalize().unwrap();
        assert_eq!(home_dir.to_string_lossy().to_string(), root)
    }
}
