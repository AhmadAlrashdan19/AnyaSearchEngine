use crate::parser::ParsedPage;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Serialize;
use std::env;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const DEFAULT_ELASTICSEARCH_URL: &str = "http://127.0.0.1:9200";
const INDEX_NAME: &str = "ayna_pages";

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

fn document_id(url: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

// Creates the index with the proper mapping/analyzer if it doesn't exist yet.
// Safe to call on every document since it short-circuits after the first check.
async fn ensure_index_exists(client: &Client, base_url: &str) -> Result<()> {
    let check = client
        .head(format!("{base_url}/{INDEX_NAME}"))
        .send()
        .await
        .context("failed to check if index exists")?;
 
    if check.status().is_success() {
        return Ok(());
    }
 
    let mapping = serde_json::json!({
        "settings": {
            "analysis": {
                "analyzer": {
                    "ayna_text_analyzer": {
                        "type": "custom",
                        "tokenizer": "standard",
                        "filter": ["lowercase", "asciifolding", "stop"]
                    }
                }
            }
        },
        "mappings": {
            "properties": {
                "url": { "type": "keyword" },
                "title": {
                    "type": "text",
                    "analyzer": "ayna_text_analyzer",
                    "fields": { "raw": { "type": "keyword" } }
                },
                "description": { "type": "text", "analyzer": "ayna_text_analyzer" },
                "content": { "type": "text", "analyzer": "ayna_text_analyzer" },
                "links": { "type": "integer" },
                "indexed_at": { "type": "date" }
            }
        }
    });
 
    let response = client
        .put(format!("{base_url}/{INDEX_NAME}"))
        .json(&mapping)
        .send()
        .await
        .context("failed to create Elasticsearch index")?;
 
    // 400 here usually means another worker created it concurrently; that's fine.
    if !response.status().is_success() && response.status().as_u16() != 400 {
        anyhow::bail!("failed to create index: {}", response.status());
    }
 
    Ok(())
}

async fn publish_to_elasticsearch(document: &IndexedDocument) -> Result<()> {
    let base_url = env::var("ELASTICSEARCH_URL").unwrap_or_else(|_| DEFAULT_ELASTICSEARCH_URL.to_string());
    let client = Client::new();
    let doc_id = document_id(&document.url);

    ensure_index_exists(&client, &base_url).await?;

    let response = client
        .put(format!("{base_url}/{INDEX_NAME}/_doc/{doc_id}"))
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
    fn builds_document_with_expected_fields() {
        let page = ParsedPage {
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            description: "A test page".to_string(),
            content: "Some body text".to_string(),
            links: vec!["https://example.com/about".to_string()],
        };

        let document = build_document(&page);
        assert_eq!(document.url, "https://example.com");
        assert_eq!(document.title, "Example");
        assert_eq!(document.links, 1);
    }

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
