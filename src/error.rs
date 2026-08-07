use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("missing authorization")]
    MissingAuthorization,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("asset does not exist")]
    AssetDoesNotExist,

    #[error("asset data is invalid: {0}")]
    InvalidAssetData(&'static str),

    #[error("asset name is already registered")]
    AssetNameTaken,

    #[error("user does not exist")]
    UserDoesNotExist,

    #[error("username is already registered")]
    UsernameTaken,

    #[error("invalid password hash")]
    InvalidPasswordHash,

    #[error("application configuration is missing: {0}")]
    Configuration(&'static str),

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Template(#[from] askama::Error),

    #[error(transparent)]
    Jwt(#[from] jwt_simple::Error),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, public_message, should_log) = match &self {
            Self::MissingAuthorization => {
                (StatusCode::UNAUTHORIZED, "Autenticação necessária.", false)
            }

            Self::InvalidCredentials | Self::Jwt(_) => (
                StatusCode::UNAUTHORIZED,
                "Sessão inválida, expirada ou credenciais incorretas.",
                false,
            ),

            Self::AssetDoesNotExist => (
                StatusCode::NOT_FOUND,
                "O ativo solicitado não foi encontrado.",
                false,
            ),

            Self::InvalidAssetData(message) => (StatusCode::UNPROCESSABLE_ENTITY, *message, false),

            Self::AssetNameTaken => (
                StatusCode::CONFLICT,
                "Já existe um ativo cadastrado com esse nome.",
                false,
            ),

            Self::UserDoesNotExist => (
                StatusCode::NOT_FOUND,
                "O usuário solicitado não foi encontrado.",
                false,
            ),

            Self::UsernameTaken => (
                StatusCode::CONFLICT,
                "Este nome de usuário já está em uso.",
                false,
            ),

            Self::InvalidPasswordHash
            | Self::Configuration(_)
            | Self::Database(_)
            | Self::Template(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Ocorreu um erro interno. Tente novamente.",
                true,
            ),
        };

        if should_log {
            tracing::error!(error = ?self, "internal application error");
        }

        (
            status,
            Json(ErrorResponse {
                error: public_message,
            }),
        )
            .into_response()
    }
}
