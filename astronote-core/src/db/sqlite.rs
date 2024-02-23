
use crate::db::{NoteDatabaseInterface, Error};
use crate::SerializedNote;

use async_trait::async_trait;
use sqlx::sqlite::SqlitePool;

type Result<T> = std::result::Result<T, Error>;

pub struct NoteRepository {
    pool: SqlitePool,
}

impl NoteRepository {
    pub async fn new(path: &str) -> Result<Self> {
        // create DB file if it does not exist
        if !std::path::Path::new(&path).exists() {
            std::fs::File::create(path).map_err(Error::FailedToCreateDBFile)?;
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
            "INSERT INTO notes (relative_path, next_datetime, scheduler) VALUES (?, ?, ?) ON CONFLICT(relative_path) DO NOTHING",
        )
        .bind(&item.relative_path)
        .bind(item.next_datetime)
        .bind(&item.scheduler)
        .execute(&self.pool)
        .await
        .map_err(Error::FailedToCreateNote)?
        .last_insert_rowid();
        Ok(id)
    }

    async fn find_by_path(&mut self, path: &str) -> Result<SerializedNote> {
        let note =
            sqlx::query_as::<_, SerializedNote>("SELECT * FROM notes WHERE relative_path = ?")
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
            .map_err(Error::FailedToFindNoteById)?;
        Ok(note)
    }

    async fn update(&mut self, note: &SerializedNote) -> Result<()> {
        sqlx::query(
            "UPDATE notes SET relative_path = ?, next_datetime = ?, scheduler = ? WHERE id = ?",
        )
        .bind(&note.relative_path)
        .bind(note.next_datetime)
        .bind(&note.scheduler)
        .bind(note.id)
        .execute(&self.pool)
        .await
        .map_err(Error::FailedToUpdateNote)?;
        Ok(())
    }

    async fn delete(&mut self, note: &SerializedNote) -> Result<()> {
        sqlx::query("DELETE FROM notes WHERE id = ?")
            .bind(note.id)
            .execute(&self.pool)
            .await
            .map_err(Error::FailedToDeleteNote)?;
        Ok(())
    }

    async fn get_old_notes(&mut self, size: usize) -> Result<Vec<SerializedNote>> {
        let notes = sqlx::query_as::<_, SerializedNote>(
            "SELECT * FROM notes ORDER BY next_datetime LIMIT ?",
        )
        .bind(size as u32)
        .fetch_all(&self.pool)
        .await
        .map_err(Error::FailedToGetOldNotes)?;
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
            relative_path: String::from("test"),
            next_datetime: chrono::NaiveDateTime::default(),
            scheduler: serde_json::Value::Null,
        };
        let _id = repo.create(&note).await.unwrap();
    }
}
