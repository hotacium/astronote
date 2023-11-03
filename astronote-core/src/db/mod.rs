use std::error::Error;
use std::marker::Sync;
type Result<T> = std::result::Result<T, Box<dyn Error>>;
use async_trait::async_trait;

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
    use crate::SerializedNote;

    use super::*;

    use sqlx::sqlite::SqlitePool;

    pub struct NoteRepository {
        pool: SqlitePool,
    }

    impl NoteRepository {
        pub async fn new(url: &str) -> Result<Self> {
            let pool = SqlitePool::connect(url).await?;
            sqlx::migrate!("./migrations").run(&pool).await?;
            Ok(NoteRepository { pool })
        }
    }

    #[async_trait]
    impl NoteDatabaseInterface<SerializedNote> for NoteRepository {
        async fn create(&mut self, item: &SerializedNote) -> Result<i64> {
            let id = sqlx::query("INSERT INTO notes (absolute_path, next_datetime, scheduler) VALUES (?, ?, ?)")
                .bind(&item.absolute_path)
                .bind(&item.next_datetime)
                .bind(&item.scheduler)
                .execute(&self.pool)
                .await?
                .last_insert_rowid();
            Ok(id)
        }

        async fn find_by_path(&mut self, path: &str) -> Result<SerializedNote> {
            let note = sqlx::query_as::<_, SerializedNote>("SELECT * FROM notes WHERE absolute_path = ?")
                .bind(path)
                .fetch_one(&self.pool)
                .await?;
            Ok(note)
        }

        async fn find_by_id(&mut self, id: i64) -> Result<SerializedNote> {
            let note = sqlx::query_as::<_, SerializedNote>("SELECT * FROM notes WHERE id = ?")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;
            Ok(note)
        }

        async fn update(&mut self, note: &SerializedNote) -> Result<()> {
            sqlx::query("UPDATE notes SET absolute_path = ?, next_datetime = ?, scheduler = ? WHERE id = ?")
                .bind(&note.absolute_path)
                .bind(&note.next_datetime)
                .bind(&note.scheduler)
                .bind(&note.id)
                .execute(&self.pool)
                .await?;
            Ok(())
        }

        async fn delete(&mut self, note: &SerializedNote) -> Result<()> {
            sqlx::query("DELETE FROM notes WHERE id = ?")
                .bind(&note.id)
                .execute(&self.pool)
                .await?;
            Ok(())
        }

        async fn get_old_notes(&mut self, size: usize) -> Result<Vec<SerializedNote>> {
            let notes = sqlx::query_as::<_, SerializedNote>("SELECT * FROM notes ORDER BY next_datetime LIMIT ?")
                .bind(size as u32)
                .fetch_all(&self.pool)
                .await?;
            Ok(notes)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[tokio::test]
        async fn new_repository() {
            let pool = SqlitePool::connect("sqlite::memory:")
                .await
                .unwrap();
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .unwrap();

            let mut _repo = NoteRepository { pool };
        }
    }
}