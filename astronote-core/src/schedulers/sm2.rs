use crate::schedulers::SchedulingAlgorithm;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuperMemo2 {
    pub counter: i64,
    pub interval: i64,
    pub easiness_factor: f64,
}

impl SuperMemo2 {
    const INTERVAL_1ST_REPETITION: i64 = 1;
    const INTERVAL_2ND_REPETITION: i64 = 6;
    
    pub fn new(counter: i64, interval: i64, easiness_factor: f64) -> Self {
        Self {
            counter,
            interval,
            easiness_factor,
        }
    }

    pub fn update_repetition_interval(&mut self, repetition_response: u8) -> i64 {
        self.counter += 1;
        self.interval = match self.counter {
            0 => unreachable!(), // todo
            1 => Self::INTERVAL_1ST_REPETITION,
            2 => Self::INTERVAL_2ND_REPETITION,
            _n if _n >= 3 => {
                if repetition_response < 3 {
                    self.counter = 0;
                    return self.update_repetition_interval(repetition_response);
                }
                let ef = self.update_easiness_factor(repetition_response);
                (self.interval as f64 *ef).ceil() as i64
            },
            _ => unreachable!(), // todo
        };
        self.interval
    }

    fn update_easiness_factor(&mut self, repetition_response: u8) -> f64 {
        let q = f64::from(repetition_response.min(5)); // todo
        self.easiness_factor += 0.1 - (5.0 - q)*(0.08 + (5.0 - q)*0.02);
        self.easiness_factor = self.easiness_factor.max(1.3);
        self.easiness_factor
    }

}

impl Default for SuperMemo2 {
    fn default() -> Self {
        Self::new(0, 0, 2.5)
    }
}

#[typetag::serde]
impl SchedulingAlgorithm for SuperMemo2 {
    fn update_and_calculate_next_datetime(&mut self, response: u8) -> chrono::NaiveDateTime {
        let response = response.min(6); // todo
        self.update_repetition_interval(response);
        let interval = chrono::Days::new(self.interval as u64);
        let today = chrono::Local::now().naive_local();
        today.checked_add_days(interval).unwrap()
    }
    fn calculate_next_datetime(&self, response: u8) -> chrono::NaiveDateTime {
        let response = response.min(6); // todo
        let interval = calculate_interval(self.counter, self.easiness_factor, response);
        let interval = chrono::Days::new(interval as u64);
        let today = chrono::Local::now().naive_local();
        today.checked_add_days(interval).unwrap()
    }
}

fn calculate_interval(counter: i64, easiness_factor: f64, repetition_response: u8) -> i64 {
    let counter = counter + 1;
    match counter {
        0 => unreachable!(), // todo
        1 => SuperMemo2::INTERVAL_1ST_REPETITION,
        2 => SuperMemo2::INTERVAL_2ND_REPETITION,
        _n if _n >= 3 => {
            if repetition_response < 3 {
                return calculate_interval(0, easiness_factor, repetition_response);
            }
            let ef = easiness_factor;
            (counter as f64 *ef).ceil() as i64
        },
        _ => unreachable!(), // todo
    }
}

#[cfg(test)]
mod tests {
    use super::SuperMemo2;

    #[test]
    fn test_1st_repetition() {
        for i in u8::MIN..u8::MAX {
            let mut sm2 = SuperMemo2::default();
            sm2.update_repetition_interval(i);
            assert_eq!(SuperMemo2::INTERVAL_1ST_REPETITION, sm2.interval);
        }
    }

    #[test]
    fn test_2nd_repetition() {
        for i in u8::MIN..u8::MAX {
            let mut sm2 = SuperMemo2 {
                counter: 1,
                interval: 0,
                easiness_factor: 2.5, 
            };
            sm2.update_repetition_interval(i);
            assert_eq!(SuperMemo2::INTERVAL_2ND_REPETITION, sm2.interval);
        }
    }

    #[test]
    fn test_3rd_repetition() {
        let response_qualities = (0..=5).collect::<Vec<u8>>();
        let easiness_factors = vec![0.0, 0.5, 1.0, 1.3, 1.5, 2.5, 3.0];
        let expected_intervals = vec![
            vec![1; easiness_factors.len()],
            vec![1; easiness_factors.len()],
            vec![1; easiness_factors.len()],
            vec![8, 8, 8, 8, 9, 15, 18],
            vec![8, 8, 8, 8, 9, 15, 18],
            vec![8, 8, 8, 9, 10, 16, 19],
        ];
        for (i, &q) in response_qualities.iter().enumerate() {
            for (j, &ef) in easiness_factors.iter().enumerate() {
                let mut sm2 = create_3rd_repetition_status(ef);
                let interval = sm2.update_repetition_interval(q);
                assert_eq!(expected_intervals[i][j], interval);
            }
        }
    }

    fn create_3rd_repetition_status(easiness_factor: f64) -> SuperMemo2 {
        SuperMemo2 { 
            counter: 3, 
            interval: 6, 
            easiness_factor,
        }
    }
}

