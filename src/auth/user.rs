use std::{convert::Infallible, env};

use axum::extract::FromRequestParts;
use axum_extra::extract::CookieJar;
use jwt_simple::{
    claims::Claims,
    prelude::{Duration, HS256Key, MACLike},
};
use password_auth::VerifyError;
use serde::{Deserialize, Serialize};

use crate::{app::AppState, error::AppError, repository::Repository};

const TOKEN_DURATION_MINUTES: u64 = 30;
const MINIMUM_SECRET_LENGTH: usize = 32;

pub struct UnauthenticatedUser {
    username: String,
    password: String,
}

impl UnauthenticatedUser {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    pub async fn authenticate(&self, repository: &Repository) -> Result<User, AppError> {
        let user_record = repository
            .get_user_by_name(&self.username)
            .await?
            .ok_or(AppError::UserDoesNotExist)?;

        match password_auth::verify_password(&self.password, &user_record.password_hash) {
            Ok(()) => Ok(User::new(user_record.id, user_record.username)),

            Err(VerifyError::PasswordInvalid) => Err(AppError::InvalidCredentials),

            Err(error) => {
                tracing::error!(
                    error = ?error,
                    user_id = user_record.id,
                    "stored password hash could not be verified"
                );

                Err(AppError::InvalidPasswordHash)
            }
        }
    }

    pub async fn register(self, repository: &Repository) -> Result<User, AppError> {
        let password_hash = password_auth::generate_hash(self.password);

        let user_record = match repository.add_user(&self.username, &password_hash).await {
            Ok(user_record) => user_record,

            Err(sqlx::Error::Database(db_error)) if db_error.is_unique_violation() => {
                return Err(AppError::UsernameTaken);
            }

            Err(error) => return Err(AppError::Database(error)),
        };

        Ok(User::new(user_record.id, user_record.username))
    }
}

pub struct User {
    id: i64,
    username: String,
}

impl User {
    pub(crate) fn new(id: i64, username: String) -> Self {
        Self { id, username }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub const fn id(&self) -> i64 {
        self.id
    }

    pub fn auth_token(self) -> Result<String, AppError> {
        let secret = jwt_secret()?;
        let key = HS256Key::from_bytes(secret.as_bytes());

        let claims = Claims::with_custom_claims(
            UserClaims::from(self),
            Duration::from_mins(TOKEN_DURATION_MINUTES),
        );

        Ok(key.authenticate(claims)?)
    }

    pub fn from_auth_token(token: &str) -> Result<Self, AppError> {
        let secret = jwt_secret()?;
        let key = HS256Key::from_bytes(secret.as_bytes());

        let claims: UserClaims = key.verify_token(token, None)?.custom;

        Ok(Self::new(claims.id, claims.username))
    }
}

impl FromRequestParts<AppState> for User {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);

        let token = jar
            .get("token")
            .map(|cookie| cookie.value())
            .ok_or(AppError::MissingAuthorization)?;

        Self::from_auth_token(token)
    }
}

impl FromRequestParts<AppState> for Option<User> {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(User::from_request_parts(parts, state).await.ok())
    }
}

#[derive(Serialize, Deserialize)]
struct UserClaims {
    id: i64,
    username: String,
}

impl From<User> for UserClaims {
    fn from(User { id, username }: User) -> Self {
        Self { id, username }
    }
}

fn jwt_secret() -> Result<String, AppError> {
    let secret = env::var("JWT_SECRET").map_err(|_| AppError::Configuration("JWT_SECRET"))?;

    if secret.len() < MINIMUM_SECRET_LENGTH {
        tracing::error!(
            configured_length = secret.len(),
            minimum_length = MINIMUM_SECRET_LENGTH,
            "JWT_SECRET is too short"
        );

        return Err(AppError::Configuration("JWT_SECRET"));
    }

    Ok(secret)
}
