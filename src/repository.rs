use std::convert::Infallible;

use axum::extract::FromRequestParts;
use sqlx::PgPool;

use crate::{
    app::AppState,
    models::{Asset, UserRecord},
};

pub struct Repository {
    db: PgPool,
}

impl Repository {
    pub async fn list_assets(&self) -> sqlx::Result<Vec<Asset>> {
        sqlx::query_as!(
            Asset,
            r#"
            SELECT
                id,
                name,
                unit_value,
                quantity
            FROM assets
            ORDER BY id;
            "#
        )
        .fetch_all(&self.db)
        .await
    }

    pub async fn asset_name_exists(
        &self,
        name: &str,
        exclude_id: Option<i64>,
    ) -> sqlx::Result<bool> {
        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM assets
                WHERE LOWER(name) = LOWER($1)
                  AND ($2::BIGINT IS NULL OR id <> $2)
            ) AS "exists!"
            "#,
            name,
            exclude_id
        )
        .fetch_one(&self.db)
        .await?;

        Ok(exists)
    }

    pub async fn create_asset(
        &self,
        name: String,
        unit_value: f64,
        quantity: f64,
    ) -> sqlx::Result<Asset> {
        sqlx::query_as!(
            Asset,
            r#"
            INSERT INTO assets (
                name,
                unit_value,
                quantity
            )
            VALUES ($1, $2, $3)
            RETURNING
                id,
                name,
                unit_value,
                quantity;
            "#,
            name,
            unit_value,
            quantity
        )
        .fetch_one(&self.db)
        .await
    }

    pub async fn update_asset(
        &self,
        asset_id: i64,
        name: Option<String>,
        unit_value: Option<f64>,
        quantity: Option<f64>,
    ) -> sqlx::Result<Option<Asset>> {
        sqlx::query_as!(
            Asset,
            r#"
            UPDATE assets
            SET
                name = COALESCE($2, name),
                unit_value = COALESCE($3, unit_value),
                quantity = COALESCE($4, quantity)
            WHERE id = $1
            RETURNING
                id,
                name,
                unit_value,
                quantity;
            "#,
            asset_id,
            name,
            unit_value,
            quantity
        )
        .fetch_optional(&self.db)
        .await
    }

    pub async fn delete_asset(&self, asset_id: i64) -> sqlx::Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM assets
            WHERE id = $1;
            "#,
            asset_id
        )
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn add_user(&self, username: &str, password_hash: &str) -> sqlx::Result<UserRecord> {
        sqlx::query_as!(
            UserRecord,
            r#"
            INSERT INTO users (
                username,
                password_hash
            )
            VALUES ($1, $2)
            RETURNING
                id,
                username,
                password_hash;
            "#,
            username,
            password_hash
        )
        .fetch_one(&self.db)
        .await
    }

    pub async fn get_user_by_name(&self, username: &str) -> sqlx::Result<Option<UserRecord>> {
        sqlx::query_as!(
            UserRecord,
            r#"
            SELECT
                id,
                username,
                password_hash
            FROM users
            WHERE username = $1;
            "#,
            username
        )
        .fetch_optional(&self.db)
        .await
    }
}

impl FromRequestParts<AppState> for Repository {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self {
            db: state.db.clone(),
        })
    }
}

#[cfg(test)]
impl From<PgPool> for Repository {
    fn from(db: PgPool) -> Self {
        Self { db }
    }
}
