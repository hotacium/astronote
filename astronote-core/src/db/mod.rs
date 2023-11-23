use crate::SerializedNote;
use async_trait::async_trait;
use std::marker::Sync;

#[derive(Debug)]
pub enum Error {
    FailedToCreateDBFile(std::io::Error),
    FailedToConect { url: String, source: sqlx::Error },
    FailedToMigrate(sqlx::Error),
    FailedToCreateNote(sqlx::Error),
    FailedToFindNoteByPath { path: String, source: sqlx::Error },
    FailedToFindNoteById(sqlx::Error),
    FailedToUpdateNote(sqlx::Error),
    FailedToDeleteNote(sqlx::Error),
    FailedToGetOldNotes(sqlx::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FailedToCreateDBFile(source) => {
                write!(f, "Failed to create database file: {}", source)
            }
            Self::FailedToConect { url, source } => {
                write!(f, "Failed to connect to database: {} {}", url, source)
            }
            Self::FailedToMigrate(source) => {
                write!(f, "Failed to migrate database: {}", source)
            }
            Self::FailedToCreateNote(source) => {
                write!(f, "Failed to create note: {}", source)
            }
            Self::FailedToFindNoteByPath { path, source } => {
                write!(f, "Failed to find note by path: {} {}", path, source)
            }
            Self::FailedToFindNoteById(source) => {
                write!(f, "Failed to find note by id: {}", source)
            }
            Self::FailedToUpdateNote(source) => {
                write!(f, "Failed to update note: {}", source)
            }
            Self::FailedToDeleteNote(source) => {
                write!(f, "Failed to delete note: {}", source)
            }
            Self::FailedToGetOldNotes(source) => {
                write!(f, "Failed to get old notes: {}", source)
            }
        }
    }
}

impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;

#[async_trait]
pub trait NoteDatabaseInterface<Item: Sync> {
    async fn create(&mut self, item: &Item) -> Result<i64>;
    async fn find_by_path(&mut self, path: &str) -> Result<Item>;
    async fn find_by_id(&mut self, id: i64) -> Result<Item>;
    async fn update(&mut self, item: &Item) -> Result<()>;
    async fn delete(&mut self, item: &Item) -> Result<()>;
    async fn get_old_notes(&mut self, size: usize) -> Result<Vec<Item>>;
}

pub mod sqlite {

    use super::*;

    use sqlx::sqlite::SqlitePool;

    pub struct NoteRepository {
        pool: SqlitePool,
    }

    impl NoteRepository {
        pub async fn new(path: &str) -> Result<Self> {
            // create DB file if it does not exist
            if !std::path::Path::new(&path).exists() {
                std::fs::File::create(&path)
                    .map_err(|e| Error::FailedToCreateDBFile(e))?;
            }
            let url = format!("sqlite://{}", path);
            let pool = SqlitePool::connect(&url)
                .await
                .map_err(|e| Error::FailedToConect {
                    url: url.to_string(),
                    source: e,
                })?;
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .map_err(|e| Error::FailedToMigrate(e.into()))?;
            Ok(NoteRepository { pool })
        }
    }

    #[async_trait]
    impl NoteDatabaseInterface<SerializedNote> for NoteRepository {
        async fn create(&mut self, item: &SerializedNote) -> Result<i64> {
            let id = sqlx::query(
                "INSERT INTO notes (absolute_path, next_datetime, scheduler) VALUES (?, ?, ?) ON CONFLICT(absolute_path) DO NOTHING",
            )
            .bind(&item.absolute_path)
            .bind(&item.next_datetime)
            .bind(&item.scheduler)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::FailedToCreateNote(e))?
            .last_insert_rowid();
            Ok(id)
        }

        async fn find_by_path(&mut self, path: &str) -> Result<SerializedNote> {
            let note =
                sqlx::query_as::<_, SerializedNote>("SELECT * FROM notes WHERE absolute_path = ?")
                    .bind(path)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| Error::FailedToFindNoteByPath {
                        path: path.to_string(),
                        source: e,
                    })?;
            Ok(note)
        }

        async fn find_by_id(&mut self, id: i64) -> Result<SerializedNote> {
            let note = sqlx::query_as::<_, SerializedNote>("SELECT * FROM notes WHERE id = ?")
                .bind(id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| Error::FailedToFindNoteById(e))?;
            Ok(note)
        }

        async fn update(&mut self, note: &SerializedNote) -> Result<()> {
            sqlx::query(
                "UPDATE notes SET absolute_path = ?, next_datetime = ?, scheduler = ? WHERE id = ?",
            )
            .bind(&note.absolute_path)
            .bind(&note.next_datetime)
            .bind(&note.scheduler)
            .bind(&note.id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::FailedToUpdateNote(e))?;
            Ok(())
        }

        async fn delete(&mut self, note: &SerializedNote) -> Result<()> {
            sqlx::query("DELETE FROM notes WHERE id = ?")
                .bind(&note.id)
                .execute(&self.pool)
                .await
                .map_err(|e| Error::FailedToDeleteNote(e))?;
            Ok(())
        }

        async fn get_old_notes(&mut self, size: usize) -> Result<Vec<SerializedNote>> {
            let notes = sqlx::query_as::<_, SerializedNote>(
                "SELECT * FROM notes ORDER BY next_datetime LIMIT ?",
            )
            .bind(size as u32)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::FailedToGetOldNotes(e))?;
            Ok(notes)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[tokio::test]
        async fn new_repository() {
            let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
            sqlx::migrate!("./migrations").run(&pool).await.unwrap();

            let mut _repo = NoteRepository { pool };
        }

        #[tokio::test]
        async fn insert_into_repository() {
            let mut repo = NoteRepository::new("sqlite::memory:").await.unwrap();
            let note = SerializedNote {
                id: 0,
                absolute_path: String::from("test"),
                next_datetime: chrono::NaiveDateTime::default(),
                scheduler: serde_json::Value::Null,
            };
            let _id = repo.create(&note).await.unwrap();
        }
    }
}
