use crate::parser::ParsedPage;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Serialize;
use std::env;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const DEFAULT_ELASTICSEARCH_URL: &str = "http://127.0.0.1:9200";
const DEFAULT_INDEX_NAME: &str = "ayna_pages";

// Shape of the stored page record.
#[derive(Debug, Serialize)]
struct IndexedDocument {
    url: String,
    title: String,
    description: String,
    content: String,
    links: usize,
    indexed_at: String,
}

// Entry point for indexing a parsed page.
pub async fn index_page(page: &ParsedPage) {
    let document = build_document(page);
    let index_path = default_index_path();

    if let Err(error) = index_page_to_path_from_document(&document, &index_path) {
        eprintln!("failed to write index entry: {error}");
    }

    if let Err(error) = publish_to_elasticsearch(&document).await {
        eprintln!("failed to publish to Elasticsearch: {error}");
    }
}

// Write one JSON record for each crawled page.
#[cfg(test)]
pub fn index_page_to_path(page: &ParsedPage, path: &Path) -> std::io::Result<()> {
    let document = build_document(page);
    index_page_to_path_from_document(&document, path)
}

fn index_page_to_path_from_document(document: &IndexedDocument, path: &Path) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    let line = serde_json::to_string(document).unwrap();
    writeln!(file, "{line}")?;
    Ok(())
}

fn build_document(page: &ParsedPage) -> IndexedDocument {
    IndexedDocument {
        url: page.url.clone(),
        title: page.title.clone(),
        description: page.description.clone(),
        content: page.content.clone(),
        links: page.links.len(),
        indexed_at: chrono::Utc::now().to_rfc3339(),
    }
}

async fn publish_to_elasticsearch(document: &IndexedDocument) -> Result<()> {
    let base_url = env::var("ELASTICSEARCH_URL").unwrap_or_else(|_| DEFAULT_ELASTICSEARCH_URL.to_string());
    let index_name = env::var("ELASTICSEARCH_INDEX").unwrap_or_else(|_| DEFAULT_INDEX_NAME.to_string());
    let client = Client::new();
    let response = client
        .post(format!("{base_url}/{index_name}/_doc"))
        .json(document)
        .send()
        .await
        .context("failed to send document to Elasticsearch")?;

    if !response.status().is_success() {
        anyhow::bail!("Elasticsearch returned {}", response.status());
    }

    Ok(())
}

// Default path for the local index file.
fn default_index_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("index.jsonl");
    if let Some(parent) = path.parent() {
        let _ = create_dir_all(parent);
    }
    path
}

// Quick check that the index file contains valid JSON data.
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn writes_one_json_line_per_page() {
        let temp_dir = std::env::temp_dir().join(format!(
            "ayna-indexer-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let index_path = temp_dir.join("pages.jsonl");
        let page = ParsedPage {
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            description: "A test page".to_string(),
            content: "Some body text".to_string(),
            links: vec!["https://example.com/about".to_string()],
        };

        index_page_to_path(&page, &index_path).unwrap();

        let content = fs::read_to_string(&index_path).unwrap();
        assert!(content.contains("\"url\":\"https://example.com\""));
        assert!(content.contains("\"title\":\"Example\""));
        assert!(content.contains("\"links\":1"));

        let _ = fs::remove_dir_all(temp_dir);
    }
}
