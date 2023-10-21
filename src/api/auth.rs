use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub async fn authenticate<B>(request: Request<B>, next: Next<B>) -> Response {
    let auth_key = request.headers().get("Authorization");

    if auth_key.is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "No authorization header provided"})),
        )
            .into_response();
    }

    let auth_key = match auth_key.and_then(|key| key.to_str().ok()) {
        Some(auth_key) => auth_key,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid authorization key"})),
            )
                .into_response()
        }
    };

    if auth_key != std::env::var("INTERNAL_API_KEY").unwrap() {
        // unwrap is safe because we check for it in main.rs
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid authorization key"})),
        )
            .into_response();
    }

    let response = next.run(request).await;

    response
}
