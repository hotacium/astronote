use clap::{Parser, Subcommand};
use std::{io::Write, path::PathBuf};

#[derive(Debug)]
pub enum Error {
    DBURLNotFound,
    URLIsNotValidUTF8(PathBuf),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::DBURLNotFound => write!(f, "Database URL is not found"),
            Self::URLIsNotValidUTF8(path) => {
                write!(f, "Database URL is not valid UTF-8: {:?}", path)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DBURLNotFound => None,
            Self::URLIsNotValidUTF8(_) => None,
        }
    }
}

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
    pub fn database_path(&self) -> Result<String, Error> {
        let url = self.database_path.as_ref().ok_or(Error::DBURLNotFound)?;
        let url = url.to_str().ok_or(Error::URLIsNotValidUTF8(url.clone()))?;
        Ok(url.to_string())
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// add file to astronote system
    Add {
        /// target file
        #[arg(short, long, value_name = "FILE", required = true)]
        #[arg(num_args = 1.., value_delimiter = ' ')]
        file: Vec<PathBuf>,
    },
    /// updates repetition state of FILE
    Update {
        /// target file
        #[arg(short, long, value_name = "FILE", required = true)]
        file: PathBuf,
        /// quality of answer; from 0 to 6
        #[arg(short, long, value_name = "0-6")]
        #[arg(value_parser = clap::value_parser!(u64).range(0..=6))]
        quality: Option<usize>,
        /// sets next day after DAYS
        #[arg(short, long, value_name = "DAYS")]
        next: Option<usize>,
        /// mv to PATH
        #[arg(long, value_name = "PATH")]
        new_path: Option<PathBuf>,
        /// reset repetition state
        #[arg(long)]
        reset: bool,
    },
    /// starts reviewing
    Review {
        /// number of files to review
        #[arg(short, long, value_name = "NUM")]
        num: usize,
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
