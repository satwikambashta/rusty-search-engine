mod model;
mod politeness;
mod frontier;
mod fetcher;
mod parser;

use std::sync::Arc;
use url::Url;
use crate::model::CrawlConfig;
use crate::politeness::PolitenessManager;
use crate::frontier::URLFrontier;
use crate::fetcher::Fetcher;
use crate::parser::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize configuration and components
    let config = Arc::new(CrawlConfig::default());
    let frontier = Arc::new(URLFrontier::new());
    let politeness = Arc::new(PolitenessManager::new(config.clone()));
    let fetcher = Arc::new(Fetcher::new(config.clone()));

    // Seed the frontier
    frontier.add_url(Url::parse("https://rust-lang.org")?).await;

    println!("Starting crawler...");

    // 2. Spawn workers
    let mut workers = Vec::new();
    for i in 0..config.max_concurrent_requests {
        let frontier = frontier.clone();
        let politeness = politeness.clone();
        let fetcher = fetcher.clone();
        
        let worker = tokio::spawn(async move {
            println!("Worker {} started", i);
            while let Some(url) = frontier.next_url().await {
                // Respect robots.txt
                if !politeness.can_crawl(&url).await {
                    continue;
                }

                // Respect per-domain rate limit
                politeness.wait_for_politeness(&url).await;

                println!("Worker {} fetching: {}", i, url);

                // Fetch the page with conditional headers
                match fetcher.fetch(&url, None, None).await {
                    Ok(Some(mut page)) => {
                        // Parse page content and extract links
                        Parser::parse(&mut page);
                        println!("Worker {} found {} links", i, page.outgoing_links.len());

                        // Add new links to frontier
                        frontier.add_urls(page.outgoing_links).await;

                        // TODO: Save to storage (S3/Postgres)
                    }
                    Ok(None) => println!("Worker {} page not modified: {}", i, url),
                    Err(e) => eprintln!("Worker {} failed to fetch {}: {}", i, url, e),
                }
            }
        });
        workers.push(worker);
    }

    // Wait for all workers or handle termination
    for worker in workers {
        let _ = worker.await;
    }

    Ok(())
}
