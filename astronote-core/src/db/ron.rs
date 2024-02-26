use std::fs::{self, read_dir, DirBuilder, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use crate::{Note, SerializedNote};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

#[derive(Debug)]
pub enum Error {
    FileNotExists(PathBuf),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::FileNotExists(path) => {
                if let Some(s) = path.to_str() {
                    write!(f, "Couldn't find the path: {}", s)
                } else {
                    write!(f, "Couldn't find the path")
                }
            }
        }
    }
}

impl std::error::Error for Error {}

pub struct NoteRepository {
    database_dir: PathBuf,
}

impl NoteRepository {
    pub fn new(database_root: &Path) -> Result<Self> {
        Ok(Self {
            database_dir: PathBuf::from(database_root),
        })
    }

    pub fn create(&self, notes: Vec<Note>) -> Result<()> {
        let _result = notes
            .into_iter()
            // filter out existing metadata
            .filter(|note| !get_metadata_path_from_note(&note, &self.database_dir).exists())
            .map(|note| write_metadata(note, &self.database_dir))
            .collect::<Result<Vec<_>>>()?;
        return Ok(());
    }

    pub fn update(&self, notes: Vec<Note>) -> Result<()> {
        let _result = notes
            .into_iter()
            // filter out not-existing metadata
            .filter(|note| get_metadata_path_from_note(&note, &self.database_dir).exists())
            .map(|note| write_metadata(note, &self.database_dir))
            .collect::<Result<Vec<_>>>()?;
        return Ok(());
    }

    pub fn get_one(&self, path: &Path) -> Result<Note> {
        let path = get_metadata_path_from_path(path, &self.database_dir);
        read_metadata(&path)
    }

    pub fn get_all(&self) -> Result<Vec<Note>> {
        read_metadata_from_directory(&self.database_dir)
    }

    pub fn delete(&self, notes: Vec<Note>) -> Result<()> {
        let _result = notes
            .into_iter()
            .map(|note| delete_metadata(note, &self.database_dir))
            .collect::<Result<Vec<_>>>()?;
        return Ok(());
    }
}

fn write_metadata(note: Note, database_root: &Path) -> Result<()> {
    let metadata_path = get_metadata_path_from_note(&note, database_root);
    // create parent directory if not exists
    let parent_dir_path = metadata_path
        .parent()
        .ok_or(Error::FileNotExists(metadata_path.clone()))?;
    if !parent_dir_path.exists() {
        DirBuilder::new().recursive(true).create(parent_dir_path)?;
    }
    // convert note into ron string
    let serialized_note: SerializedNote = note.try_into()?;
    let ron = ron::ser::to_string_pretty(&serialized_note, ron::ser::PrettyConfig::default())?;
    // write into file
    let mut file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(metadata_path)?;
    file.write_all(ron.as_bytes())?;
    Ok(())
}

fn delete_metadata(note: Note, database_root: &Path) -> Result<()> {
    let metadata_path = get_metadata_path_from_note(&note, database_root);
    if !metadata_path.exists() {
        return Err(Box::new(Error::FileNotExists(metadata_path)));
    }
    fs::remove_file(metadata_path)?;
    return Ok(());
}

// 1. directory recursively
// 2. read each metadata file
// 3. and then convert the content into Note
fn read_metadata_from_directory(dir: &Path) -> Result<Vec<Note>> {
    let pathes: Vec<PathBuf> = read_dir(dir)?
        .into_iter()
        .map(|entry| Ok(entry?.path()))
        .collect::<Result<Vec<PathBuf>>>()?;
    let result = pathes
        .into_iter()
        .map(|path| {
            let result = if path.is_dir() {
                read_metadata_from_directory(&path)?
            } else {
                let note = read_metadata(&path)?;
                vec![note]
            };
            Result::Ok(result)
        })
        .try_fold(vec![], |mut acc, result_note_vec| {
            let note_vec = result_note_vec?;
            acc.extend(note_vec);
            Result::Ok(acc)
        })?;
    Ok(result)
}

fn read_metadata(filepath: &Path) -> Result<Note> {
    let ron_string: String = {
        let mut file = File::open(&filepath)?;
        let mut buf = String::new();
        let _size = file.read_to_string(&mut buf)?;
        Result::Ok(buf)
    }?;
    let serialized_note: SerializedNote = ron::from_str(&ron_string)?;
    let note: Note = serialized_note.try_into()?;
    Ok(note)
}

fn get_metadata_path_from_note(note: &Note, database_root: &Path) -> PathBuf {
    let note_path = PathBuf::from(&note.relative_path);
    let path = get_metadata_path_from_path(&note_path, database_root);
    path
}

fn get_metadata_path_from_path(path: &Path, database_root: &Path) -> PathBuf {
    let mut path = [PathBuf::from(database_root), PathBuf::from(path)]
        .iter()
        .collect::<PathBuf>();
    path.set_extension("metadata");
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{distributions::Alphanumeric, Rng};
    use std::fs::remove_dir_all;

    const TMPDIR_PATH: &str = "target/test-database";

    #[test]
    fn create_metadata() {
        let randstr: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        let path = format!("test/test/{}", randstr);
        let note = Note::new_default(&path);

        let database_root = PathBuf::from(TMPDIR_PATH);
        let repo = NoteRepository::new(&database_root).unwrap();
        repo.create(vec![note]).unwrap();

        let note_from_file = repo.get_all().unwrap();
        assert_eq!(note_from_file.len(), 1);
        println!("note: {:?}", note_from_file);
        remove_dir_all(&database_root).unwrap();
    }
}
