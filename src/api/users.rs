use axum::{Json, extract::Path};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn get_user(Path(id): Path<i32>) -> Json<serde_json::Value> {
    Json(json!({
        "id": id,
        "name": "John Doe"
    }))
}


