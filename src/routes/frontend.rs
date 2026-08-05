use std::env;

use askama::Template;
use axum::{
    Form, Router,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use serde::Deserialize;

use crate::{
    app::AppState,
    auth::user::{UnauthenticatedUser, User},
    error::AppError,
    models::PortfolioSummary,
    repository::Repository,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/login", get(login_page).post(login))
        .route("/register", get(register_page).post(register))
        .route("/logout", post(logout))
        .route("/assets/create", post(create_asset_from_form))
        .route("/assets/update", post(update_asset_from_form))
        .route("/assets/delete", post(delete_asset_from_form))
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage;

async fn login_page(maybe_user: Option<User>) -> Result<Response, AppError> {
    if maybe_user.is_some() {
        return Ok(Redirect::to("/").into_response());
    }

    Ok(Html(LoginPage.render()?).into_response())
}

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterPage;

async fn register_page(maybe_user: Option<User>) -> Result<Response, AppError> {
    if maybe_user.is_some() {
        return Ok(Redirect::to("/").into_response());
    }

    Ok(Html(RegisterPage.render()?).into_response())
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn login(
    repository: Repository,
    jar: CookieJar,
    Form(request): Form<LoginForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    let username = request.username.trim().to_string();

    if username.is_empty() || request.password.is_empty() {
        return Ok((jar, Redirect::to("/login?status=invalid-fields")));
    }

    let unauthenticated_user = UnauthenticatedUser::new(username, request.password);

    let user = match unauthenticated_user.authenticate(&repository).await {
        Ok(user) => user,

        Err(AppError::UserDoesNotExist) | Err(AppError::InvalidCredentials) => {
            return Ok((jar, Redirect::to("/login?status=invalid-credentials")));
        }

        Err(error) => return Err(error),
    };

    let token = user.auth_token()?;
    let cookie = authentication_cookie(token);

    Ok((jar.add(cookie), Redirect::to("/")))
}

#[derive(Deserialize)]
struct RegisterForm {
    username: String,
    password: String,
    password_confirmation: String,
}

async fn register(
    repository: Repository,
    jar: CookieJar,
    Form(request): Form<RegisterForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    let username = request.username.trim().to_string();
    let username_length = username.chars().count();
    let password_length = request.password.chars().count();

    if !(3..=40).contains(&username_length) {
        return Ok((jar, Redirect::to("/register?status=invalid-username")));
    }

    if !(8..=128).contains(&password_length) {
        return Ok((jar, Redirect::to("/register?status=invalid-password")));
    }

    if request.password != request.password_confirmation {
        return Ok((jar, Redirect::to("/register?status=password-mismatch")));
    }

    let unauthenticated_user = UnauthenticatedUser::new(username, request.password);

    let user = match unauthenticated_user.register(&repository).await {
        Ok(user) => user,

        Err(AppError::UsernameTaken) => {
            return Ok((jar, Redirect::to("/register?status=username-taken")));
        }

        Err(error) => return Err(error),
    };

    let token = user.auth_token()?;
    let cookie = authentication_cookie(token);

    Ok((jar.add(cookie), Redirect::to("/?status=registered")))
}

async fn logout(jar: CookieJar) -> (CookieJar, Redirect) {
    let removal_cookie = Cookie::build("token").path("/").build();

    (
        jar.remove(removal_cookie),
        Redirect::to("/login?status=logged-out"),
    )
}

fn authentication_cookie(token: String) -> Cookie<'static> {
    Cookie::build(("token", token))
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(cookie_secure())
        .path("/")
        .build()
}

fn cookie_secure() -> bool {
    env::var("COOKIE_SECURE")
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "true" | "1" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

#[derive(Deserialize)]
struct CreateAssetForm {
    name: String,
    unit_value: f64,
    quantity: f64,
}

async fn create_asset_from_form(
    maybe_user: Option<User>,
    repository: Repository,
    Form(request): Form<CreateAssetForm>,
) -> Result<Redirect, AppError> {
    let Some(user) = maybe_user else {
        return Ok(Redirect::to("/login"));
    };

    let name = request.name.trim().to_string();

    if name.is_empty() {
        return Ok(Redirect::to("/?status=empty-name"));
    }

    if request.unit_value <= 0.0 {
        return Ok(Redirect::to("/?status=invalid-value"));
    }

    if request.quantity <= 0.0 {
        return Ok(Redirect::to("/?status=invalid-quantity"));
    }

    if repository.asset_name_exists(user.id(), &name, None).await? {
        return Ok(Redirect::to("/?status=duplicate"));
    }

    repository
        .create_asset(user.id(), name, request.unit_value, request.quantity)
        .await?;

    Ok(Redirect::to("/?status=created"))
}

#[derive(Deserialize)]
struct UpdateAssetForm {
    id: i64,
    name: String,
    unit_value: f64,
    quantity: f64,
}

async fn update_asset_from_form(
    maybe_user: Option<User>,
    repository: Repository,
    Form(request): Form<UpdateAssetForm>,
) -> Result<Redirect, AppError> {
    let Some(user) = maybe_user else {
        return Ok(Redirect::to("/login"));
    };

    let name = request.name.trim().to_string();

    if request.id <= 0 {
        return Ok(Redirect::to("/?status=invalid"));
    }

    if name.is_empty() {
        return Ok(Redirect::to("/?status=empty-name"));
    }

    if request.unit_value <= 0.0 {
        return Ok(Redirect::to("/?status=invalid-value"));
    }

    if request.quantity <= 0.0 {
        return Ok(Redirect::to("/?status=invalid-quantity"));
    }

    if repository
        .asset_name_exists(user.id(), &name, Some(request.id))
        .await?
    {
        return Ok(Redirect::to("/?status=duplicate"));
    }

    let updated = repository
        .update_asset(
            user.id(),
            request.id,
            Some(name),
            Some(request.unit_value),
            Some(request.quantity),
        )
        .await?;

    match updated {
        Some(_) => Ok(Redirect::to("/?status=updated")),
        None => Ok(Redirect::to("/?status=not-found")),
    }
}

#[derive(Deserialize)]
struct DeleteAssetForm {
    id: i64,
}

async fn delete_asset_from_form(
    maybe_user: Option<User>,
    repository: Repository,
    Form(request): Form<DeleteAssetForm>,
) -> Result<Redirect, AppError> {
    let Some(user) = maybe_user else {
        return Ok(Redirect::to("/login"));
    };

    if request.id <= 0 {
        return Ok(Redirect::to("/?status=invalid"));
    }

    let deleted = repository.delete_asset(user.id(), request.id).await?;

    if deleted {
        Ok(Redirect::to("/?status=deleted"))
    } else {
        Ok(Redirect::to("/?status=not-found"))
    }
}

struct AssetView {
    id: i64,
    name: String,
    symbol: String,
    unit_value: String,
    unit_value_raw: String,
    quantity: String,
    quantity_raw: String,
    total_value: String,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardPage {
    username: String,
    portfolio_value: String,
    total_assets: usize,
    has_assets: bool,
    assets: Vec<AssetView>,
}

async fn index(maybe_user: Option<User>, repository: Repository) -> Result<Response, AppError> {
    let Some(user) = maybe_user else {
        return Ok(Redirect::to("/login").into_response());
    };

    let assets = repository.list_assets(user.id()).await?;
    let summary = PortfolioSummary::from_assets(&assets);

    let asset_views = assets
        .into_iter()
        .map(|asset| {
            let total_value = asset.total_value();
            let symbol = asset
                .name
                .chars()
                .take(2)
                .collect::<String>()
                .to_uppercase();

            AssetView {
                id: asset.id,
                symbol,
                name: asset.name,
                unit_value: format_brl(asset.unit_value),
                unit_value_raw: format!("{:.2}", asset.unit_value),
                quantity: format_quantity(asset.quantity),
                quantity_raw: format!("{:.4}", asset.quantity),
                total_value: format_brl(total_value),
            }
        })
        .collect::<Vec<_>>();

    let page = DashboardPage {
        username: user.username().to_string(),
        portfolio_value: format_brl(summary.portfolio_value),
        total_assets: summary.total_assets,
        has_assets: !asset_views.is_empty(),
        assets: asset_views,
    };

    Ok(Html(page.render()?).into_response())
}

fn format_quantity(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.4}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .replace('.', ",")
    }
}

fn format_brl(value: f64) -> String {
    let absolute_value = value.abs();
    let formatted = format!("{absolute_value:.2}");
    let mut parts = formatted.split('.');

    let integer_part = parts.next().unwrap_or("0");
    let decimal_part = parts.next().unwrap_or("00");

    let reversed = integer_part.chars().rev().collect::<Vec<_>>();
    let mut grouped = String::new();

    for (index, character) in reversed.iter().enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped.push('.');
        }

        grouped.push(*character);
    }

    let integer_formatted = grouped.chars().rev().collect::<String>();
    let signal = if value < 0.0 { "-" } else { "" };

    format!("{signal}R$ {integer_formatted},{decimal_part}")
}
