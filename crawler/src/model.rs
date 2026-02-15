use serde::{Deserialize, Serialize};
use url::Url;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawledPage {
    pub url: Url,
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub html: String,
    pub text_content: String,
    pub outgoing_links: Vec<Url>,
    pub timestamp: DateTime<Utc>,
    pub content_hash: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

pub struct CrawlConfig {
    pub user_agent: String,
    pub bot_info_url: String,
    pub contact_email: String,
    pub default_delay: std::time::Duration,
    pub max_concurrent_requests: usize,
}

impl Default for CrawlConfig {
    fn default() -> Self {
        Self {
            user_agent: "MySearchBot/1.0".to_string(),
            bot_info_url: "https://mysite.com/bot-info".to_string(),
            contact_email: "crawler@mysite.com".to_string(),
            default_delay: std::time::Duration::from_secs(2),
            max_concurrent_requests: 10,
        }
    }
}
