
pub mod sm2;

use std::fmt::Debug;


#[typetag::serde(tag = "type")]
pub trait SchedulingAlgorithm : Debug + Send + Sync {
    fn next_day(&mut self, response: u8) -> chrono::NaiveDateTime;
}

