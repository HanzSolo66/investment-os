use axum::Router;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{
    Layer, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
};

use crate::routes;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

impl AppState {
    async fn new() -> color_eyre::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")?;
        let db = PgPool::connect(&database_url).await?;

        Ok(Self { db })
    }
}

pub struct App;

impl App {
    pub async fn start() -> color_eyre::Result<()> {
        let layer = tracing_subscriber::fmt::layer()
            .with_span_events(FmtSpan::NEW)
            .boxed();

        tracing_subscriber::registry().with(layer).init();

        dotenvy::dotenv().ok();
        let state = AppState::new().await?;

        sqlx::migrate!().run(&state.db).await?;

        let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
        let address = format!("0.0.0.0:{port}");
        let listener = TcpListener::bind(&address).await?;
        let router = Router::new()
            .nest("/api", routes::api::router())
            .merge(routes::frontend::router())
            .with_state(state);

        info!("Starting service");

        axum::serve(listener, router).await?;

        Ok(())
    }
}
