use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, bail};
use tracing::info;

use crate::config::lef_matrix_config_dto::LedMatrixConfigDto;
use crate::hw::port::Port;

#[derive(Debug)]
pub struct LedMatrixConfig {
    pub left_port: Option<Arc<Mutex<Port>>>,
    pub right_port: Option<Arc<Mutex<Port>>>,

    pub listen_address: Option<Arc<SocketAddr>>,
    pub unix_socket: Option<Arc<String>>,
    pub max_queue_size: usize,
    pub num_http_workers: usize,
}

impl LedMatrixConfig {
    fn log_port_version(&self, position: &str, port: Arc<Mutex<Port>>) -> anyhow::Result<()> {
        let mut port = port
            .lock()
            .map_err(|err| anyhow!("Poisoned mutex: {err:?}"))?;
        let version = port.get_device_version()?;
        info!(%version, "{position} led matrix");
        Ok(())
    }
    pub fn log_led_matrix_versions(&self) -> anyhow::Result<()> {
        if let Some(port) = self.left_port.as_ref() {
            self.log_port_version("Left", port.clone())?;
        }

        if let Some(port) = self.right_port.as_ref() {
            self.log_port_version("Right", port.clone())?;
        }

        Ok(())
    }
}

impl TryFrom<LedMatrixConfigDto> for LedMatrixConfig {
    type Error = anyhow::Error;

    fn try_from(value: LedMatrixConfigDto) -> Result<Self, Self::Error> {
        if value.listen_address.is_none() && value.unix_socket.is_none() {
            bail!("Either listen_address or unix_socket must be set");
        }

        let left_port = if let Some(left_port) = value.left_port {
            Arc::new(Mutex::new(Port::try_from(left_port)?)).into()
        } else {
            None
        };

        let right_port = if let Some(right_port) = value.right_port {
            Arc::new(Mutex::new(Port::try_from(right_port)?)).into()
        } else {
            None
        };

        Ok(LedMatrixConfig {
            left_port,
            right_port,
            listen_address: value.listen_address.map(Arc::new),
            unix_socket: value.unix_socket.map(Arc::new),

            max_queue_size: value.max_queue_size,
            num_http_workers: value.num_http_workers,
        })
    }
}
