use actix_multipart::Multipart;
use actix_web::{post, web};
use actix_web::web::{BytesMut, Json};
use anyhow::anyhow;
use futures_util::{StreamExt, TryStreamExt};

use crate::api::{AppState, RenderResponse, RenderTask};
use crate::api::error::ApiError;

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

    let mut iter = images.into_iter().array_chunks::<2>();

    while let Some([left, right]) = iter.next() {
        state.sender.send(RenderTask::Both(left, right)).await?;
    }

    if let Some(mut rem) = iter.into_remainder() && let Some(left) = rem.next() {
        state.sender.send(RenderTask::Left(left)).await?;
    }

    Ok(Json(RenderResponse {
        queue_len: state.sender.len(),
        success: true,
    }))
}
