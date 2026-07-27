mod fetcher;
mod indexer;
mod parser;
mod queue;
mod robots;

use anyhow::{Context, Result};
use clap::Parser;
use fetcher::Fetcher;
use queue::CrawlQueue;
use robots::{DomainRateLimiter, RobotsChecker};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use url::Url;

#[derive(Debug, Parser)]
#[command(name = "ayna-crawler", about = "Ayna web crawler (Phase 1)")]
struct Cli {
    /// Seed URL to start crawling from
    #[arg(long)]
    seed: String,

    /// Maximum number of pages to crawl
    #[arg(long, default_value_t = 10)]
    max_pages: usize,

    /// Number of concurrent fetch workers
    #[arg(long, default_value_t = 2)]
    concurrency: usize,

    /// Minimum delay between requests to the same domain (milliseconds)
    #[arg(long, default_value_t = 1000)]
    delay_ms: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let seed = Url::parse(&cli.seed).context("seed must be a valid URL")?;
    let allowed_host = seed
        .host_str()
        .context("seed URL must include a host")?
        .to_string();

    println!("Ayna crawler starting");
    println!("Seed: {}", cli.seed);
    println!("Host limit: {allowed_host}");
    println!("Max pages: {}", cli.max_pages);
    println!("Concurrency: {}", cli.concurrency);
    println!("Delay: {}ms per domain", cli.delay_ms);
    println!();

    let fetcher = Arc::new(Fetcher::new()?);
    let http_client = fetcher.client().clone();
    let queue = Arc::new(Mutex::new(CrawlQueue::new(&cli.seed, &allowed_host)));
    let robots_checker = Arc::new(Mutex::new(RobotsChecker::new(http_client.clone())));
    let rate_limiter = Arc::new(Mutex::new(DomainRateLimiter::new(Duration::from_millis(
        cli.delay_ms,
    ))));
    let pages_crawled = Arc::new(Mutex::new(0usize));

    let mut workers = JoinSet::new();

    loop {
        while workers.len() < cli.concurrency {
            let should_stop = {
                let queue = queue.lock().await;
                queue.visited_count() >= cli.max_pages
            };

            if should_stop {
                break;
            }

            let next_url = {
                let mut queue = queue.lock().await;
                queue.next_url()
            };

            let Some(url) = next_url else {
                break;
            };

            let fetcher = Arc::clone(&fetcher);
            let queue = Arc::clone(&queue);
            let robots_checker = Arc::clone(&robots_checker);
            let rate_limiter = Arc::clone(&rate_limiter);
            let pages_crawled = Arc::clone(&pages_crawled);
            let allowed_host = allowed_host.clone();

            workers.spawn(async move {
                if let Err(error) = crawl_page(
                    &url,
                    &allowed_host,
                    fetcher,
                    queue,
                    robots_checker,
                    rate_limiter,
                    pages_crawled,
                )
                .await
                {
                    eprintln!("{error}");
                }
            });
        }

        if workers.is_empty() {
            break;
        }

        workers.join_next().await;
    }

    while let Some(result) = workers.join_next().await {
        if let Err(error) = result {
            eprintln!("Worker failed: {error}");
        }
    }

    let queue = queue.lock().await;
    println!();
    println!(
        "Done. Crawled {} page(s). Pending in queue: {}",
        *pages_crawled.lock().await,
        queue.pending_count()
    );

    Ok(())
}

async fn crawl_page(
    url: &str,
    allowed_host: &str,
    fetcher: Arc<Fetcher>,
    queue: Arc<Mutex<CrawlQueue>>,
    robots_checker: Arc<Mutex<RobotsChecker>>,
    rate_limiter: Arc<Mutex<DomainRateLimiter>>,
    pages_crawled: Arc<Mutex<usize>>,
) -> Result<()> {
    let sleep_for = {
        let mut limiter = rate_limiter.lock().await;
        limiter.reserve_slot(url).await?
    };
    if !sleep_for.is_zero() {
        tokio::time::sleep(sleep_for).await;
    }

    let allowed = {
        let mut checker = robots_checker.lock().await;
        checker.is_allowed(url).await.unwrap_or(true)
    };

    if !allowed {
        eprintln!("Blocked by robots.txt: {url}");
        return Ok(());
    }

    let html = fetcher.fetch(url).await?;
    let parsed = parser::parse_page(&html, url, allowed_host);
    indexer::index_page(&parsed).await;

    {
        let mut queue = queue.lock().await;
        queue.enqueue_links(&parsed.links);
    }

    {
        let mut count = pages_crawled.lock().await;
        *count += 1;
    }

    Ok(())
}
