use axum::Json;
use axum::extract::FromRequestParts;
use axum::extract::State;
use axum::http::request::Parts;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use tracing::warn;

use crate::error::AppError;
use crate::models::{AppState, Claims, LoginRequest, LoginResponse, User};
use crate::response::ApiResponse;

pub fn encode_jwt(jwt_secret: &str, user: &User) -> Result<String, AppError> {
    let now = chrono::Utc::now();
    let claims = Claims {
        sub: user.id,
        username: user.username.clone(),
        exp: (now + chrono::Duration::hours(24)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("jwt encode: {e}")))
}

pub fn decode_jwt(jwt_secret: &str, token: &str) -> Result<Claims, AppError> {
    let token_data = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;
    Ok(token_data.claims)
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    let row: Option<User> =
        sqlx::query_as("SELECT id, username, password FROM users WHERE username = ?")
            .bind(&body.username)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "test-secret-key";

    fn test_user() -> User {
        User {
            id: 1,
            username: "alice".into(),
            password: "hashed".into(),
        }
    }

    #[test]
    fn jwt_round_trip() {
        let user = test_user();
        let token = encode_jwt(SECRET, &user).unwrap();
        let claims = decode_jwt(SECRET, &token).unwrap();
        assert_eq!(claims.sub, 1);
        assert_eq!(claims.username, "alice");
    }

    #[test]
    fn wrong_secret_is_unauthorized() {
        let user = test_user();
        let token = encode_jwt(SECRET, &user).unwrap();
        let result = decode_jwt("wrong-secret", &token);
        assert!(matches!(result, Err(AppError::Unauthorized)));
    }

    #[test]
    fn expired_token_is_unauthorized() {
        let now = chrono::Utc::now();
        let claims = Claims {
            sub: 1,
            username: "alice".into(),
            exp: (now - chrono::Duration::hours(1)).timestamp() as usize,
            iat: (now - chrono::Duration::hours(2)).timestamp() as usize,
        };
        let token = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(SECRET.as_bytes()),
        )
        .unwrap();
        let result = decode_jwt(SECRET, &token);
        assert!(matches!(result, Err(AppError::Unauthorized)));
    }
}
