use std::sync::Arc;

use crate::storage::{InMemoryStorage, Storage};
use anyhow::Result;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Form, Router,
};
use axum_macros::debug_handler;
use nanoid::nanoid;
use serde::Deserialize;
use tracing::info;

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
    match state.storage.get(form_data.url.clone()) {
        Some(url) => url,
        None => {
            let short_url = generate_short_url();
            state.storage.set(short_url.clone(), form_data.url.clone());
            info!("{} -> {}", form_data.url, short_url);
            short_url
        }
    }
}

#[debug_handler]
async fn to_normal_url(
    Path(url): Path<String>,
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    let html = match state.storage.get(url.clone()) {
        Some(id) => id,
        None => "index.html".to_owned(),
    };
    info!("{} -> {}", url.clone(), html);

    let path = std::path::Path::new("./url_shorter/htmls/").join(html);
    match tokio::fs::read_to_string(path).await {
        Ok(html) => html,
        Err(_) => "index.html".to_owned(),
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
