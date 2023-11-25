use astronote_cli::cli::{CommandParser, Commands};
use astronote_cli::config::Config;
use astronote_cli::prompt;
use astronote_core::{
    db::NoteDatabaseInterface,
    prelude::{sqlite::*, *},
    schedulers::sm2::SuperMemo2,
};
use colored::Colorize;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // load config file
    let config = Config::try_new()?;

    // parse command line arguments
    let parser = CommandParser::parse_args();

    // use argument url if it is provided, otherwise use config file
    let db_path = parser.database_path().unwrap_or(config.database_path);
    // create DB connection
    let mut repo = NoteRepository::new(&db_path).await?;

    // main logic; subcommands
    match parser.subcommand {
        // Add file metadata to DB
        Commands::Add { file } => {
            // file paths into SerializedNote
            let serialized_notes: Vec<SerializedNote> = file
                .iter()
                .map(|path| get_validated_absolute_path(path))
                .collect::<Result<Vec<String>, _>>()?
                .iter()
                .map(|validated_path| Note::new_default(validated_path))
                .map(SerializedNote::try_from)
                .collect::<Result<Vec<SerializedNote>, _>>()?;

            // store notes into DB
            for note in &serialized_notes {
                repo.create(note).await?;
            }

            // print result
            println!(
                "{} {} {}",
                "Added".green(),
                serialized_notes.len(),
                "notes".green()
            );
        }
        // update file metadata in DB
        Commands::Update {
            file,
            quality,
            next,
            new_path,
            reset,
        } => {
            // update note metadata
            let path = get_validated_absolute_path(&file)?;
            let mut note: Note = repo.find_by_path(&path).await?.try_into()?;
            if let Some(quality) = quality {
                note.next_datetime = note
                    .scheduler
                    .update_and_calculate_next_datetime(quality as u8);
            }
            if let Some(next) = next {
                note.next_datetime = chrono::Local::now()
                    .naive_local()
                    .checked_add_days(chrono::Days::new(next as u64))
                    .ok_or("Error in adding days to datetime")?;
            }
            if let Some(new_path) = new_path {
                let new_path = get_validated_absolute_path(&new_path).unwrap_or_else(|e| {
                    panic!("Error in converting path to str: {:?}: {}", new_path, e)
                });
                note.absolute_path = new_path;
            }
            if reset {
                note.next_datetime = chrono::Local::now().naive_local();
                note.scheduler = Box::<SuperMemo2>::default();
            }
        }
        // main; review file in DB
        Commands::Review { num } => {
            // read `num` of note metadata
            let notes = repo
                .get_old_notes(num)
                .await?
                .into_iter()
                .map(SerializedNote::try_into)
                .collect::<Result<Vec<Note>, _>>()?;

            // for each file, open it with editor and update the metadata accordingly
            for mut note in notes {
                println!("{} {}", "Reviewing".green(), note.absolute_path);

                // let users choose which editor to use
                let program = match prompt!(
                    "{} [{}]: ",
                    "Enter editor to continue (or CTRL+C to cancel)".green(),
                    config.editor_command,
                ) {
                    s if s.is_empty() => config.editor_command.clone(),
                    s if !s.is_empty() => s,
                    _ => unreachable!(),
                };
                // open the note with editor
                Command::new(&program)
                    .arg(&note.absolute_path)
                    .status()?
                    .success()
                    .then_some(())
                    .ok_or("Status is not success")?;

                // update the metadata
                let quality = input_quality(&note);
                note.next_datetime = note
                    .scheduler
                    .update_and_calculate_next_datetime(quality as u8);

                // store the updated metadata into DB
                let serialized_note = SerializedNote::try_from(note)?;
                repo.update(&serialized_note).await?;

                // print result
                println!(
                    "{} {}",
                    "Next datetime:".green(),
                    &serialized_note.next_datetime
                );
                println!()
            }
        }
    }
    Ok(())
}

use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn get_validated_absolute_path(
    path: &Path,
) -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let absolute_path = path.canonicalize()?;
    assert!(absolute_path.is_absolute());
    let validated_path = validate_path(&absolute_path)?;
    let s = validated_path
        .to_string_lossy() // already validated
        .to_string();
    Ok(s)
}

fn validate_path(
    path: &PathBuf,
) -> Result<&PathBuf, Box<dyn std::error::Error + Send + Sync + 'static>> {
    path.try_exists()?
        .then_some(())
        .ok_or("File does not exist")?;
    path.is_absolute()
        .then_some(())
        .ok_or("Path is not absolute path")?;
    Ok(path)
}

fn input_quality(note: &Note) -> u32 {
    let input = prompt!(
        "{}",
        "Enter quality of answer [0-6] (or `h` for help): ".green()
    );
    // todo: the following code evaluates only first char of input
    let input_char = match input.chars().next() {
        Some(c) => c,
        None => {
            println!("Empty input");
            return input_quality(note);
        }
    };
    match input_char {
        'h' => {
            println!("Quality of answer is a number from 0 to 6");
            println!("0: complete blackout");
            println!("1: incorrect response; the correct one remembered");
            println!("2: incorrect response; where the correct one seemed easy to recall");
            println!("3: correct response recalled with serious difficulty");
            println!("4: correct response after a hesitation");
            println!("5: perfect response");
            println!("6: perfect response over multiple sessions");
            println!();
            println!("You can exist from astronote by pressing CTRL+C");
            input_quality(note)
        }
        'n' => {
            // Show next datetime depending on quality of answer
            println!("Next datetime for each quality of answer: ");
            for i in 0..=6 {
                let next_datetime = note.scheduler.calculate_next_datetime(i);
                println!("{}: {}", i, next_datetime);
            }
            input_quality(note)
        }
        '0'..='6' => input_char
            .to_digit(10)
            .unwrap_or_else(|| panic!("Error in parsing quality: {}", input_char)),
        _ => {
            println!("Invalid input");
            input_quality(note)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
