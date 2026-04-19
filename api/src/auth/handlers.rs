use axum::Json;
use axum::extract::State;
use tracing::warn;

use crate::auth::jwt::encode_jwt;
use crate::auth::models::{Claims, LoginRequest, LoginResponse, User};
use crate::core::AppState;
use crate::core::error::AppError;
use crate::core::response::ApiResponse;

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    let row: Option<User> =
        sqlx::query_as("SELECT id, username, password FROM users WHERE username = ?")
            .bind(&body.username)
            .fetch_optional(&state.db)
            .await?;

    let user = row.ok_or(AppError::Unauthorized)?;

    let valid = bcrypt::verify(&body.password, &user.password)
        .map_err(|e| AppError::Internal(format!("bcrypt: {e}")))?;

    if !valid {
        warn!(username = %body.username, "failed login attempt");
        return Err(AppError::Unauthorized);
    }

    let token = encode_jwt(&state.jwt_secret, &user)?;
    Ok(ApiResponse::success(LoginResponse { token }))
}

pub async fn me(claims: Claims) -> Json<ApiResponse<Claims>> {
    ApiResponse::success(claims)
}
