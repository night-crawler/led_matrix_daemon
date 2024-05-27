use std::io::Cursor;
use std::sync::{Arc, Mutex};

use actix_web::{post, web};
use image::{DynamicImage, GrayImage, ImageBuffer};
use serde::Deserialize;
use serde_with::base64::Base64;
use serde_with::serde_as;
use tokio::task::JoinSet;
use tracing::info;

use crate::api::AppState;
use crate::api::error::ApiError;
use crate::config::led_matrix_config::LedMatrixConfig;
use crate::hw::port::Port;

#[serde_as]
#[derive(Deserialize, Debug)]
struct SingleRenderRequest {
    #[serde_as(as = "Base64")]
    left_image: Option<Vec<u8>>,
    #[serde_as(as = "Base64")]
    right_image: Option<Vec<u8>>,
}

impl SingleRenderRequest {
    fn get_left_image(&self) -> Result<Option<GrayImage>, ApiError> {
        match &self.left_image {
            Some(buf) => Ok(Some(Self::buf_to_gray_image(buf)?)),
            None => Ok(None),
        }
    }

    fn get_right_image(&self) -> Result<Option<GrayImage>, ApiError> {
        match &self.right_image {
            Some(buf) => Ok(Some(Self::buf_to_gray_image(buf)?)),
            None => Ok(None),
        }
    }

    fn buf_to_gray_image(buf: &[u8]) -> Result<GrayImage, ApiError> {
        let image = image::io::Reader::new(Cursor::new(buf)).with_guessed_format()?.decode()?;
        Ok(image.into_luma8())
    }

    async fn render(&self, config: &LedMatrixConfig) -> Result<(), ApiError> {
        let left_image = self.get_left_image()?;
        let right_image = self.get_right_image()?;

        let left_port = config.left_port.clone();
        let right_port = config.right_port.clone();

        let mut join_set: JoinSet<Result<(), ApiError>> = JoinSet::new();

        if let Some(left_image) = left_image {
            join_set.spawn_blocking(move || {
                Self::render_port(left_port, left_image)
            });
        }

        if let Some(right_image) = right_image {
            join_set.spawn_blocking(move || {
                Self::render_port(right_port, right_image)
            });
        }

        while let Some(result) = join_set.join_next().await {
            result??;
        }

        Ok(())
    }

    fn render_port(port: Arc<Mutex<Port>>, image: GrayImage) -> Result<(), ApiError> {
        let mut port = port.lock().unwrap();
        port.display_gray_image(&image)?;
        Ok(())
    }
}

#[post("/render/base64")]
pub async fn render_base64(render_request: web::Json<SingleRenderRequest>, state: web::Data<AppState>) -> Result<String, ApiError> {
    render_request.render(&state.config).await?;

    info!(?state, "Rendering base64 images");

    format!("Request number: {render_request:?}");

    Ok("".to_string())
}

