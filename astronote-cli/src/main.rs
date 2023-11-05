use astronote_cli::cli::{CommandParser, Commands};
use astronote_cli::prompt;
use astronote_core::{
    db::NoteDatabaseInterface,
    prelude::{sqlite::*, *},
};
use colored::Colorize;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() {
    let url = "sqlite://astronote.db";

    let parser = CommandParser::parse_args();
    let url = parser
        .database_url
        .unwrap_or_else(|| panic!("Error in parsing database url"));
    let url = url
        .to_str()
        .unwrap_or_else(|| panic!("Error in converting database url to str"));

    let mut repo = NoteRepository::new(url)
        .await
        .unwrap_or_else(|e| panic!("Error in connecting to repository: {}", e));

    match parser.subcommand {
        Commands::Add { file } => {
            let absolute_paths = file
                .iter()
                .map(|p| {
                    p.canonicalize()
                        .map_err(|e| panic!("Error in parseing path: {}", e))
                })
                .collect::<Result<Vec<_>>>()
                .unwrap_or_else(|e| panic!("Error in validating path: {}", e));
            let validated_paths = absolute_paths
                .iter()
                .map(validate_path)
                .collect::<Result<Vec<_>>>() // collect once here to evaluate `validate_path` to check if all paths are valid
                .unwrap_or_else(|e| panic!("Error in validating path: {}", e));

            let serialized_notes: Vec<SerializedNote> = validated_paths
                .into_iter()
                .map(|path| {
                    // PathBuf -> Note
                    let path = path
                        .to_str()
                        .unwrap_or_else(|| panic!("Error in converting path to str: {:?}", path));
                    Note::new_default(path)
                })
                .map(|path| {
                    // Note -> SerializedNote
                    SerializedNote::try_from(path)
                        .unwrap_or_else(|e| panic!("Error in serializing note: {}", e))
                })
                .collect();

            // store notes into DB
            for note in &serialized_notes {
                repo.create(note)
                    .await
                    .unwrap_or_else(|e| panic!("Error in adding note to repository: {}", e));
            }

            println!(
                "{} {} {}",
                "Added".green(),
                serialized_notes.len(),
                "notes".green()
            );
        }

        Commands::Update {
            file,
            quality,
            next,
            new_path,
            reset,
        } => {
            // update note metadata
        }
        Commands::Reset { file } => {
            // reset note metadata
        }
        Commands::Move { src, dst } => {
            // move note
            // may be not needed
        }
        Commands::Review { num } => {
            // read `num` of note metadata
            let notes = repo
                .get_old_notes(num)
                .await
                .unwrap_or_else(|e| panic!("Error in getting old notes: {}", e))
                .into_iter()
                .map(|note| {
                    SerializedNote::try_into(note)
                        .unwrap_or_else(|e| panic!("Error in deserializing note: {}", e))
                })
                .collect::<Vec<Note>>();

            // for each file
            for mut note in notes {
                println!("{} {}", "Reviewing".green(), note.absolute_path);

                let program = prompt!(
                    "{}",
                    "Enter editor to continue (or CTRL+D to cancel): ".green()
                );
                Command::new(&program)
                    .arg(&note.absolute_path)
                    .status()
                    .unwrap_or_else(|e| panic!("Error in executing `{}`: {}", &program, e))
                    .success()
                    .then(|| ())
                    .ok_or("Status is not success")
                    .unwrap_or_else(|e| panic!("Error in executing `{}`: {}", &program, e));

                let quality = input_quality(&note);
                note.next_datetime = note
                    .scheduler
                    .update_and_calculate_next_datetime(quality as u8);

                let serialized_note = SerializedNote::try_from(note)
                    .unwrap_or_else(|e| panic!("Error in serializing note: {}", e));
                repo.update(&serialized_note)
                    .await
                    .unwrap_or_else(|e| panic!("Error in updating note: {}", e));

                println!(
                    "{} {}",
                    "Next datetime:".green(),
                    &serialized_note.next_datetime
                );
                println!("")
            }
        }
    }
}

use std::{path::PathBuf, process::Command};

fn validate_path(path: &PathBuf) -> Result<&PathBuf> {
    path.try_exists()?
        .then(|| ())
        .ok_or("File does not exist")?;
    path.is_absolute()
        .then(|| ())
        .ok_or("Path is not absolute path")?;
    Ok(path)
}

fn input_quality(note: &Note) -> u32 {
    let input = prompt!(
        "{}",
        "Enter quality of answer [0-6] (or `h` for help): ".green()
    );
    // todo: the following code evaluates only first char of input
    let input_char = input
        .chars()
        .next()
        .unwrap_or_else(|| panic!("Error in parsing quality: input is empty"));
    match input_char {
        'h' => {
            println!("{}", "Quality of answer is a number from 0 to 6".green());
            println!("0: complete blackout");
            println!("1: incorrect response; the correct one remembered");
            println!("2: incorrect response; where the correct one seemed easy to recall");
            println!("3: correct response recalled with serious difficulty");
            println!("4: correct response after a hesitation");
            println!("5: perfect response");
            println!("6: perfect response over multiple sessions");
            println!("");
            println!("You can exist from astronote by pressing CTRL+D");
            input_quality(note)
        }
        'n' => {
            // Show next datetime depending on quality of answer
            println!("{}", "Next datetime for each quality of answer: ".green());
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
