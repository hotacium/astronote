use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use std::{io::Write, path::PathBuf};

#[derive(Parser)]
#[command(about = "astronote is a tool for spaced repetition", long_about = None)]
#[command(version = "0.1")]
#[command(arg_required_else_help(true))]
pub struct CommandParser {
    #[command(subcommand)]
    pub subcommand: Commands,
    /// path to database (default: ./.astronote.db))
    pub database_path: Option<PathBuf>,
}

impl CommandParser {
    pub fn parse_args() -> Self {
        Self::parse()
    }
    pub fn database_path(&self) -> Result<String> {
        let path = self
            .database_path
            .as_ref()
            .ok_or(anyhow!("Path to metadata database was not provided"))?;
        let path = path
            .to_str()
            .ok_or(anyhow!("{:?} is not valid UTF-8", path))?;
        Ok(path.to_string())
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add file to astronote system.
    Add {
        /// Target file to add to astrnote.
        #[arg(value_name = "FILE")]
        #[arg(num_args = 1.., value_delimiter = ' ')]
        files: Vec<PathBuf>,
    },
    /// Start reviewing.
    Review {
        /// Number of files to review.
        #[arg(short, long, value_name = "NUM")]
        num: Option<usize>,
        /// Ignore schedule to review; You can review files regardless of their next schedule.
        #[arg(short, long)]
        ignore_schedule: Option<bool>,
    },
}

pub fn read_line() -> String {
    let stdin = std::io::stdin();
    let mut buf = String::new();
    std::io::stdout().flush().unwrap();
    stdin.read_line(&mut buf).unwrap();
    buf.trim().to_string()
}

#[macro_export]
macro_rules! prompt {
    () => {
        $crate::cli::read_line()
    };
    ($($arg:tt)*) => {{
        print!("{}", format_args!($($arg)*));
        $crate::cli::read_line()
    }};
}
