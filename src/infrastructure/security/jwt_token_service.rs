use std::time::Duration;

use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::application::ports::TokenService;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Clone)]
pub struct JwtTokenService {
    secret: Vec<u8>,
    duration: Duration,
}

impl JwtTokenService {
    pub fn new(secret: String, duration_hours: u64) -> Self {
        Self {
            secret: secret.into_bytes(),
            duration: Duration::from_secs(duration_hours * 3600),
        }
    }
}

impl TokenService for JwtTokenService {
    fn generate_token(&self, user_id: Uuid) -> Result<String, ApplicationError> {
        let now = Utc::now();
        let iat = now.timestamp() as usize;
        let exp = (now
            + chrono::Duration::from_std(self.duration).map_err(|e| {
                ApplicationError::InternalError(format!("Duration conversion error: {e}"))
            })?)
        .timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            exp,
            iat,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&self.secret),
        )
        .map_err(|e| ApplicationError::InternalError(format!("Failed to create token: {e}")))
    }

    fn verify_token(&self, token: &str) -> Result<Uuid, ApplicationError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(&self.secret),
            &Validation::default(),
        )
        .map_err(|_| {
            ApplicationError::Unauthorized("Token tidak valid atau telah kedaluwarsa".to_string())
        })?;

        Uuid::parse_str(&token_data.claims.sub).map_err(|_| {
            ApplicationError::Unauthorized("ID pengguna dalam token tidak valid".to_string())
        })
    }
}
