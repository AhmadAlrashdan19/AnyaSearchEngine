use anyhow::{Context, Result};
use robotstxt::DefaultMatcher;
use reqwest::Client;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use url::Url;

const USER_AGENT: &str = "AynaBot";

/// Checks whether a URL is allowed according to each site's robots.txt rules.
pub struct RobotsChecker {
    client: Client,
    cache: HashMap<String, String>,
}

impl RobotsChecker {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            cache: HashMap::new(),
        }
    }

    pub async fn is_allowed(&mut self, url: &str) -> Result<bool> {
        let parsed = Url::parse(url).context("invalid URL for robots check")?;
        let host = parsed
            .host_str()
            .context("URL missing host for robots check")?
            .to_string();

        if !self.cache.contains_key(&host) {
            let robots_url = format!("{}://{}/robots.txt", parsed.scheme(), host);
            let body = match self.client.get(&robots_url).send().await {
                Ok(response) if response.status().is_success() => {
                    response.text().await.unwrap_or_default()
                }
                _ => String::new(),
            };

            self.cache.insert(host.clone(), body);
        }

        let robots_body = self.cache.get(&host).expect("robots cache populated");
        let mut matcher = DefaultMatcher::default();
        Ok(matcher.one_agent_allowed_by_robots(robots_body, USER_AGENT, url))
    }
}

/// Ensures we wait between requests to the same domain.
pub struct DomainRateLimiter {
    min_delay: Duration,
    last_request: HashMap<String, Instant>,
}

impl DomainRateLimiter {
    pub fn new(min_delay: Duration) -> Self {
        Self {
            min_delay,
            last_request: HashMap::new(),
        }
    }

    // Computes how long to wait for this domain and records the request time.
    // Does NOT sleep itself — caller must await the returned duration
    // *after* releasing any lock on this limiter, or concurrency is lost.
    pub async fn reserve_slot(&mut self, url: &str) -> Result<Duration> {
        let host = Url::parse(url)
            .context("invalid URL for rate limiting")?
            .host_str()
            .context("URL missing host for rate limiting")?
            .to_string();

        let wait = match self.last_request.get(&host) {
            Some(last_request) => {
                let elapsed = last_request.elapsed();
                self.min_delay.saturating_sub(elapsed)
            }
            None => Duration::ZERO,
        };

        self.last_request.insert(host, Instant::now());
        Ok(wait)
    }
}
