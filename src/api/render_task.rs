use std::io::ErrorKind;
use std::sync::{Arc, Mutex};

use crate::config::led_matrix_config::LedMatrixConfig;
use crate::hw::port::Port;
use anyhow::{anyhow, bail};
use futures_util::join;
use futures_util::FutureExt;
use image::GrayImage;
use tokio::task::JoinHandle;
use tracing::error;

#[derive(Debug)]
pub enum RenderTask {
    Left(GrayImage),
    Right(GrayImage),
    Both(GrayImage, GrayImage),
}

impl RenderTask {
    fn spawn_blocking_render_port(
        port: Arc<Mutex<Port>>,
        image: GrayImage,
    ) -> JoinHandle<anyhow::Result<()>> {
        tokio::task::spawn_blocking(move || {
            let mut port = port
                .lock()
                .map_err(|err| anyhow!("Poisoned mutex: {err:?}"))?;

            // We return ErrorKind::Other ourselves: stdlib does not use it, so we know that
            // something wrong with the port has happened, and we'll try our luck and release 
            // the handle, so we would not interfere with kernel device numbering
            if let Err(err) = port.display_gray_image(image)
                && err.kind() != ErrorKind::Other
            {
                error!(?err, ?port, "Shutting down the port");
                port.close();
            }
            Ok(())
        })
    }
    pub async fn render(self, config: Arc<LedMatrixConfig>) -> anyhow::Result<()> {
        match self {
            RenderTask::Left(left) => {
                if let Some(port) = config.left_port.as_ref() {
                    Self::spawn_blocking_render_port(port.clone(), left).await??;
                } else {
                    bail!("Left port is not configured");
                }
            }

            RenderTask::Right(right) => {
                if let Some(port) = config.right_port.as_ref() {
                    Self::spawn_blocking_render_port(port.clone(), right).await??;
                } else {
                    bail!("Right port is not configured");
                }
            }

            RenderTask::Both(left, right) => {
                match (config.left_port.as_ref(), config.right_port.as_ref()) {
                    (Some(left_port), Some(right_port)) => {
                        let (left_result, right_result) = join! {
                            Self::spawn_blocking_render_port(left_port.clone(), left).fuse(),
                            Self::spawn_blocking_render_port(right_port.clone(), right).fuse(),
                        };
                        left_result??;
                        right_result??;
                    }
                    (None, Some(_)) => bail!("Left port is not configured"),
                    (Some(_), None) => bail!("Right port is not configured"),
                    (None, None) => bail!("Both ports are not configured"),
                }
            }
        }

        Ok(())
    }
}
