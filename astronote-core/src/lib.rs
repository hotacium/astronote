pub mod db;
pub mod schedulers;

pub mod prelude {
    pub use crate::schedulers::SchedulingAlgorithm;
    pub use crate::Note;
    pub use crate::SerializedNote;
    pub mod sqlite {
        pub use crate::db::sqlite::NoteRepository;
    }
}

use schedulers::{sm2::SuperMemo2, SchedulingAlgorithm};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub absolute_path: String,
    pub next_datetime: chrono::NaiveDateTime,
    pub scheduler: Box<dyn SchedulingAlgorithm>,
}

impl Note {
    pub fn new(
        id: i64,
        absolute_path: &str,
        next_datetime: &chrono::NaiveDateTime,
        scheduler: Box<dyn SchedulingAlgorithm>,
    ) -> Self {
        Self {
            id,
            absolute_path: String::from(absolute_path),
            next_datetime: next_datetime.clone(),
            scheduler,
        }
    }

    pub fn new_default(absolute_path: &str) -> Self {
        let now = chrono::Local::now().naive_local();
        let sm2 = Box::new(SuperMemo2::default());
        Self::new(0, absolute_path, &now, sm2)
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SerializedNote {
    pub id: i64,
    pub absolute_path: String,
    pub next_datetime: chrono::NaiveDateTime,
    pub scheduler: serde_json::Value,
}

impl TryFrom<Note> for SerializedNote {
    type Error = serde_json::Error;

    fn try_from(value: Note) -> Result<Self, Self::Error> {
        let serialized_scheduler = serde_json::to_value(value.scheduler)?;
        Ok(SerializedNote {
            id: value.id,
            absolute_path: value.absolute_path,
            next_datetime: value.next_datetime,
            scheduler: serialized_scheduler,
        })
    }
}

impl TryInto<Note> for SerializedNote {
    type Error = serde_json::Error;

    fn try_into(self) -> Result<Note, Self::Error> {
        let deserialized_scheduler: Box<dyn SchedulingAlgorithm> =
            serde_json::from_value(self.scheduler)?;
        Ok(Note {
            id: self.id,
            absolute_path: self.absolute_path,
            next_datetime: self.next_datetime,
            scheduler: deserialized_scheduler,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use rand::Rng;
    use schedulers::sm2::SuperMemo2;

    #[test]
    fn serialize_note() {
        let mut rng = rand::thread_rng();
        let note = Note {
            id: 0,
            absolute_path: String::from("test"),
            next_datetime: NaiveDateTime::default(),
            scheduler: Box::new(SuperMemo2::new(
                rng.gen_range(0..10),
                rng.gen_range(0..10),
                rng.gen_range(0.0..5.0),
            )),
        };

        let serialized_note: SerializedNote = note.try_into().unwrap();
        let _deserialized_note: Note = serialized_note.try_into().unwrap();
    }
}
