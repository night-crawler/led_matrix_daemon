use serde::Serialize;

use crate::api::render_task::RenderTask;

pub mod base64;
mod error;
pub mod files;
mod render_task;


#[derive(Debug)]
pub struct AppState {
    pub sender: kanal::AsyncSender<RenderTask>,
}

#[derive(Debug, Serialize)]
pub struct RenderResponse {
    queue_len: usize,
    success: bool,
}
