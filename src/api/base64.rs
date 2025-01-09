use std::io::Cursor;

use actix_web::{post, web};
use image::GrayImage;
use serde::Deserialize;
use serde_with::base64::Base64;
use serde_with::serde_as;

use crate::api::error::ApiError;
use crate::api::{AppState, RenderResponse, RenderTask};

#[serde_as]
#[derive(Deserialize, Debug)]
struct SingleRenderRequest {
    #[serde(default)]
    #[serde_as(as = "Base64")]
    left_image: Option<Vec<u8>>,
    #[serde(default)]
    #[serde_as(as = "Base64")]
    right_image: Option<Vec<u8>>,
}

#[derive(Deserialize, Debug)]
struct MultipleRenderRequest {
    render: Vec<SingleRenderRequest>,
}

impl SingleRenderRequest {
    fn buf_to_gray_image(buf: &[u8]) -> Result<GrayImage, ApiError> {
        let image = image::ImageReader::new(Cursor::new(buf))
            .with_guessed_format()?
            .decode()?;
        Ok(image.into_luma8())
    }
}

#[post("/render/base64")]
pub async fn render_base64(
    render_request: web::Json<SingleRenderRequest>,
    state: web::Data<AppState>,
) -> Result<web::Json<RenderResponse>, ApiError> {
    let task = prepare_task(render_request.into_inner())?;
    state.sender.send(task).await?;
    Ok(web::Json(RenderResponse {
        queue_len: state.sender.len(),
        queued: true,
    }))
}

#[post("/render/base64/multiple")]
pub async fn render_base64_multiple(
    render_request: web::Json<MultipleRenderRequest>,
    state: web::Data<AppState>,
) -> Result<web::Json<RenderResponse>, ApiError> {
    for request in render_request.into_inner().render {
        let task = prepare_task(request)?;
        state.sender.send(task).await?;
    }

    Ok(web::Json(RenderResponse {
        queue_len: state.sender.len(),
        queued: true,
    }))
}

fn prepare_task(mut render_request: SingleRenderRequest) -> Result<RenderTask, ApiError> {
    let render_task = match (
        render_request.left_image.take(),
        render_request.right_image.take(),
    ) {
        (Some(left), Some(right)) => RenderTask::Both(
            SingleRenderRequest::buf_to_gray_image(&left)?,
            SingleRenderRequest::buf_to_gray_image(&right)?,
        ),
        (Some(left), None) => RenderTask::Left(SingleRenderRequest::buf_to_gray_image(&left)?),
        (None, Some(right)) => RenderTask::Right(SingleRenderRequest::buf_to_gray_image(&right)?),
        (None, None) => {
            return Err(ApiError::BadRequest("No images provided".to_string()));
        }
    };

    Ok(render_task)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deserialize() {
        let request = r#"
        {"left_image":"iVBORw0KGgoAAAANSUhEUgAAAAkAAAAnCAAAAAD4XD2KAAABL0lEQVR4Ae3gAZAkSZIkSRKLqpm7R0REZmZmVlVVVVV3d3d3d/fMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMdHd3d3dXV1VVVVVmZkZGRIS7m5kKz0xmV3d1d3dPz8zMzMxMYuUKqFwBlSv4qcIVX6wCAO/bpgLAa03DqgA8fBpWRwJO9H3f94JZqbXUIiIiIiLqRhwAQCwWAEBsHwMA4sy1AEDc9CAAoD5kGwCoDzkBANSbTgMA9cw1AIC491oAEFdA5QqoXAGVK6ByBVSugMoVULkCKldA5QqoXAGVK6ByBVSugMoVULkCKldA5QqoXAH/CPdDDvUGNwlOAAAAAElFTkSuQmCC"}
        "#;
        assert!(serde_json::from_str::<SingleRenderRequest>(request).is_ok())
    }
}
