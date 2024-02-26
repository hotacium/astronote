use astronote_cli::cli::{CommandParser, Commands};
use astronote_cli::config::Config;
use astronote_cli::prompt;
use astronote_core::Note;
use astronote_core::{
    db::ron::*,
    schedulers::sm2::SuperMemo2,
};
use colored::Colorize;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

#[tokio::main]
async fn main() -> Result<()> {
    // load config file
    let config = Config::try_new()?;
    
    let config_root = Path::new(&config.root).canonicalize()?;

    // parse command line arguments
    let parser = CommandParser::parse_args();

    // use argument url if it is provided, otherwise use config file
    let db_path = parser.database_path().unwrap_or(config.database_path);
    // create DB connection
    let db_path = PathBuf::from(&db_path);
    let repo = NoteRepository::new(&db_path)?;

    // main logic; subcommands
    match parser.subcommand {
        // Add file metadata to DB
        Commands::Add { files } => {
            // validate file paths
            let validated_pathes = files
                .iter()
                .map(|path| get_validated_path(path, &config_root) )
                .collect::<Result<Vec<_>>>()?;
            // note from validated file
            let notes = validated_pathes
                .iter()
                .map(|path| Note::new_default(path.to_str().unwrap()) )
                .collect::<Vec<Note>>();
            repo.create(notes)?;
            // print result
            println!(
                "{} {} {}",
                "Added".green(),
                files.len(),
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
            let path = get_validated_path(&file, &config_root)?;
            let mut note: Note = repo.get_one(&path)?;
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
                let new_path = get_validated_path(&new_path, &config_root).unwrap_or_else(|e| {
                    panic!("Error in converting path to str: {:?}: {}", new_path, e)
                });
                note.relative_path = new_path.to_string_lossy().to_string();
            }
            if reset {
                note.next_datetime = chrono::Local::now().naive_local();
                note.scheduler = Box::<SuperMemo2>::default();
            }
        }
        // main; review file in DB
        Commands::Review { num } => {
            // get `num` of old notes
            let notes: Vec<Note> = {
                let mut notes = repo.get_all()?;
                notes.sort_by_key(|note| note.next_datetime);
                let now = chrono::Local::now().naive_local();
                let notes_to_review = notes.into_iter()
                    .filter(|note| note.next_datetime <= now )
                    .inspect(|note| {
                        println!("next: {:?}", note.next_datetime);
                        println!("now: {:?}", now);
                    })
                    .take(num)
                    .collect::<Vec<_>>();
                Result::Ok(notes_to_review)
            }?;
            if notes.len() < 1 {
                println!("There is no file to review (for now)!");
                return Ok(());
            }
            // for each file, open it with editor and update the metadata accordingly
            for mut note in notes {
                let validated_path = get_validated_path(&Path::new(&note.relative_path), &config_root)?;
                let absolute_path = Path::new(&validated_path).canonicalize()?;
                println!("{} {}", "Reviewing".green(), absolute_path.to_str().unwrap());

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
                    // .arg(&absolute_path)
                    .arg(&validated_path)
                    .status()?
                    .success()
                    .then_some(())
                    .ok_or("Status is not success")?;

                // update the metadata
                let quality = input_quality(&note);
                note.next_datetime = note
                    .scheduler
                    .update_and_calculate_next_datetime(quality as u8);

                // print result
                println!(
                    "{} {}",
                    "Next datetime:".green(),
                    &note.next_datetime
                );
                println!();

                // store the updated metadata into DB
                repo.update(vec![note])?;
            }
        }
    }
    Ok(())
}

use std::fs::canonicalize;
use std::path::PathBuf;
use std::{path::Path, process::Command};

fn get_validated_path(
    path: &Path,
    root: &Path,
) -> Result<PathBuf> {
    let absolute_path = canonicalize(path)?;
    if !absolute_path.try_exists()? {
        return Err(format!(
            "File does not exist. Maybe file path is not under astronote `root`. Hint: root: {}, path: {}", 
            root.to_str().unwrap(), 
            path.to_str().unwrap(),
        ).into())
    }
    let path = absolute_path.strip_prefix(&root)?;
    println!("stripped path: {:?}", path); // debug
        // .to_string_lossy() // already validated
        // .to_string();
    Ok(PathBuf::from(path))
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
