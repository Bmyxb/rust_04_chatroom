use std::sync::Arc;

use crate::storage::{InMemoryStorage, Storage, StorageError};
use anyhow::Result;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Form, Router,
};
use axum_macros::debug_handler;
use nanoid::nanoid;
use serde::Deserialize;
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum HttpError {
    #[error("url not found")]
    UrlNotFound,
    #[error("unknown error")]
    Unknown,
}

impl axum::response::IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        match self {
            HttpError::UrlNotFound => axum::response::Response::builder()
                .status(404)
                .body("url not found".into())
                .unwrap(),
            _ => axum::response::Response::builder()
                .status(500)
                .body(format!("{:?}", self).into())
                .unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<InMemoryStorage>,
}

fn generate_short_url() -> String {
    nanoid!(10).to_string()
}

#[derive(Deserialize)]
struct FormData {
    url: String,
}

#[debug_handler]
async fn get_short(
    State(state): State<AppState>,
    Form(form_data): Form<FormData>,
) -> impl axum::response::IntoResponse {
    loop {
        let short_url = generate_short_url();
        match state.storage.set(short_url.clone(), form_data.url.clone()) {
            Ok(_) => {
                info!("{} -> {}", form_data.url, short_url);
                return short_url;
            }
            Err(StorageError::AlreadyExists) => continue,
        }
    }
}

#[debug_handler]
async fn to_normal_url(
    Path(url): Path<String>,
    State(state): State<AppState>,
) -> Result<String, HttpError> {
    let html = match state.storage.get(url.clone()) {
        Some(id) => id,
        None => {
            return Err(HttpError::UrlNotFound);
        }
    };
    info!("{} -> {}", url.clone(), html);

    let path = std::path::Path::new("./url_shorter/htmls/").join(html);
    match tokio::fs::read_to_string(path).await {
        Ok(html) => Ok(html),
        Err(_) => Err(HttpError::Unknown),
    }
}

pub async fn serve(state: AppState) -> Result<()> {
    let router = Router::new()
        .route("/", post(get_short))
        .route("/:url", get(to_normal_url))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9000").await?;
    axum::serve(listener, router).await?;
    Ok(())
}
