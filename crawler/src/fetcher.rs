use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;

const USER_AGENT: &str = "AynaBot/0.1 (+https://github.com/ayna-search-engine)";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// Downloads the raw HTML body for a URL over HTTP.
pub struct Fetcher {
    client: Client,
}

impl Fetcher {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(REQUEST_TIMEOUT)
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .context("failed to build HTTP client")?;

        Ok(Self { client })
    }

    pub async fn fetch(&self, url: &str) -> Result<String> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .with_context(|| format!("request failed for {url}"))?;

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("HTTP {status} for {url}");
        }

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("");

        if !content_type.contains("text/html") && !content_type.is_empty() {
            anyhow::bail!("skipping non-HTML content at {url} ({content_type})");
        }

        response
            .text()
            .await
            .with_context(|| format!("failed to read response body for {url}"))
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}
