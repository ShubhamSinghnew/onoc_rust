use axum::{http::StatusCode, middleware::Next, response::Response, body::Body,};
use axum::extract::Request;


pub async fn jwt_auth(req: Request<Body>, next: Next)-> Result<Response, StatusCode> {
    // Validate JWT here
    Ok(next.run(req).await)
}
