use harbour_chat_api::{create_app, Config};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    let state = harbour_chat_api::infrastructure::state::AppState::new(config.clone())
        .await
        .expect("failed to initialize application state");

    let app = create_app(state);
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind listener");

    tracing::info!(
        port = config.port,
        trust_gateway = config.trust_gateway_headers,
        "Harbour Chat API listening"
    );

    axum::serve(listener, app).await.expect("server error");
}
