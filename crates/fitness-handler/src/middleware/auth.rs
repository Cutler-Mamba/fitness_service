use fitness_core::{error::AppError, types::JwtClaims};
use jsonwebtoken::{DecodingKey, Validation, decode};
use uuid::Uuid;

pub fn extract_user_id(auth_header: &str, jwt_secret: &str) -> Result<Uuid, AppError> {
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid Authorization header format".into()))?;

    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid or expired token".into()))?;

    Ok(token_data.claims.sub)
}
