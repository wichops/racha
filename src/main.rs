use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store::PostgresStore;

use racha::{AppState, config, db, routes};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let cfg = config::Config::from_env();
    let pool = db::create_pool(&cfg.database_url).await;

    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await.expect("Failed to migrate session store");

    let session_layer = SessionManagerLayer::new(session_store);

    let state = AppState { db: pool };

    let app = routes::build_router()
        .nest_service("/static", ServeDir::new("static"))
        .layer(session_layer)
        .with_state(state);

    let listener = TcpListener::bind(&cfg.bind_addr).await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
