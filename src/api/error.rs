use actix_web::{HttpResponse, ResponseError};
use actix_web::http::StatusCode;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Internal error: {0}")]
    InternalError(#[from] anyhow::Error),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("Join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

struct ErrorMessage {
    message: String,
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match &self {
            Self::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,

            // treat IoError as BadRequest
            ApiError::IoError(_) => StatusCode::BAD_REQUEST,
            ApiError::ImageError(_) => StatusCode::BAD_REQUEST,
            ApiError::JoinError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}