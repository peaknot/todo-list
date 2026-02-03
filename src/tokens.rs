use crate::structs::User;
use argon2::{Argon2, PasswordVerifier};
use axum::{
    Json,
    extract::{Request, State},
    http::{
        StatusCode,
        header::{AUTHORIZATION, HeaderMap},
    },
    middleware::Next,
    response::IntoResponse,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

pub const JWT_SECRET: &str = "my_super_secret_key_12345";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Claims {
    pub sub: String, // subject
    pub exp: usize,  // expiration
    pub iat: usize,  //issued at
}

#[derive(Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

pub async fn login(
    State(pool): State<SqlitePool>,
    Json(payload): Json<LoginPayload>,
) -> impl IntoResponse {
    let data = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
        .bind(&payload.username)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            println!("DB Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        });

    let verified_user = match data {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid credentials"})),
            );
        }
        Err(code) => return (code, Json(serde_json::json!({"error": "Database error"}))),
    };

    let parsed_hash = match argon2::PasswordHash::new(&verified_user.password) {
        Ok(hash) => hash,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Stored password hash is invalid"})),
            );
        }
    };

    if Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid credentials"})),
        );
    }
    let issued_at = Utc::now();
    let expiration = issued_at + Duration::hours(1);

    let claims = Claims {
        sub: verified_user.id.to_string(),
        exp: expiration.timestamp() as usize,
        iat: issued_at.timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .expect("Token creation failed.");

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "token": token,
            "type": "Bearer"
        })),
    )
}

pub async fn authorize(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let auth_header = headers
        .get(AUTHORIZATION)
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Missing Authorization Header".to_string(),
        ))?
        .to_str()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid Header Value".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid Token Format".to_string()))?;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid Token: {}", e)))?;

    request.extensions_mut().insert(token_data.claims);

    Ok(next.run(request).await)
}
