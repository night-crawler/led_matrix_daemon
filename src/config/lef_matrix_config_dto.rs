use std::net::SocketAddr;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::config::port_dto::PortDto;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct LedMatrixConfigDto {
    pub left_port: PortDto,
    pub right_port: PortDto,

    pub listen_address: Option<SocketAddr>,
    pub unix_socket: Option<String>,

    #[serde(default = "super::default_max_queue_size")]
    pub max_queue_size: usize,
    // default 1
    #[serde(default = "super::default_http_workers")]
    pub num_http_workers: usize
}


impl TryFrom<&Path> for LedMatrixConfigDto {
    type Error = anyhow::Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let content = std::fs::read_to_string(value)?;
        let config: LedMatrixConfigDto = toml::from_str(&content)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test() -> anyhow::Result<()> {
        let config = LedMatrixConfigDto {
            left_port: PortDto {
                path: "/dev/ttyACM0".to_string(),
                baud_rate: 115200,
                timeout: Duration::from_secs(2),
                keep_open: false,
            },
            right_port: PortDto {
                path: "/dev/ttyACM1".to_string(),
                baud_rate: 115200,
                timeout: Duration::from_secs(2),
                keep_open: false,
            },
            listen_address: SocketAddr::from(([127, 0, 0, 1], 45935)).into(),
            unix_socket: "/tmp/led-matrix.sock".to_string().into(),
            max_queue_size: 10,
            num_http_workers: 1,
        };

        let repr = toml::to_string(&config)?;
        println!("{repr}");
        let parsed: LedMatrixConfigDto = toml::from_str(&repr)?;
        assert_eq!(config, parsed);
        Ok(())
    }
}
