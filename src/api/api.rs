// This is an INTERNAL api for interacting with the bot internals specifically.
// This is NOT the same as the public api and should only be used for internal purposes.

use std::{str::FromStr, sync::Arc};

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Extension, Router,
};
use axum_yaml::Yaml;
use serde::Deserialize;
use serde_json::{json, Value};
use twilight_model::id;

use crate::{api::auth, config::Config, handlers::Handler};

pub struct AppState {
    pub bot_instance: Arc<Handler>,
}

pub async fn start_api(bot_instance: Arc<Handler>) {
    let app_state = Arc::new(AppState { bot_instance });

    let auth_middleware = axum::middleware::from_fn(auth::authenticate);

    let app = Router::new()
        .route("/status", get(status))
        .route("/messages", post(send_message))
        .route("/leave/:guild_id", get(leave_guild))
        .route("/config/:guild_id", get(get_config))
        .route("/config/:guild_id", post(set_config))
        .layer(auth_middleware)
        .layer(Extension(app_state));

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));

    tracing::info!("Starting Internal API on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn status(state: Extension<Arc<AppState>>) -> Response {
    let state = Arc::clone(&state.0);
    let bot_instance = Arc::clone(&state.bot_instance);

    let db_status = bot_instance.db.status().await;
    let redis_status = bot_instance.redis.status().await;
    let discord_status = bot_instance.rest.current_user().await.is_ok();
    let mem = match bot_instance.redis.get_memory_usage().await {
        Ok(mem) => mem,
        Err(_) => 0,
    };

    Json(json!({
        "db": db_status,
        "redis": redis_status,
        "discord": discord_status,
        "memory": mem,
    }))
    .into_response()
}

async fn send_message(
    state: Extension<Arc<AppState>>,
    send_message: Option<Json<SendMessage>>,
) -> (StatusCode, Json<Value>) {
    if let Some(msg) = send_message {
        let msg = state
            .bot_instance
            .rest
            .create_message(id::Id::from_str(&msg.channel_id).unwrap())
            .content(&msg.message)
            .unwrap()
            .await;

        if let Ok(msg) = msg {
            let msg_id = msg.model().await.unwrap().id.to_string();
            (StatusCode::OK, Json(json!({ "message_id": msg_id })))
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Error sending message"
                })),
            )
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "No payload provided"
            })),
        )
    }
}

async fn leave_guild(
    state: Extension<Arc<AppState>>,
    Path(guild_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    let guild_id = match id::Id::from_str(&guild_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Invalid guild ID"
                })),
            )
        }
    };
    let resp = state.bot_instance.rest.leave_guild(guild_id).await;

    if let Ok(_) = resp {
        (
            StatusCode::OK,
            Json(json!({
                "message": "Left guild"
            })),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Error leaving guild"
            })),
        )
    }
}

async fn get_config(
    state: Extension<Arc<AppState>>,
    Path(guild_id): Path<String>,
    format: Query<ConfigParameters>,
) -> Response {
    let guild_id = guild_id.to_string();
    let config = state.bot_instance.db.get_guild(&guild_id).await;

    let format = match &format.format {
        Some(format) => format,
        None => &ConfigFormat::Json,
    };

    if let Ok(config) = config {
        if let Some(config) = config {
            match format {
                ConfigFormat::Json => return (StatusCode::OK, Json(json!(config))).into_response(),
                ConfigFormat::Yaml => {
                    let config = serde_yaml::to_string(&config).unwrap();
                    return (StatusCode::OK, Yaml(config)).into_response();
                }
            }
        }
        (StatusCode::OK, Json(json!(null))).into_response()
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Error getting config"
            })),
        )
            .into_response()
    }
}

async fn set_config(
    state: Extension<Arc<AppState>>,
    Path(guild_id): Path<String>,
    format: Query<ConfigParameters>,
    config: Option<String>,
) -> Response {
    let guild_id = guild_id.to_string();
    let format = match &format.format {
        Some(format) => format,
        None => &ConfigFormat::Json,
    };

    match format {
        ConfigFormat::Json => {
            let config = match config {
                Some(config) => config,
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "error": "No payload provided"
                        })),
                    )
                        .into_response()
                }
            };
            let config: Config = match serde_json::from_str(&config) {
                Ok(config) => config,
                Err(_) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "error": "Invalid JSON"
                        })),
                    )
                        .into_response()
                }
            };

            let res = state.bot_instance.db.set_guild(&guild_id, &config).await;
            if let Ok(_) = res {
                (StatusCode::OK, Json(json!({}))).into_response()
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Error setting config"
                    })),
                )
                    .into_response()
            }
        }
        ConfigFormat::Yaml => {
            let config = match config {
                Some(config) => config,
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "error": "No payload provided"
                        })),
                    )
                        .into_response()
                }
            };
            let config: Config = match serde_yaml::from_str(&config) {
                Ok(config) => config,
                Err(_) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "error": "Invalid YAML"
                        })),
                    )
                        .into_response()
                }
            };
            let res = state.bot_instance.db.set_guild(&guild_id, &config).await;
            if let Ok(_) = res {
                (StatusCode::OK, Json(json!({}))).into_response()
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Error setting config"
                    })),
                )
                    .into_response()
            }
        }
    }
}

#[derive(Deserialize)]
struct SendMessage {
    channel_id: String,
    message: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum ConfigFormat {
    Json,
    Yaml,
}

#[derive(Deserialize)]
struct ConfigParameters {
    format: Option<ConfigFormat>,
}
