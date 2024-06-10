use actix_multipart::Multipart;
use actix_web::web::{BytesMut, Json};
use actix_web::{post, web};
use anyhow::anyhow;
use futures_util::{StreamExt, TryStreamExt};
use image::GrayImage;
use kanal::AsyncSender;

use crate::api::error::ApiError;
use crate::api::{AppState, RenderResponse, RenderTask};

#[post("/render/files")]
pub async fn render_files(
    mut payload: Multipart,
    state: web::Data<AppState>,
) -> Result<Json<RenderResponse>, ApiError> {
    let mut images = vec![];

    while let Some(mut field) = payload
        .try_next()
        .await
        .map_err(|err| anyhow!("Multipart error: {err:?}"))?
    {
        let mut file_data = BytesMut::new();
        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(|err| anyhow!("Multipart error: {err:?}"))?;
            file_data.extend_from_slice(&data);
        }

        let image = image::load_from_memory(&file_data)?.into_luma8();
        images.push(image);
    }

    match (
        state.config.left_port.as_ref(),
        state.config.right_port.as_ref(),
    ) {
        (Some(_), Some(_)) => {
            queue_even_odd(images, &state.sender).await?;
        }
        (Some(_), None) => {
            for image in images {
                state.sender.send(RenderTask::Left(image)).await?;
            }
        }
        (None, Some(_)) => {
            for image in images {
                state.sender.send(RenderTask::Right(image)).await?;
            }
        }
        (None, None) => {
            return Err(ApiError::InternalError(anyhow!("No ports configured")));
        }
    }

    Ok(Json(RenderResponse {
        queue_len: state.sender.len(),
        queued: true,
    }))
}

async fn queue_even_odd(
    images: Vec<GrayImage>,
    sender: &AsyncSender<RenderTask>,
) -> anyhow::Result<()> {
    let mut iter = images.into_iter().array_chunks::<2>();

    for [left, right] in iter.by_ref() {
        sender.send(RenderTask::Both(left, right)).await?;
    }

    if let Some(mut rem) = iter.into_remainder()
        && let Some(left) = rem.next()
    {
        sender.send(RenderTask::Left(left)).await?;
    }

    Ok(())
}
