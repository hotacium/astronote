pub mod sm2;

use std::fmt::Debug;

#[typetag::serde(tag = "type")]
pub trait SchedulingAlgorithm: Debug + Send + Sync {
    fn update_and_calculate_next_datetime(&mut self, response: u8) -> chrono::NaiveDateTime;
    fn calculate_next_datetime(&self, response: u8) -> chrono::NaiveDateTime;
}
