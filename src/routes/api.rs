use axum::{
    Json, Router,
    extract::Path,
    http::StatusCode,
    routing::{delete, get},
};
use serde::Deserialize;

use crate::{
    app::AppState,
    auth::user::User,
    error::AppError,
    models::{Asset, PortfolioSummary},
    repository::Repository,
};

const MAX_ASSET_NAME_LENGTH: usize = 80;

fn validate_asset_name(name: String) -> Result<String, AppError> {
    let normalized = name.trim().to_string();
    let length = normalized.chars().count();

    if normalized.is_empty() {
        return Err(AppError::InvalidAssetData("Informe o nome do ativo."));
    }

    if length > MAX_ASSET_NAME_LENGTH {
        return Err(AppError::InvalidAssetData(
            "O nome do ativo deve possuir no máximo 80 caracteres.",
        ));
    }

    Ok(normalized)
}

fn validate_positive_number(value: f64, message: &'static str) -> Result<f64, AppError> {
    if !value.is_finite() || value <= 0.0 {
        return Err(AppError::InvalidAssetData(message));
    }

    Ok(value)
}

fn map_asset_database_error(error: sqlx::Error) -> AppError {
    match error {
        sqlx::Error::Database(ref database_error) if database_error.is_unique_violation() => {
            AppError::AssetNameTaken
        }
        other => AppError::Database(other),
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/assets",
            get(list_assets).post(create_asset).patch(update_asset),
        )
        .route("/assets/{id}", delete(delete_asset))
        .route("/portfolio/summary", get(portfolio_summary))
}

#[tracing::instrument(skip_all)]
async fn list_assets(user: User, repository: Repository) -> Result<Json<Vec<Asset>>, AppError> {
    let assets = repository.list_assets(user.id()).await?;

    Ok(Json(assets))
}

#[tracing::instrument(skip_all)]
async fn portfolio_summary(
    user: User,
    repository: Repository,
) -> Result<Json<PortfolioSummary>, AppError> {
    let assets = repository.list_assets(user.id()).await?;
    let summary = PortfolioSummary::from_assets(&assets);

    Ok(Json(summary))
}

#[derive(Deserialize)]
struct CreateAssetRequest {
    name: String,
    unit_value: f64,
    quantity: f64,
}

#[tracing::instrument(skip_all)]
async fn create_asset(
    user: User,
    repository: Repository,
    Json(request): Json<CreateAssetRequest>,
) -> Result<Json<Asset>, AppError> {
    let name = validate_asset_name(request.name)?;
    let unit_value = validate_positive_number(
        request.unit_value,
        "O valor unitário deve ser maior que zero.",
    )?;
    let quantity =
        validate_positive_number(request.quantity, "A quantidade deve ser maior que zero.")?;

    if repository.asset_name_exists(user.id(), &name, None).await? {
        return Err(AppError::AssetNameTaken);
    }

    let new_asset = repository
        .create_asset(user.id(), name, unit_value, quantity)
        .await
        .map_err(map_asset_database_error)?;

    Ok(Json(new_asset))
}

#[derive(Deserialize)]
struct UpdateAssetRequest {
    id: i64,
    name: Option<String>,
    unit_value: Option<f64>,
    quantity: Option<f64>,
}

#[tracing::instrument(skip_all)]
async fn update_asset(
    user: User,
    repository: Repository,
    Json(request): Json<UpdateAssetRequest>,
) -> Result<Json<Asset>, AppError> {
    if request.id <= 0 {
        return Err(AppError::InvalidAssetData(
            "O identificador do ativo é inválido.",
        ));
    }

    if request.name.is_none() && request.unit_value.is_none() && request.quantity.is_none() {
        return Err(AppError::InvalidAssetData(
            "Informe ao menos um campo para atualização.",
        ));
    }

    let name = match request.name {
        Some(name) => Some(validate_asset_name(name)?),
        None => None,
    };

    let unit_value = match request.unit_value {
        Some(value) => Some(validate_positive_number(
            value,
            "O valor unitário deve ser maior que zero.",
        )?),
        None => None,
    };

    let quantity = match request.quantity {
        Some(value) => Some(validate_positive_number(
            value,
            "A quantidade deve ser maior que zero.",
        )?),
        None => None,
    };

    if let Some(ref asset_name) = name {
        if repository
            .asset_name_exists(user.id(), asset_name, Some(request.id))
            .await?
        {
            return Err(AppError::AssetNameTaken);
        }
    }

    match repository
        .update_asset(user.id(), request.id, name, unit_value, quantity)
        .await
        .map_err(map_asset_database_error)?
    {
        Some(updated_asset) => Ok(Json(updated_asset)),
        None => Err(AppError::AssetDoesNotExist),
    }
}

#[tracing::instrument(skip_all)]
async fn delete_asset(
    user: User,
    repository: Repository,
    Path(asset_id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let deleted = repository.delete_asset(user.id(), asset_id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::AssetDoesNotExist)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    async fn create_test_user(repository: &Repository, username: &str) -> User {
        let record = repository
            .add_user(username, "test-password-hash")
            .await
            .expect("create test user");

        User::new(record.id, record.username)
    }

    async fn fixture_user(repository: &Repository) -> User {
        let record = repository
            .get_user_by_name("fixture_user")
            .await
            .expect("query fixture user")
            .expect("fixture user exists");

        User::new(record.id, record.username)
    }

    #[sqlx::test]
    async fn test_create_asset(db: PgPool) {
        let repository: Repository = db.into();
        let user = create_test_user(&repository, "create_user").await;

        let request = CreateAssetRequest {
            name: "Bitcoin".to_string(),
            unit_value: 10.0,
            quantity: 2.0,
        };

        let Json(new_asset) = create_asset(user, repository, Json(request))
            .await
            .expect("success");

        assert_eq!(new_asset.id, 1);
        assert_eq!(new_asset.name, "Bitcoin");
        assert_eq!(new_asset.unit_value, 10.0);
        assert_eq!(new_asset.quantity, 2.0);

        insta::assert_json_snapshot!(new_asset);
    }

    #[sqlx::test(fixtures("bitcoin_asset"))]
    async fn test_list_assets(db: PgPool) {
        let repository: Repository = db.into();
        let user = fixture_user(&repository).await;

        let Json(assets) = list_assets(user, repository).await.expect("success");

        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].name, "Bitcoin");
        assert_eq!(assets[0].quantity, 0.0);

        insta::assert_json_snapshot!(assets);
    }

    #[sqlx::test(fixtures("bitcoin_asset"))]
    async fn test_update_asset(db: PgPool) {
        let repository: Repository = db.into();
        let user = fixture_user(&repository).await;

        let request = UpdateAssetRequest {
            id: 1,
            name: Some("Ethereum".to_string()),
            unit_value: Some(20.0),
            quantity: Some(3.0),
        };

        let Json(updated_asset) = update_asset(user, repository, Json(request))
            .await
            .expect("success");

        assert_eq!(updated_asset.id, 1);
        assert_eq!(updated_asset.name, "Ethereum");
        assert_eq!(updated_asset.unit_value, 20.0);
        assert_eq!(updated_asset.quantity, 3.0);

        insta::assert_json_snapshot!(updated_asset);
    }

    #[sqlx::test(fixtures("bitcoin_asset"))]
    async fn test_delete_asset(db: PgPool) {
        let repository: Repository = db.clone().into();
        let user = fixture_user(&repository).await;
        let user_id = user.id();

        let status = delete_asset(user, repository, Path(1))
            .await
            .expect("success");

        assert_eq!(status, StatusCode::NO_CONTENT);

        let repository: Repository = db.into();
        let assets = repository.list_assets(user_id).await.expect("list assets");

        assert!(assets.is_empty());
    }

    #[sqlx::test(fixtures("bitcoin_asset"))]
    async fn test_portfolio_summary(db: PgPool) {
        let repository: Repository = db.into();
        let user = fixture_user(&repository).await;

        let Json(summary) = portfolio_summary(user, repository).await.expect("success");

        assert_eq!(summary.total_assets, 1);
        assert_eq!(summary.portfolio_value, 0.0);

        insta::assert_json_snapshot!(summary);
    }

    #[sqlx::test]
    async fn test_assets_are_isolated_between_users(db: PgPool) {
        let repository: Repository = db.into();

        let first_user = create_test_user(&repository, "first_user").await;
        let second_user = create_test_user(&repository, "second_user").await;

        repository
            .create_asset(first_user.id(), "Bitcoin".to_string(), 10.0, 1.0)
            .await
            .expect("create first asset");

        repository
            .create_asset(second_user.id(), "Bitcoin".to_string(), 20.0, 2.0)
            .await
            .expect("same asset name is allowed for another user");

        let first_assets = repository
            .list_assets(first_user.id())
            .await
            .expect("list first user assets");

        let second_assets = repository
            .list_assets(second_user.id())
            .await
            .expect("list second user assets");

        assert_eq!(first_assets.len(), 1);
        assert_eq!(second_assets.len(), 1);
        assert_eq!(first_assets[0].unit_value, 10.0);
        assert_eq!(second_assets[0].unit_value, 20.0);
    }

    #[sqlx::test]
    async fn test_user_cannot_change_another_users_asset(db: PgPool) {
        let repository: Repository = db.into();

        let owner = create_test_user(&repository, "owner").await;
        let intruder = create_test_user(&repository, "intruder").await;

        let asset = repository
            .create_asset(owner.id(), "Ethereum".to_string(), 30.0, 3.0)
            .await
            .expect("create owner asset");

        let update_result = repository
            .update_asset(
                intruder.id(),
                asset.id,
                Some("Alterado".to_string()),
                Some(999.0),
                Some(999.0),
            )
            .await
            .expect("attempt update");

        let delete_result = repository
            .delete_asset(intruder.id(), asset.id)
            .await
            .expect("attempt delete");

        assert!(update_result.is_none());
        assert!(!delete_result);

        let owner_assets = repository
            .list_assets(owner.id())
            .await
            .expect("list owner assets");

        assert_eq!(owner_assets.len(), 1);
        assert_eq!(owner_assets[0].name, "Ethereum");
        assert_eq!(owner_assets[0].unit_value, 30.0);
        assert_eq!(owner_assets[0].quantity, 3.0);
    }
    #[sqlx::test]
    async fn test_rejects_invalid_asset_values(db: PgPool) {
        let repository: Repository = db.into();
        let user = create_test_user(&repository, "invalid_values_user").await;

        let request = CreateAssetRequest {
            name: "Ativo inválido".to_string(),
            unit_value: f64::NAN,
            quantity: 1.0,
        };

        let result = create_asset(user, repository, Json(request)).await;

        assert!(matches!(result, Err(AppError::InvalidAssetData(_))));
    }

    #[sqlx::test]
    async fn test_rejects_duplicate_asset_name_for_same_user(db: PgPool) {
        let repository: Repository = db.into();
        let user = create_test_user(&repository, "duplicate_name_user").await;
        let user_id = user.id();

        repository
            .create_asset(user_id, "Bitcoin".to_string(), 10.0, 1.0)
            .await
            .expect("create first asset");

        let request = CreateAssetRequest {
            name: "  bitcoin  ".to_string(),
            unit_value: 20.0,
            quantity: 2.0,
        };

        let result = create_asset(
            User::new(user_id, "duplicate_name_user".to_string()),
            repository,
            Json(request),
        )
        .await;

        assert!(matches!(result, Err(AppError::AssetNameTaken)));
    }
}
