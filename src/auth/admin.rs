use std::env;

use axum::{extract::FromRequestParts, http::header::AUTHORIZATION};

use crate::{app::AppState, error::AppError};

const MINIMUM_SECRET_LENGTH: usize = 32;

pub struct Admin;

impl FromRequestParts<AppState> for Admin {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or(AppError::MissingAuthorization)?;

        let authorization = header.to_str().map_err(|_| AppError::InvalidCredentials)?;

        let provided_secret = authorization
            .strip_prefix("Bearer ")
            .unwrap_or(authorization);

        let expected_secret = admin_secret()?;

        if provided_secret == expected_secret {
            Ok(Admin)
        } else {
            Err(AppError::InvalidCredentials)
        }
    }
}

fn admin_secret() -> Result<String, AppError> {
    let secret = env::var("ADMIN_SECRET").map_err(|_| AppError::Configuration("ADMIN_SECRET"))?;

    if secret.len() < MINIMUM_SECRET_LENGTH {
        tracing::error!(
            configured_length = secret.len(),
            minimum_length = MINIMUM_SECRET_LENGTH,
            "ADMIN_SECRET is too short"
        );

        return Err(AppError::Configuration("ADMIN_SECRET"));
    }

    Ok(secret)
}
