use std::collections::{HashSet, VecDeque};

/// Breadth-first frontier of URLs waiting to be crawled.
pub struct CrawlQueue {
    frontier: VecDeque<String>,
    seen: HashSet<String>,
    crawled_count: usize,
    allowed_host: String,
}

impl CrawlQueue {
    pub fn new(seed_url: &str, allowed_host: &str) -> Self {
        let mut queue = Self {
            frontier: VecDeque::new(),
            seen: HashSet::new(),
            crawled_count: 0,
            allowed_host: allowed_host.to_string(),
        };

        if let Some(normalized) = queue.normalize(seed_url) {
            if queue.seen.insert(normalized.clone()) {
                queue.frontier.push_back(normalized);
            }
        }

        queue
    }

    pub fn visited_count(&self) -> usize {
        self.crawled_count
    }

    pub fn pending_count(&self) -> usize {
        self.frontier.len()
    }

    pub fn next_url(&mut self) -> Option<String> {
        let url = self.frontier.pop_front()?;
        self.crawled_count += 1;
        Some(url)
    }

    pub fn enqueue_links(&mut self, links: &[String]) {
        for link in links {
            if let Some(normalized) = self.normalize(link) {
                if self.seen.insert(normalized.clone()) {
                    self.frontier.push_back(normalized);
                }
            }
        }
    }

    fn normalize(&self, raw_url: &str) -> Option<String> {
        let mut parsed = url::Url::parse(raw_url).ok()?;

        if parsed.scheme() != "http" && parsed.scheme() != "https" {
            return None;
        }
        if parsed.host_str()? != self.allowed_host {
            return None;
        }

        parsed.set_fragment(None);
        Some(parsed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deduplicates_urls() {
        let mut queue = CrawlQueue::new("https://example.com/", "example.com");
        assert_eq!(queue.next_url().as_deref(), Some("https://example.com/"));

        queue.enqueue_links(&[
            "https://example.com/about".to_string(),
            "https://example.com/about".to_string(),
            "https://example.com/about#team".to_string(),
        ]);

        assert_eq!(queue.pending_count(), 1);
        assert_eq!(
            queue.next_url().as_deref(),
            Some("https://example.com/about")
        );
    }
}
