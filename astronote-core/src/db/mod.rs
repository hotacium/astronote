
pub mod sqlite;

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

