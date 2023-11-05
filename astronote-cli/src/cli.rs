use clap::{Parser, Subcommand};
use glob::glob;
use std::{io::Write, path::PathBuf};
// type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
#[command(about = "astronote is a tool for spaced repetition", long_about = None)]
#[command(version = "0.1")]
#[command(arg_required_else_help(true))]
pub struct CommandParser {
    #[command(subcommand)]
    pub subcommand: Commands,
    // #[arg(short, long, value_name="PATH")]
    // #[arg(value_parser = path_parser)]
    pub database_url: Option<PathBuf>,
}

impl CommandParser {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// add file to astronote system
    Add {
        /// target file
        #[arg(short, long, value_name = "FILE", required = true)]
        #[arg(num_args = 1.., value_delimiter = ' ')]
        // #[arg(value_parser = path_parser)]
        file: Vec<PathBuf>,
    },
    /// updates repetition state of FILE
    Update {
        /// target file
        #[arg(short, long, value_name = "FILE", required = true)]
        // #[arg(value_parser = path_parser)]
        file: PathBuf,
        /// quality of answer; from 0 to 6
        #[arg(short, long, value_name = "0-6")]
        #[arg(value_parser = quality_validator)]
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
    /// resets repetition state of FILE
    Reset {
        /// target file
        #[arg(short, long, value_name = "FILE")]
        #[arg(value_parser = path_parser)]
        file: Vec<PathBuf>,
    },
    /// updates path of FILE
    Move {
        #[arg(short, long, value_name = "FILE")]
        src: PathBuf,
        #[arg(short, long, value_name = "FILE")]
        dst: PathBuf,
    },
    /// starts reviewing
    Review {
        /// number of files to review
        #[arg(short, long, value_name = "NUM")]
        num: usize,
    },
}

fn path_parser(path: &str) -> Result<Vec<PathBuf>, String> {
    let paths = glob(path)
        .map_err(|e| format!("Error in parseing path: {}", e))?
        .map(|p| p.map_err(|e| format!("Error in parseing path: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;
    paths
        .into_iter()
        .map(|p| {
            p.canonicalize()
                .map_err(|e| format!("Error in parseing path: {}", e))
        })
        .collect::<Result<Vec<_>, _>>()
}

fn quality_validator(num: &str) -> Result<usize, String> {
    let num: usize = num
        .parse()
        .or(Err(format!("Quality must be a number, from 0 to 6")))?;

    if (0..=6).contains(&num) {
        Ok(num)
    } else {
        Err(format!("Quality must be from 0 to 6"))
    }
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
