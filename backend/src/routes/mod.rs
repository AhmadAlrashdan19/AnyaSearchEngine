pub mod search;

use search::search_handler;
use crate::elasticsearch::EsClient;
use axum::routing::get;
use axum::Router;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

#[allow(unused)]
pub fn build_router(es_client: Arc<EsClient>) -> Router {
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any);

    Router::new()
        .route("/search", get(search_handler))
        .with_state(es_client)
        .layer(cors)
}