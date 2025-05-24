use futures::future::{BoxFuture, FutureExt, join_all};
use tokio::sync::Semaphore;
use std::sync::Arc;

mod models;
mod myredditapi;
use myredditapi::{build_client, get_posts};
mod readnwrite;
use readnwrite::{write_bulk_posts_to_file, write_bulk_videos_to_file, clear_folder};
// THIS IS FOR REDDIT

#[tokio::main]
async fn main() {
    let text_urls = ["https://www.reddit.com/r/MaliciousCompliance/top.json", 
    "https://www.reddit.com/r/tifu/top.json", "https://www.reddit.com/r/entitledparents/top.json"];

    let video_urls = ["https://www.reddit.com/r/funny/top.json?limit=100",];
    let amount_of_top_posts = 125usize;
    let my_client = build_client();
    

    //clear_folder("redditStories").await.unwrap();

    let sem = Arc::new(Semaphore::new(3));
    let mut all_futures: Vec<BoxFuture<'static, ()>> = Vec::new();

    // texts
    for url in &text_urls {
        let client = my_client.clone();
        let url    = url.to_string();
        let sem    = sem.clone();

        all_futures.push(async move {
            let _permit = sem.acquire().await.unwrap();
            write_bulk_posts_to_file(
                get_posts(&client, &url, amount_of_top_posts, false).await.unwrap()).await.unwrap();
        }.boxed());
    }

    // videos
    for url in &video_urls {
        let client = my_client.clone();
        let url    = url.to_string();
        let sem    = sem.clone();

        all_futures.push(async move {
            let _permit = sem.acquire().await.unwrap();
            write_bulk_videos_to_file(get_posts(&client, &url, amount_of_top_posts, true).await.unwrap()).await.unwrap();
        }.boxed());
    }

    join_all(all_futures).await;
}












