use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("hledger error (exit {exit_code}): {stderr}")]
    HledgerCommand { exit_code: i32, stderr: String },

    #[error("failed to parse hledger output: {0}")]
    HledgerParse(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("not found: {0}")]
    NotFound(String),

    #[error("payload too large")]
    PayloadTooLarge,

    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = json!({ "ok": false, "error": self.to_string() });
        (status, axum::Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::response::IntoResponse;

    async fn body_json(resp: Response) -> serde_json::Value {
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn hledger_command_error_is_500() {
        let err = AppError::HledgerCommand {
            exit_code: 1,
            stderr: "no such file".into(),
        };
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let json = body_json(resp).await;
        assert_eq!(json["ok"], false);
        assert!(json["error"].as_str().unwrap().contains("hledger error"));
    }

    #[tokio::test]
    async fn bad_request_is_400() {
        let err = AppError::BadRequest("missing param".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let json = body_json(resp).await;
        assert_eq!(json["ok"], false);
        assert!(json["error"].as_str().unwrap().contains("bad request"));
    }

    #[tokio::test]
    async fn unauthorized_is_401() {
        let err = AppError::Unauthorized;
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let json = body_json(resp).await;
        assert_eq!(json["ok"], false);
    }

    #[tokio::test]
    async fn internal_is_500() {
        let err = AppError::Internal("something broke".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let json = body_json(resp).await;
        assert_eq!(json["ok"], false);
    }

    #[tokio::test]
    async fn parse_error_is_500() {
        let raw = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let err = AppError::HledgerParse(raw);
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let json = body_json(resp).await;
        assert_eq!(json["ok"], false);
    }
}
