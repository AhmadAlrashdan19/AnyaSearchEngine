mod elasticsearch;
mod models;
mod routes;

use axum;
use anyhow;
use tokio;
use tokio::net::TcpListener;
use elasticsearch::EsClient;
use routes::build_router;
use std::net::SocketAddr;
use std::sync::Arc;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let es_client = Arc::new(EsClient::new());
    let app = build_router(es_client);

    let address = SocketAddr::from(([0, 0, 0, 0], 5050));
    println!("Ayna backend listening on http://{address}");

    let listener = TcpListener::bind(address).await?;

    axum::serve(listener, app).await?;

    Ok(())
}