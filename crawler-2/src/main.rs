use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use reqwest::{Client, StatusCode};
use robots_txt::Robots;
use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;
use url::Url;

const USER_AGENT: &str = "MySearchBot/0.1 (+https://example.com/bot)";
const DEFAULT_DELAY: Duration = Duration::from_secs(2);
const MAX_DEPTH: usize = 2;

#[derive(Clone)]
struct DomainState {
    last_access: Arc<Mutex<Instant>>,
    delay: Arc<Mutex<Duration>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(15))
        .build()?;

    let frontier = Arc::new(Mutex::new(VecDeque::new()));
    frontier.lock().await.push_back(("https://example.com".to_string(), 0));

    let visited = Arc::new(DashMap::new());
    let domain_states = Arc::new(DashMap::new());
    let robots_cache = Arc::new(DashMap::new());

    while let Some((url, depth)) = frontier.lock().await.pop_front() {
        if depth > MAX_DEPTH || visited.contains_key(&url) {
            continue;
        }

        visited.insert(url.clone(), true);

        if let Err(e) = crawl(
            &client,
            &url,
            depth,
            &frontier,
            &domain_states,
            &robots_cache,
        )
        .await
        {
            println!("Error crawling {}: {:?}", url, e);
        }
    }

    Ok(())
}

async fn crawl(
    client: &Client,
    url_str: &str,
    depth: usize,
    frontier: &Arc<Mutex<VecDeque<(String, usize)>>>,
    domain_states: &DashMap<String, DomainState>,
    robots_cache: &DashMap<String, Robots>,
) -> Result<()> {
    let url = Url::parse(url_str)?;
    let domain = url.domain().unwrap_or("").to_string();

    let state = domain_states
        .entry(domain.clone())
        .or_insert_with(|| DomainState {
            last_access: Arc::new(Mutex::new(Instant::now() - DEFAULT_DELAY)),
            delay: Arc::new(Mutex::new(DEFAULT_DELAY)),
        })
        .clone();

    {
        let mut last = state.last_access.lock().await;
        let delay = *state.delay.lock().await;
        let elapsed = last.elapsed();

        if elapsed < delay {
            sleep(delay - elapsed).await;
        }

        *last = Instant::now();
    }

    if !allowed_by_robots(client, &domain, url.path(), robots_cache).await? {
        println!("Blocked by robots.txt: {}", url_str);
        return Ok(());
    }

    println!("Crawling: {}", url_str);

    let response = client.get(url_str).send().await?;

    match response.status() {
        StatusCode::OK => {
            let body = response.text().await?;
            extract_links(&body, &url, depth, frontier).await?;
        }
        StatusCode::TOO_MANY_REQUESTS => {
            println!("Rate limited on {}", domain);
            let mut delay = state.delay.lock().await;
            *delay *= 2;    
        }
        status => {
            println!("Status {} on {}", status, url_str);
        }
    }

    Ok(())
}

async fn allowed_by_robots(
    client: &Client,
    domain: &str,
    path: &str,
    cache: &DashMap<String, Robots>,
) -> Result<bool> {
    if let Some(robots) = cache.get(domain) {
        return Ok(robots.is_allowed(USER_AGENT, path));
    }

    let robots_url = format!("https://{}/robots.txt", domain);

    if let Ok(resp) = client.get(&robots_url).send().await {
        if let Ok(text) = resp.text().await {
            if let Ok(robots) = Robots::from_str(&text) {
                cache.insert(domain.to_string(), robots.clone());
                return Ok(robots.is_allowed(USER_AGENT, path));
            }
        }
    }

    Ok(true) // default allow if no robots.txt
}

async fn extract_links(
    body: &str,
    base_url: &Url,
    depth: usize,
    frontier: &Arc<Mutex<VecDeque<(String, usize)>>>,
) -> Result<()> {
    let document = Html::parse_document(body);
    let selector = Selector::parse("a").unwrap();

    for element in document.select(&selector) {
        if let Some(link) = element.value().attr("href") {
            if let Ok(next_url) = base_url.join(link) {
                frontier
                    .lock()
                    .await
                    .push_back((next_url.to_string(), depth + 1));
            }
        }
    }

    Ok(())
}
