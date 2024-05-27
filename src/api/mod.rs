use serde::Serialize;

use crate::config::led_matrix_config::LedMatrixConfig;

pub mod files;
pub mod base64;
mod error;


#[derive(Debug)]
pub struct AppState {
    pub config: LedMatrixConfig,
}


#[derive(Debug, Serialize)]
pub struct RenderResponse {
    elapsed_ms: u128,
    success: bool,
}
