use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct PortDto {
    pub path: String,

    #[serde(default = "super::default_baud_rate")]
    pub baud_rate: u32,

    #[serde(with = "humantime_serde", default = "super::default_port_timeout")]
    pub timeout: Duration,

    #[serde(with = "humantime_serde", default)]
    pub wait_delay: Option<Duration>,

    #[serde(default = "super::yes")]
    pub keep_open: bool,
}
