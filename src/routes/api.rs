use axum::{
    Json, Router,
    extract::Path,
    http::StatusCode,
    routing::{delete, get},
};
use serde::Deserialize;

use crate::{
    app::AppState,
    auth::admin::Admin,
    error::AppError,
    models::{Asset, PortfolioSummary},
    repository::Repository,
};

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
async fn list_assets(repository: Repository) -> Result<Json<Vec<Asset>>, AppError> {
    let assets = repository.list_assets().await?;

    Ok(Json(assets))
}

#[tracing::instrument(skip_all)]
async fn portfolio_summary(repository: Repository) -> Result<Json<PortfolioSummary>, AppError> {
    let assets = repository.list_assets().await?;
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
    _: Admin,
    repository: Repository,
    Json(request): Json<CreateAssetRequest>,
) -> Result<Json<Asset>, AppError> {
    let new_asset = repository
        .create_asset(request.name, request.unit_value, request.quantity)
        .await?;

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
    _: Admin,
    repository: Repository,
    Json(request): Json<UpdateAssetRequest>,
) -> Result<Json<Asset>, AppError> {
    match repository
        .update_asset(
            request.id,
            request.name,
            request.unit_value,
            request.quantity,
        )
        .await?
    {
        Some(updated_asset) => Ok(Json(updated_asset)),
        None => Err(AppError::AssetDoesNotExist),
    }
}

#[tracing::instrument(skip_all)]
async fn delete_asset(
    _: Admin,
    repository: Repository,
    Path(asset_id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let deleted = repository.delete_asset(asset_id).await?;

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

    #[sqlx::test]
    async fn test_create_asset(db: PgPool) {
        let request = CreateAssetRequest {
            name: "Bitcoin".to_string(),
            unit_value: 10.0,
            quantity: 2.0,
        };

        let Json(new_asset) = create_asset(Admin, db.into(), Json(request))
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
        let Json(assets) = list_assets(db.into()).await.expect("success");

        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].name, "Bitcoin");
        assert_eq!(assets[0].quantity, 0.0);

        insta::assert_json_snapshot!(assets);
    }

    #[sqlx::test(fixtures("bitcoin_asset"))]
    async fn test_update_asset(db: PgPool) {
        let request = UpdateAssetRequest {
            id: 1,
            name: Some("Ethereum".to_string()),
            unit_value: Some(20.0),
            quantity: Some(3.0),
        };

        let Json(updated_asset) = update_asset(Admin, db.into(), Json(request))
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
        let status = delete_asset(Admin, db.clone().into(), Path(1))
            .await
            .expect("success");

        assert_eq!(status, StatusCode::NO_CONTENT);

        let Json(assets) = list_assets(db.into()).await.expect("success");

        assert!(assets.is_empty());
    }

    #[sqlx::test(fixtures("bitcoin_asset"))]
    async fn test_portfolio_summary(db: PgPool) {
        let Json(summary) = portfolio_summary(db.into()).await.expect("success");

        assert_eq!(summary.total_assets, 1);
        assert_eq!(summary.portfolio_value, 0.0);

        insta::assert_json_snapshot!(summary);
    }
}
