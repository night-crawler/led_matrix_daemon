use std::sync::Arc;

use serde::Serialize;

use crate::api::render_task::RenderTask;
use crate::config::led_matrix_config::LedMatrixConfig;

pub mod base64;
mod error;
pub mod files;
mod render_task;

#[derive(Debug)]
pub struct AppState {
    pub sender: kanal::AsyncSender<RenderTask>,
    pub config: Arc<LedMatrixConfig>,
}

#[derive(Debug, Serialize)]
pub struct RenderResponse {
    queue_len: usize,
    queued: bool,
}
