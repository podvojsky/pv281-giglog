use crate::app::App;
use anyhow::Result;

mod app;
mod auth;
mod error;
mod handlers;
mod middleware;
mod models;
mod regex;
mod repositories;
mod templates;
mod utils;
mod view_models;

#[tokio::main]
async fn main() -> Result<()> {
    App::new().await?.serve().await
}
