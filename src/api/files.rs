use actix_multipart::Multipart;
use actix_web::{Error, HttpResponse, post, web};
use actix_web::web::{BytesMut, Json};
use anyhow::anyhow;
use futures_util::{StreamExt, TryStreamExt};
use tokio::time::Instant;
use tracing::info;

use crate::api::{AppState, RenderResponse};
use crate::api::error::ApiError;

#[post("/render/files")]
pub async fn render_files(
    mut payload: Multipart,
    state: web::Data<AppState>,
) -> Result<Json<RenderResponse>, ApiError> {
    let ports = [
        state.config.left_port.clone(),
        state.config.right_port.clone(),
    ];

    let mut index = 0;

    let start = Instant::now();

    while let Some(mut field) = payload
        .try_next()
        .await
        .map_err(|err| anyhow!("Multipart error: {err:?}"))?
    {
        let content_disposition = field.content_disposition();

        let filename = content_disposition.get_filename();
        info!(?filename, "Processing file");

        let mut file_data = BytesMut::new();
        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(|err| anyhow!("Multipart error: {err:?}"))?;
            file_data.extend_from_slice(&data);
        }

        // Create an in-memory dynamic image from the bytes
        let image = image::load_from_memory(&file_data)?.into_luma8();

        let port = ports[index % 2].clone();
        let mut port = port
            .lock()
            .map_err(|err| anyhow!("Poison error: {err:?}"))?;

        port.display_gray_image(&image)?;

        index += 1;
    }

    let elapsed = start.elapsed();

    Ok(Json(RenderResponse {
        elapsed_ms: elapsed.as_millis(),
        success: true,
    }))
}
