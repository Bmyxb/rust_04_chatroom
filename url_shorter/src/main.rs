mod http;
mod storage;

use std::sync::Arc;

use anyhow::Result;
use storage::InMemoryStorage;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
pub async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let storage = InMemoryStorage::new();
    let app_state = http::AppState {
        storage: Arc::new(storage),
    };
    http::serve(app_state).await?;
    Ok(())
}
