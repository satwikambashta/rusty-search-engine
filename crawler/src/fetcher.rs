use reqwest::{Client, StatusCode};
use url::Url;
use std::sync::Arc;
use crate::model::{CrawlConfig, CrawledPage};
use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;

pub struct Fetcher {
    client: Client,
    config: Arc<CrawlConfig>,
}

impl Fetcher {
    pub fn new(config: Arc<CrawlConfig>) -> Self {
        let client = Client::builder()
            .user_agent(format!("{} (+{})", config.user_agent, config.bot_info_url))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap();
        
        Self { client, config }
    }

    pub async fn fetch(&self, url: &Url, etag: Option<&str>, last_modified: Option<&str>) -> Result<Option<CrawledPage>> {
        let mut request = self.client.get(url.as_str());

        if let Some(e) = etag {
            request = request.header("If-None-Match", e);
        }
        if let Some(lm) = last_modified {
            request = request.header("If-Modified-Since", lm);
        }

        let response = request.send().await?;

        if response.status() == StatusCode::NOT_MODIFIED {
            return Ok(None);
        }

        let status = response.status().as_u16();
        let mut headers = HashMap::new();
        for (name, value) in response.headers().iter() {
            headers.insert(name.to_string(), value.to_str().unwrap_or("").to_string());
        }

        let new_etag = response.headers().get("ETag").and_then(|h| h.to_str().ok()).map(|s| s.to_string());
        let new_last_modified = response.headers().get("Last-Modified").and_then(|h| h.to_str().ok()).map(|s| s.to_string());
        
        let html = response.text().await?;
        
        // Basic page model creation
        Ok(Some(CrawledPage {
            url: url.clone(),
            status_code: status,
            headers,
            html,
            text_content: String::new(), // To be filled by parser
            outgoing_links: Vec::new(),  // To be filled by parser
            timestamp: Utc::now(),
            content_hash: String::new(), // To be filled by hasher
            etag: new_etag,
            last_modified: new_last_modified,
        }))
    }
}
