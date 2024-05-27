use std::sync::{Arc, LockResult, Mutex};

use anyhow::anyhow;
use image::GrayImage;
use rayon::ThreadPool;
use serde::Serialize;
use tracing::error;

use crate::config::led_matrix_config::LedMatrixConfig;
use crate::hw::port::Port;

pub mod base64;
mod error;
pub mod files;

#[derive(Debug)]
pub enum RenderTask {
    Left(GrayImage),
    Right(GrayImage),
    Both(GrayImage, GrayImage),
}

trait SpawnImageRenderExt {
    fn spawn_image_render(&self, image: GrayImage, port: Arc<Mutex<Port>>);
}

impl SpawnImageRenderExt for ThreadPool {
    fn spawn_image_render(&self, image: GrayImage, port: Arc<Mutex<Port>>) {
        self.spawn(move || {
            let mut port = match port.lock() {
                Ok(port) => port,
                Err(err) => {
                    error!(?err, "Poisoned mutex");
                    return;
                }
            };
            if let Err(err) = port.display_gray_image(&image) {
                error!(?err, "Failed to display image");
            }
        });
    }
}

impl RenderTask {
    pub fn render(self, config: &LedMatrixConfig, thread_pool: &ThreadPool) -> anyhow::Result<()> {
        match self {
            RenderTask::Left(image) => {
                thread_pool.spawn_image_render(image, config.left_port.clone());
            }

            RenderTask::Right(image) => {
                thread_pool.spawn_image_render(image, config.right_port.clone());
            }

            RenderTask::Both(left, right) => {
                thread_pool.spawn_image_render(left, config.left_port.clone());
                thread_pool.spawn_image_render(right, config.right_port.clone());
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct AppState {
    pub sender: kanal::AsyncSender<RenderTask>,
}

#[derive(Debug, Serialize)]
pub struct RenderResponse {
    queue_len: usize,
    success: bool,
}
