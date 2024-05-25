use std::net::SocketAddr;

use anyhow::bail;

use crate::config::lef_matrix_config_dto::LedMatrixConfigDto;
use crate::hw::port::Port;

#[derive(Debug)]
pub struct LedMatrixConfig {
    pub left_port: Port,
    pub right_port: Port,

    pub listen_address: Option<SocketAddr>,
    pub unix_socket: Option<String>,
}

impl TryFrom<LedMatrixConfigDto> for LedMatrixConfig {
    type Error = anyhow::Error;

    fn try_from(value: LedMatrixConfigDto) -> Result<Self, Self::Error> {
        if value.listen_address.is_none() && value.unix_socket.is_none() {
            bail!("Either listen_address or unix_socket must be set");
        }
        Ok(LedMatrixConfig {
            left_port: Port::try_from(value.left_port)?,
            right_port: Port::try_from(value.right_port)?,
            listen_address: value.listen_address,
            unix_socket: value.unix_socket,
        })
    }
}
