mod api;
mod config;
mod db;
mod middleware;
use api::auth::create_admin_users;
use config::Config;

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tokio::net::TcpListener;

// Explicitly import the handler functions 
use crate::api::auth::{login, create_admin, register, verify_otp};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

// Health check handler with concrete return type
pub async fn check_database_connection(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "✅ Database connection successful"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Postgres error: {}", e)})),
        ),
    }
}

// Startup DB check function (no Axum State)
pub async fn check_database_connection_startup(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await.map(|_| ())
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let config = Config::from_env();

    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            println!("✅ Connection successful to DB: {}", config.app_name);
            pool
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to DB: {:?}", e);
            std::process::exit(1);
        }
    };

    // Use the startup DB check function here
    if let Err(e) = check_database_connection_startup(&pool).await {
        eprintln!("❌ Database connection failed during startup: {:?}", e);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Database connection failed",
        ));
    } else {
        println!("✅ Database connected successfully at startup");
    }

    let app_state = AppState { pool };

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(check_database_connection))
        .route("/login", post(login))
        .route("/verify", post(verify_otp))
        .route("/add_user", post(register))
        .route("/ceate_user",post(create_admin))
        .route("/ceate_admin_user",post(create_admin_users))
        .with_state(app_state);

    let listener = TcpListener::bind("localhost:3100")
        .await
        .expect("Failed to bind to address");

    println!("🚀 Server running at http://localhost:3100");

    axum::serve(listener, app)
        .await
        .expect("Server failed");

    Ok(())
}

async fn root_handler() -> Json<serde_json::Value> {
    Json(json!({
        "message": "Hello from Axum!"
    }))
}