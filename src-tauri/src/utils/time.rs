#![allow(dead_code)]

pub fn format_duration_ms(ms: u64) -> String {
    let total_seconds = ms / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    let tenths = (ms % 1000) / 100;
    format!("{:02}:{:02}.{:01}", minutes, seconds, tenths)
}
