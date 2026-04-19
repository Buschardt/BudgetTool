use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};

use crate::auth::models::{Claims, User};
use crate::core::error::AppError;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::models::User;

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
