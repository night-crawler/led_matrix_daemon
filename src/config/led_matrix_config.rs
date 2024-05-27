use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use anyhow::bail;

use crate::config::lef_matrix_config_dto::LedMatrixConfigDto;
use crate::hw::port::Port;

#[derive(Debug)]
pub struct LedMatrixConfig {
    pub left_port: Arc<Mutex<Port>>,
    pub right_port: Arc<Mutex<Port>>,

    pub listen_address: Option<Arc<SocketAddr>>,
    pub unix_socket: Option<Arc<String>>,
    pub max_queue_size: usize,
    pub num_http_workers: usize
}

impl TryFrom<LedMatrixConfigDto> for LedMatrixConfig {
    type Error = anyhow::Error;

    fn try_from(value: LedMatrixConfigDto) -> Result<Self, Self::Error> {
        if value.listen_address.is_none() && value.unix_socket.is_none() {
            bail!("Either listen_address or unix_socket must be set");
        }
        Ok(LedMatrixConfig {
            left_port: Arc::new(Mutex::new(Port::try_from(value.left_port)?)),
            right_port: Arc::new(Mutex::new(Port::try_from(value.right_port)?)),
            listen_address: value.listen_address.map(Arc::new),
            unix_socket: value.unix_socket.map(Arc::new),

            max_queue_size: value.max_queue_size,
            num_http_workers: value.num_http_workers,
        })
    }
}
