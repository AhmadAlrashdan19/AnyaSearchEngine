use crate::models::Document;
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::{json, Value};
use std::env;

const DEFAULT_ELASTICSEARCH_URL: &str = "http://127.0.0.1:9200";
const DEFAULT_ELASTICSEARCH_INDEX: &str = "ayna_pages";

pub struct EsClient {
    reqwest_client: Client,
    base_url: String,
    es_index: String,
}

impl EsClient {
    pub fn new() -> EsClient {
        EsClient {
            reqwest_client: Client::new(),
            base_url: env::var("ELASTICSEARCH_URL")
                .unwrap_or_else(|_| DEFAULT_ELASTICSEARCH_URL.to_string()),
            es_index: env::var("ELASTICSEARCH_INDEX")
                .unwrap_or_else(|_| DEFAULT_ELASTICSEARCH_INDEX.to_string())
        }
    }

    pub async fn search(
        &self,
        query: &str,
        from: usize,
        size: usize,
    ) -> Result<(usize, Vec<(Document, f32)>)> {
        let url = format!("{}/{}/_search",
            self.base_url,
            self.es_index
        );
        let body = json!({
            "from": from,
            "size": size,
            "query": {
                "multi_match": {
                    "query": query,
                    "fields": ["title^3", "description^2", "content"],
                }
            },
        });

        let response = self
            .reqwest_client
            .post(&url)
            .json(&body)
            .send().await
            .context("Failed to send request to Elasticsearch")?;
        
        if !response.status().is_success() {
            anyhow::bail!("Elasticsearch request failed with status: {}", response.status());
        }
        
        let payload: Value = response
        .json().await
        .context("Failed to parse Elasticsearch response")?;
        
        let total: usize = payload["hits"]["total"]["value"]
            .as_u64()
            .unwrap_or(0) as usize;
        
        let hits = payload["hits"]["hits"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        let mut results = Vec::with_capacity(hits.len());
        for hit in hits {
            let score = hit["_score"]
                .as_f64()
                .unwrap_or(0.0) as f32;

            if let Ok(doc) = serde_json::from_value::<Document>(hit["_source"].clone()) {
                results.push((doc, score));
            }
        }

        Ok((total, results))
    }
}
