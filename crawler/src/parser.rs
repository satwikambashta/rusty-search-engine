use scraper::{Html, Selector};
use url::Url;
use crate::model::CrawledPage;

pub struct Parser;

impl Parser {
    pub fn parse(page: &mut CrawledPage) {
        let document = Html::parse_document(&page.html);
        
        // Extract links
        let selector = Selector::parse("a[href]").unwrap();
        let mut links = Vec::new();
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                if let Ok(url) = page.url.join(href) {
                    if url.scheme() == "http" || url.scheme() == "https" {
                        links.push(url);
                    }
                }
            }
        }
        page.outgoing_links = links;

        // Extract text content (simplified)
        let text_selector = Selector::parse("body").unwrap();
        if let Some(body) = document.select(&text_selector).next() {
            page.text_content = body.text().collect::<Vec<_>>().join(" ").trim().to_string();
        }

        // Generate content hash for deduplication
        let mut context = md5::Context::new();
        context.consume(page.text_content.as_bytes());
        page.content_hash = format!("{:x}", context.compute());
    }
}
