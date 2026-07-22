use crate::parser::ParsedPage;

/// Phase 2 will send pages to Elasticsearch. For now we log what we found.
pub fn index_page(page: &ParsedPage) {
    let preview = page
        .content
        .chars()
        .take(120)
        .collect::<String>();

    println!("---");
    println!("URL: {}", page.url);
    println!("Title: {}", page.title);
    if !page.description.is_empty() {
        println!("Description: {}", page.description);
    }
    println!("Links found: {}", page.links.len());
    if !preview.is_empty() {
        println!("Content preview: {preview}...");
    }
}
