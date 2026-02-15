use std::collections::{VecDeque, HashSet};
use url::Url;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct URLFrontier {
    queue: Mutex<VecDeque<Url>>,
    seen: Mutex<HashSet<Url>>,
}

impl URLFrontier {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            seen: Mutex::new(HashSet::new()),
        }
    }

    pub async fn add_url(&self, url: Url) {
        let mut seen = self.seen.lock().await;
        if seen.insert(url.clone()) {
            let mut queue = self.queue.lock().await;
            queue.push_back(url);
        }
    }

    pub async fn add_urls(&self, urls: Vec<Url>) {
        let mut seen = self.seen.lock().await;
        let mut queue = self.queue.lock().await;
        for url in urls {
            if seen.insert(url.clone()) {
                queue.push_back(url);
            }
        }
    }

    pub async fn next_url(&self) -> Option<Url> {
        let mut queue = self.queue.lock().await;
        queue.pop_front()
    }
}
