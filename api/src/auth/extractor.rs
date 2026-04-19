use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::auth::jwt::decode_jwt;
use crate::auth::models::Claims;
use crate::core::AppState;
use crate::core::error::AppError;

impl FromRequestParts<AppState> for Claims {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized)?;

        decode_jwt(&state.jwt_secret, token)
    }
}
