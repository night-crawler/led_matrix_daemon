use std::io::Cursor;

use actix_web::{post, web};
use image::GrayImage;
use serde::Deserialize;
use serde_with::base64::Base64;
use serde_with::serde_as;

use crate::api::{AppState, RenderResponse, RenderTask};
use crate::api::error::ApiError;

#[serde_as]
#[derive(Deserialize, Debug)]
struct SingleRenderRequest {
    #[serde_as(as = "Base64")]
    left_image: Option<Vec<u8>>,
    #[serde_as(as = "Base64")]
    right_image: Option<Vec<u8>>,
}

impl SingleRenderRequest {
    fn buf_to_gray_image(buf: &[u8]) -> Result<GrayImage, ApiError> {
        let image = image::io::Reader::new(Cursor::new(buf))
            .with_guessed_format()?
            .decode()?;
        Ok(image.into_luma8())
    }
}

#[post("/render/base64")]
pub async fn render_base64(
    mut render_request: web::Json<SingleRenderRequest>,
    state: web::Data<AppState>,
) -> Result<web::Json<RenderResponse>, ApiError> {
    match (render_request.left_image.take(), render_request.right_image.take()) {
        (Some(left), Some(right)) => {
            state.sender.send(RenderTask::Both(
                SingleRenderRequest::buf_to_gray_image(&left)?,
                SingleRenderRequest::buf_to_gray_image(&right)?,
            )).await?;
        },
        (Some(left), None) => {
            state.sender.send(RenderTask::Left(
                SingleRenderRequest::buf_to_gray_image(&left)?,
            )).await?;
        },
        (None, Some(right)) => {
            state.sender.send(RenderTask::Right(
                SingleRenderRequest::buf_to_gray_image(&right)?,
            )).await?;
        },
        (None, None) => {
            return Err(ApiError::BadRequest("No images provided".to_string()));
        },
    }

    Ok(web::Json(RenderResponse {
        queue_len: state.sender.len(),
        success: true,
    }))
}
