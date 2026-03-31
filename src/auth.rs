use std::env;
use anyhow::{Context, Result};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // 用戶 ID
    pub exp: usize,  // 過期時間
    pub role: String, // 角色
}

#[derive(Debug, Serialize, Deserialize,ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize,ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub expires_in: usize,
}

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtService {
    pub fn new() -> Result<Self> {
        let secret = env::var("JWT_SECRET")
            .context("請設定 JWT_SECRET 環境變數")?;

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        Ok(Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            validation,
        })
    }

    pub fn generate_token(&self, user_id: &str) -> Result<String> {
        let exp = Utc::now() + Duration::hours(24);

        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp.timestamp() as usize,
            role: "admin".to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .context("JWT token 生成失敗")
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .context("JWT token 驗證失敗")?;

        Ok(token_data.claims)
    }
}