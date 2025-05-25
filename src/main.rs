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
use myredditapi::{build_client, get_posts, stream_posts};
mod readnwrite;
// THIS IS FOR REDDIT

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let start = Instant::now();
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
    let amount = 512;
    let client = build_client();
    let sem    = Arc::new(Semaphore::new(3));

    // 1) prepare to collect all producer handles and their receivers
    let mut handles = Vec::new();
    let mut receivers = Vec::new();
    let mut is_first_subreddit = true;

    // 2) spawn all of your producers, and push each `post_receiver` into `receivers`
    for url in &text_urls {
        let client = client.clone();
        let sem = sem.clone();
        let url_str = url.to_string();

        let first_sub = is_first_subreddit;
        is_first_subreddit = false;

        let (sender, post_receiver) = mpsc::unbounded_channel::<Vec<u8>>();
        receivers.push(post_receiver);

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let posts = stream_posts(&client, &url_str, amount, false);
            pin_mut!(posts);

            let name = extract_subreddit_name(&url_str);
            sender.send(readnwrite::open_subarray(&name, first_sub)).unwrap();

            eprintln!("Fetching posts from {}:", name);
            let mut is_first_post = true;
            while let Some(item) = posts.next().await {
                match item {
                    Ok(child) => {
                        let chunk = readnwrite::serialize_post(&child, is_first_post);
                        sender.send(chunk).unwrap();
                        is_first_post = false;
                    }
                    Err(err) => {
                        eprintln!("  stream_posts failed for `{}`: {}", name, err);
                    }
                }
            }
            // close this subreddit’s array
            sender.send(vec![b']']).unwrap();
            // dropping the sender here will close *this* channel
            drop(sender);

            Ok::<(), anyhow::Error>(())
        });
        handles.push(handle);
    }

    // 3) now that you have *all* your receivers, spawn the writer
    let writer = tokio::spawn(
        readnwrite::stream_posts_to_file(receivers, "all_posts.json")
    );

    // 4) wait for all producers to finish (they’ll each drop their sender when done)
    for h in handles {
        h.await??;
    }

    // 5) by now every sender is dropped ⇒ writer’s streams will all close ⇒ it can finish
    writer.await??;

    println!("Done in {:.2?}", start.elapsed());
    Ok(())
}


fn extract_subreddit_name(url: &str) -> String {
    url.split("/r/")
       .nth(1)
       .and_then(|rest| rest.split('.').next())
       .unwrap_or("unknown")
       .to_string()
}
    