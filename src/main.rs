use futures::future::{BoxFuture, FutureExt, join_all};
use futures::stream::StreamExt;
use futures::pin_mut;
use tokio::sync::{Semaphore, mpsc};
use std::sync::Arc;
use std::time::Instant;
use std::collections::HashMap;
use dashmap::DashMap;

mod models;
use models::{RedditTextPost, RedditVideoPost, Child};
mod myredditapi;
use myredditapi::{build_client, stream_posts_to_channel};
mod readnwrite;
/// THIS IS FOR REDDIT'S JSON


const AMOUNT_OF_POSTS_PER_SUBREDDIT: usize = 10;
const AMOUNT_OF_CONCURRENT_WEBPAGES: usize = 2;
const POST_BUFFER_SIZE: usize = 1000;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let program_start_time = Instant::now();
    let text_urls = [
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
    readnwrite::stream_posts_to_database(post_receiver).await.unwrap()
    });

    let mut handles = Vec::new();

    // 2) spawn all of your producers, and push each `post_receiver` into `receivers`
    for url in &text_urls {
        let client = client.clone();
        let sem = sem.clone();
        let url_str = url.to_string();
        let sender = post_sender.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let posts = stream_posts_to_channel(&client, &url_str, AMOUNT_OF_POSTS_PER_SUBREDDIT, false);
            pin_mut!(posts);

            let name = extract_subreddit_name(&url_str);

            while let Some(item) = posts.next().await {
                match item {
                    Ok(child) => {
                        sender.send(child).await?;
                    }
                    Err(err) => {
                        eprintln!("  stream_posts failed for `{}`: {}", name, err);
                    }
                }
            }
            drop(sender);
            Ok::<(), anyhow::Error>(())
        });
        handles.push(handle);
    }

    drop(post_sender);
    


    for handle in handles {
        handle.await??;
    }

    
    writer.await?;

    println!("Done in {:.2?}", program_start_time.elapsed());
    Ok(())
}



fn extract_subreddit_name(url: &str) -> String {
    url.split("/r/")
       .nth(1)
       .and_then(|rest| rest.split('.').next())
       .unwrap_or("unknown")
       .to_string()
}
    