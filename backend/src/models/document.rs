use serde::{Serialize, Deserialize};

/// A claim document for IndexedDocument in the crawler (index ayna_pages)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub url: String,
    pub title: String,
    pub description: String,

    #[serde(default)]
    pub content: String,

    #[serde(default)]
    pub links: usize,

    #[serde(default)]
    pub indexed_at: String,
}

/// Single search resulte, back to the frontend
#[derive(Debug)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub description: String,
    pub score: f32,
}

impl From<(Document, f32)> for SearchResult {
    fn from((doc, score): (Document, f32)) -> Self {
        Self {
            url: doc.url,
            title: doc.title,
            description: doc.description,
            score,
        }
    }
}

/// Full response on GET /search
#[derive(Debug)]
pub struct SearchResponse {
    pub query: String,
    pub totle: usize,
    pub results: Vec<SearchResult>,
}
