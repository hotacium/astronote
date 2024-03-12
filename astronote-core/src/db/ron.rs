use anyhow::{anyhow, Context, Result};
use std::fs::{self, read_dir, DirBuilder, File};
use std::io::{prelude::*, ErrorKind};
use std::path::{Path, PathBuf};

use crate::{Note, SerializedNote};

pub struct NoteRepository {
    database_dir: PathBuf,
}

impl NoteRepository {
    pub fn new(database_root: &Path) -> Result<Self> {
        if !database_root.exists() {
            DirBuilder::new()
                .recursive(true)
                .create(database_root)
                .with_context(|| {
                    format!("Failed to create database directory in {database_root:?}")
                })?;
        }
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
    // create directory to store metadata under `database_root` if not exists
    let parent_dir_path = metadata_path
        .parent()
        .ok_or(anyhow!(
            "Failed to get directory path of {:?}",
            metadata_path
        ))
        .with_context(|| format!("Failed to get parent directory of {metadata_path:?}"))?;
    if !parent_dir_path.exists() {
        DirBuilder::new()
            .recursive(true)
            .create(parent_dir_path)
            .with_context(|| format!("Failed to create metadata directory: {parent_dir_path:?}"))?;
    }
    // convert note into ron string
    let serialized_note: SerializedNote = note
        .try_into()
        .with_context(|| "Failed to serialize note structure")?;
    let ron = ron::ser::to_string_pretty(&serialized_note, ron::ser::PrettyConfig::default())
        .with_context(|| "Failed to generate RON string from serialized object")?;
    // write into file
    let mut file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(metadata_path)
        .with_context(|| "Failed to open or create metadata file: {metadata_path:?}")?;
    file.write_all(ron.as_bytes())
        .with_context(|| "Failed to write metadata to {metadata_path:?}")?;
    Ok(())
}

fn delete_metadata(note: Note, database_root: &Path) -> Result<()> {
    let metadata_path = get_metadata_path_from_note(&note, database_root);
    if !metadata_path.exists() {
        return Err(anyhow!(
            "Failed to find metadata file to remove: {:?}",
            metadata_path
        ));
    }
    fs::remove_file(&metadata_path)
        .with_context(|| format!("Failed to remove metadata file: {:?}", metadata_path))?;
    return Ok(());
}

// 1. directory recursively
// 2. read each metadata file
// 3. and then convert the content into Note
fn read_metadata_from_directory(dir: &Path) -> Result<Vec<Note>> {
    let pathes: Vec<PathBuf> = read_dir(dir)
        .with_context(|| format!("Failed to read entries of {dir:?}"))?
        .into_iter()
        .map(|entry| Ok(entry?.path()))
        .collect::<Result<Vec<PathBuf>>>()
        .with_context(|| {
            format!("There's some sort of intermittent IO error during reading directory: {dir:?}")
        })?;
    let result = pathes
        .into_iter()
        .map(|path| {
            let result = if path.is_dir() {
                read_metadata_from_directory(&path)?
            } else {
                let note = read_metadata(&path)
                    .with_context(|| format!("Failed to read note metadata from {path:?}"))?;
                vec![note]
            };
            anyhow::Ok(result)
        })
        .try_fold(vec![], |mut acc, result_note_vec| {
            let note_vec = result_note_vec?;
            acc.extend(note_vec);
            anyhow::Ok(acc)
        })?;
    Ok(result)
}

fn read_metadata(filepath: &Path) -> Result<Note> {
    let ron_string: String = {
        let file = File::open(&filepath).with_context(|| format!("Failed to open {filepath:?}"))?;
        let mut buf = String::new();
        // NOTE: metadata must be < 10 KiB; rather fail to prevent zipbomb
        let _size = LimitReader::new(file, 10 * 1024)
            .read_to_string(&mut buf)
            .with_context(|| format!("Failed to read file content of {filepath:?}"))?;
        anyhow::Ok(buf)
    }?;
    let serialized_note: SerializedNote = ron::from_str(&ron_string).with_context(|| {
        format!("Failed to generate serialized metadata object from RON of {filepath:?}")
    })?;
    let note: Note = serialized_note
        .try_into()
        .with_context(|| "Failed to deserialize serialized metadata")?;
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

struct LimitReader<R: Read> {
    reader: R,
    limit: usize,
}

impl<R> LimitReader<R>
where
    R: Read,
{
    pub fn new(reader: R, limit: usize) -> Self {
        Self { reader, limit }
    }
}

impl<R> Read for LimitReader<R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.len() > self.limit {
            return Err(std::io::Error::new(ErrorKind::Other, "too many bytes"));
        }
        let size = self.reader.read(buf)?;
        self.limit -= size;
        Ok(size)
    }
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
