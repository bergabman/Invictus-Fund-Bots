use std::time::{SystemTime, UNIX_EPOCH};
pub struct Epoch {}

impl Epoch {
    pub fn now() -> u64 {
        let now = SystemTime::now();
        let epoch = now.duration_since(UNIX_EPOCH).unwrap();
        epoch.as_secs()
    }

    pub fn weeks_ago(epoch: u64, week: u64) -> u64 {
        let week_in_sec = 7*24*60*60;
        let final_number = epoch - (week * week_in_sec);
        final_number
    }

    pub fn days_ago(number: u64) -> u64 {
        let days_in_sec = 24*60*60;
        let final_number = number - (number * days_in_sec);
        final_number
    }

    // pub fn hour(number: u64) -> u64 {
    //     Self::minute(number) * 60
    // }

    // pub fn day(number: u64) -> u64 {
    //     Self::hour(number) * 24
    // }

    // pub fn week(number: u64) -> u64 {
    //     Self::day(number) * 7
    // }

    // pub fn year(number: u64) -> u64 {
    //     Self::day(number) * 365
    // }
}