use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use crate::elasticsearch::client::EsClient;
use crate::models::{SearchResult, SearchResponse};
// use crate::routes::search;

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub query: String,

    #[serde(default)]
    pub page: Option<usize>,

    #[serde(default)]
    pub per_page: Option<usize>,
}

#[allow(unused)]
#[axum::debug_handler]
pub async fn search_handler(
    State(es_client): State<Arc<EsClient>>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>, (StatusCode, String)> {
    let query = params.query.trim();
    if query.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Query parameter 'query' is required".to_string()));
    }

    let per_page = params.per_page.unwrap_or(10).clamp(1, 50);
    let page = params.page.unwrap_or(1).max(1);
    let from = (page - 1) * per_page;

    let (total, hits) = es_client
    .search(query, from, per_page)
    .await
    .map_err(|error| (StatusCode::BAD_GATEWAY, error.to_string()))?;

    let results: Vec<SearchResult> = hits.into_iter().map(SearchResult::from).collect();

    Ok(Json(SearchResponse {
        query: query.to_string(),
        total,
        results
    }))
}
