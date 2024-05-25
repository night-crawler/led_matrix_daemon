use std::time::Duration;

pub mod port_dto;
pub mod lef_matrix_config_dto;
pub mod led_matrix_config;

fn yes() -> bool {
    true
}

fn no() -> bool {
    false
}

fn one() -> Option<usize> {
    Some(1)
}

fn default_baud_rate() -> u32 {
    115200
}

fn default_timeout() -> Duration {
    Duration::from_secs(2)
}
