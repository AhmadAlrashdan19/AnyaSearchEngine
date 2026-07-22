use scraper::{Html, Selector};
use url::Url;

/// Structured data extracted from a single HTML page.
#[derive(Debug, Clone)]
pub struct ParsedPage {
    pub url: String,
    pub title: String,
    pub description: String,
    pub content: String,
    pub links: Vec<String>,
}

/// Parses HTML and extracts the title, description, body text, and outbound links.
pub fn parse_page(html: &str, page_url: &str, allowed_host: &str) -> ParsedPage {
    let document = Html::parse_document(html);
    let base_url = Url::parse(page_url).unwrap_or_else(|_| Url::parse("http://localhost/").unwrap());

    let title = select_first_text(&document, "title").unwrap_or_else(|| "Untitled".to_string());
    let description = meta_description(&document);
    let content = extract_body_text(&document);
    let links = extract_links(&document, &base_url, allowed_host);

    ParsedPage {
        url: page_url.to_string(),
        title,
        description,
        content,
        links,
    }
}

fn select_first_text(document: &Html, selector_str: &str) -> Option<String> {
    let selector = Selector::parse(selector_str).ok()?;
    document
        .select(&selector)
        .next()
        .map(|element| normalize_whitespace(&element.text().collect::<String>()))
}

fn meta_description(document: &Html) -> String {
    let selector = match Selector::parse(r#"meta[name="description"]"#) {
        Ok(selector) => selector,
        Err(_) => return String::new(),
    };

    document
        .select(&selector)
        .next()
        .and_then(|element| element.value().attr("content"))
        .map(normalize_whitespace)
        .unwrap_or_default()
}

fn extract_body_text(document: &Html) -> String {
    let body_selector = match Selector::parse("body") {
        Ok(selector) => selector,
        Err(_) => return String::new(),
    };

    let text = document
        .select(&body_selector)
        .next()
        .map(|body| normalize_whitespace(&body.text().collect::<String>()))
        .unwrap_or_default();

    truncate_chars(text, 2_000)
}

fn extract_links(document: &Html, base_url: &Url, allowed_host: &str) -> Vec<String> {
    let anchor_selector = match Selector::parse("a[href]") {
        Ok(selector) => selector,
        Err(_) => return Vec::new(),
    };

    let mut links = Vec::new();

    for anchor in document.select(&anchor_selector) {
        let Some(href) = anchor.value().attr("href") else {
            continue;
        };

        if let Some(normalized) = normalize_link(href, base_url, allowed_host) {
            links.push(normalized);
        }
    }

    links.sort();
    links.dedup();
    links
}

fn normalize_link(raw_href: &str, base_url: &Url, allowed_host: &str) -> Option<String> {
    let trimmed = raw_href.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("mailto:") || lower.starts_with("javascript:") || lower.starts_with("tel:") {
        return None;
    }

    let joined = base_url.join(trimmed).ok()?;
    if joined.scheme() != "http" && joined.scheme() != "https" {
        return None;
    }
    if joined.host_str()? != allowed_host {
        return None;
    }

    let mut normalized = joined;
    normalized.set_fragment(None);
    Some(normalized.to_string())
}

fn normalize_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_chars(input: String, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input;
    }

    input.chars().take(max_chars).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_title_and_links() {
        let html = r#"
            <html>
              <head>
                <title>Example Domain</title>
                <meta name="description" content="An example page." />
              </head>
              <body>
                <p>Hello world</p>
                <a href="/about">About</a>
                <a href="https://other.com/page">External</a>
              </body>
            </html>
        "#;

        let parsed = parse_page(html, "https://example.com/", "example.com");
        assert_eq!(parsed.title, "Example Domain");
        assert_eq!(parsed.description, "An example page.");
        assert!(parsed.content.contains("Hello world"));
        assert_eq!(parsed.links, vec!["https://example.com/about"]);
    }
}
