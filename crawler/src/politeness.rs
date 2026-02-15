use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use dashmap::DashMap;
use chrono::{DateTime, Utc};
use url::Url;
use robots_txt::Robots;
use crate::model::CrawlConfig;

pub struct PolitenessManager {
    config: Arc<CrawlConfig>,
    // Domain -> Robots rules
    robots_cache: DashMap<String, Arc<Robots<'static>>>,
    // Domain -> Last request timestamp
    last_request: DashMap<String, DateTime<Utc>>,
    // Domain -> Active workers (to ensure only one worker hits a domain)
    active_domains: DashMap<String, Mutex<()>>,
}

impl PolitenessManager {
    pub fn new(config: Arc<CrawlConfig>) -> Self {
        Self {
            config,
            robots_cache: DashMap::new(),
            last_request: DashMap::new(),
            active_domains: DashMap::new(),
        }
    }

    pub async fn can_crawl(&self, url: &Url) -> bool {
        let domain = match url.domain() {
            Some(d) => d,
            None => return false,
        };

        // 1. Check robots.txt (Simplified - in real life we'd fetch it if not cached)
        if let Some(robots) = self.robots_cache.get(domain) {
            if !robots.allowed(self.config.user_agent.as_str(), url.path()) {
                return false;
            }
        }

        true
    }

    pub async fn wait_for_politeness(&self, url: &Url) {
        let domain = match url.domain() {
            Some(d) => d.to_string(),
            None => return,
        };

        // Use a per-domain lock to ensure distributed workers don't hit same domain
        let _lock = self.active_domains.entry(domain.clone()).or_insert_with(|| Mutex::new(())).value().lock().await;

        if let Some(last) = self.last_request.get(&domain) {
            let elapsed = Utc::now().signed_duration_since(*last);
            let delay = chrono::Duration::from_std(self.config.default_delay).unwrap_or(chrono::Duration::seconds(1));
            
            if elapsed < delay {
                let wait_time = delay - elapsed;
                tokio::time::sleep(wait_time.to_std().unwrap_or(std::time::Duration::from_secs(1))).await;
            }
        }
        
        self.last_request.insert(domain, Utc::now());
    }

    pub fn update_robots(&self, domain: String, content: &str) {
        // Implementation for parsing and caching robots.xml content
        // This is a placeholder for the actual robots-txt crate usage
    }
}
