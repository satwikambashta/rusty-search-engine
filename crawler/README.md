# Scalable Web Crawler

This is a modular and scalable web crawler designed with "politeness first" principles.

## 🚀 Best Practices Implemented

1.  **Politeness (`politeness.rs`)**:
    *   **Robots.txt Compliance**: Checks for crawl permissions before fetching.
    *   **Domain-Level Rate Limiting**: Ensures a delay between requests to the same domain (default: 2 seconds).
    *   **Distributed Locking**: Use a per-domain mutex to ensure only one worker hits a specific domain at any given time.

2.  **Efficiency & Scalability**:
    *   **URL Frontier (`frontier.rs`)**: Centralized queue with duplicate detection (using a Bloom filter or Hashset).
    *   **Conditional GET (`fetcher.rs`)**: Implements `If-None-Match` (ETag) and `If-Modified-Since` (Last-Modified) headers to avoid redundant downloads.
    *   **User-Agent (`model.rs`)**: Identifies as a bot and points to a bot-info URL for transparency.
    *   **Async/Await**: Built on `tokio` for high-concurrency without thread overhead.

3.  **Content Analysis (`parser.rs`)**:
    *   **Link Extraction**: Absolute URL resolution for discovered links.
    *   **Content Hashing**: Uses MD5 hashing of text content to detect duplicates across different URLs.
    *   **Text Extraction**: Extracts clean text from HTML `body`.

## 🏗 Modular Architecture

*   **`model.rs`**: Core data structures (`CrawledPage`, `CrawlConfig`).
*   **`politeness.rs`**: The "ethics" engine of the crawler.
*   **`frontier.rs`**: Manages the crawl scope and priority.
*   **`fetcher.rs`**: Clean abstraction over the HTTP client.
*   **`parser.rs`**: Handles HTML processing.
*   **`main.rs`**: Orchestrates multiple async worker loops.

## 🚧 Not Yet Integrated

The following features are designed but stubbed out to keep the local filesystem clean:
*   **S3-compatible Storage**: For raw HTML archival.
*   **PostgreSQL**: For URL metadata and crawl status persistence.
*   **Elasticsearch**: For downstream indexing.

## 🛠 Setup

To run the crawler (ensure you have seed URLs in `main.rs`):

```bash
cd crawler
cargo run
```

*Note: This code is currently isolated from the main search engine project as requested.*
