use axum::{response::IntoResponse, Json};
use reqwest::StatusCode;
use serde::Serialize;

#[derive(Debug)]
pub enum AptCacheError {
    RepositoryNotFound,
}

#[derive(Serialize)]
struct ResponseError<'a> {
    error: &'a str,
    error_description: &'a str,
}

impl AptCacheError {
    fn response_error(&self) -> (ResponseError, StatusCode) {
        match self {
            Self::RepositoryNotFound => (
                ResponseError {
                    error: "repository_not_found",
                    error_description: "The repository wasn't found",
                },
                StatusCode::NOT_FOUND,
            ),
        }
    }
}

impl IntoResponse for AptCacheError {
    fn into_response(self) -> axum::response::Response {
        let (response_error, status_code) = self.response_error();
        (status_code, Json(response_error)).into_response()
    }
}
