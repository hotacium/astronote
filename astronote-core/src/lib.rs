
pub mod schedulers;
pub mod db;


use serde::{Serialize, Deserialize};
use schedulers::SchedulingAlgorithm;

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    id: i64,
    absolute_path: String,
    next_datetime: chrono::NaiveDateTime,
    scheduler: Box<dyn SchedulingAlgorithm>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SerializedNote {
    id: i64,
    absolute_path: String,
    next_datetime: chrono::NaiveDateTime,
    scheduler: serde_json::Value,
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

impl TryInto<Note> for  SerializedNote {
    type Error = serde_json::Error;

    fn try_into(self) -> Result<Note, Self::Error> {
        let deserialized_scheduler: Box<dyn SchedulingAlgorithm> = serde_json::from_value(self.scheduler)?;
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
    use schedulers::sm2::SuperMemo2;
    use chrono::NaiveDateTime;
    use rand::Rng;

    #[test]
    fn serialize_note() {

        let mut rng = rand::thread_rng();
        let note = Note {
            id: 0,
            absolute_path: String::from("test"),
            next_datetime: NaiveDateTime::default(),
            scheduler: Box::new(SuperMemo2::new(rng.gen_range(0..10), rng.gen_range(0..10), rng.gen_range(0.0..5.0))),
        };

        let serialized_note: SerializedNote = note.try_into().unwrap();
        let _deserialized_note: Note = serialized_note.try_into().unwrap();
    }
}