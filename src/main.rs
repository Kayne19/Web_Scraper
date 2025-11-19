// use futures::future::{BoxFuture, FutureExt, join_all};
use futures::stream::StreamExt;
use futures::pin_mut;
use tokio::sync::{Semaphore, mpsc};
use std::sync::Arc;
use std::time::Instant;
// use std::collections::HashMap;
// use dashmap::DashMap;

mod models;
use models::{RedditTextPost, RedditVideoPost, Child};
mod myredditapi;
use myredditapi::{build_client, stream_posts_to_channel};
mod readnwrite;
/// THIS IS FOR REDDIT'S JSON

// Edit these to customize amount of posts, etc.
const AMOUNT_OF_POSTS_PER_SUBREDDIT: usize = 10;
const AMOUNT_OF_CONCURRENT_WEBPAGES: usize = 2;
const POST_BUFFER_SIZE: usize = 1000;

#[tokio::main]
async fn main() {
    println!("CWD: {}", std::env::current_dir().unwrap().display());
    let program_start_time = Instant::now();
    let text_urls = [
        "https://reddit.com/r/funny.json?limit=100",
        "https://reddit.com/r/MaliciousCompliance.json?limit=100",
        "https://reddit.com/r/tifu.json?limit=100",
        "https://reddit.com/r/entitledparents.json?limit=100",
        "https://reddit.com/r/the10thdentist.json?limit=100",
        "https://reddit.com/r/unpopularopinion.json?limit=100",
        "https://reddit.com/r/steam.json?limit=100",
        "https://reddit.com/r/copypasta.json?limit=100",
        "https://reddit.com/r/advice.json?limit=100",
        "https://reddit.com/r/tarkov.json?limit=100",
        "https://reddit.com/r/amitheasshole.json?limit=100",
        "https://reddit.com/r/nosleep.json?limit=100",
        "https://reddit.com/r/UnethicalLifeProTips.json?limit=100",
    ];
    let client = build_client();
    let sem = Arc::new(Semaphore::new(AMOUNT_OF_CONCURRENT_WEBPAGES));
    let (post_sender, post_receiver) = mpsc::channel::<Child>(POST_BUFFER_SIZE);

    let writer = tokio::spawn(async move {
        if let Err(streaming_posts_error) = readnwrite::stream_posts_to_database(post_receiver).await {
            eprintln!("stream_posts_to_database failed: {}", streaming_posts_error);
        }
    });

    let mut handles = Vec::new();
    for url in &text_urls {
        let client = client.clone();
        let sem = sem.clone();
        let url_str = url.to_string();
        let sender = post_sender.clone();

        let handle = tokio::spawn(async move {
            extract_data_from_urls(client, url_str, sem, sender).await
        });
        handles.push(handle);

    }
    drop(post_sender);

    for handle in handles {
        if let Err(join_err) = handle.await {
            eprintln!("A task panicked: {}", join_err);
        }
    }
    
    if writer.await.is_err() {
        eprintln!("Error writing to database");
    } else {
        println!("All posts written to database successfully.");
    }

    println!("Done in {:.2?}", program_start_time.elapsed());
}

async fn extract_data_from_urls(client: reqwest::Client, url: String, sem: Arc<Semaphore>, sender: mpsc::Sender<Child>,) -> anyhow::Result<()> {
    let _permit = sem.acquire().await?;
    let posts = stream_posts_to_channel(&client, &url, AMOUNT_OF_POSTS_PER_SUBREDDIT);
    pin_mut!(posts);

    let name = extract_subreddit_name(&url);

    while let Some(item) = posts.next().await {
        if item.is_err() {
            eprintln!("Error fetching posts from `{}`", name);
            continue;
        }
        sender.send(item.unwrap()).await?;
    }
    drop(sender);
    Ok::<(), anyhow::Error>(())
}




fn extract_subreddit_name(url: &str) -> String {
    url.split("/r/")
       .nth(1)
       .and_then(|rest| rest.split('.').next())
       .unwrap_or("unknown")
       .to_string()
}
    