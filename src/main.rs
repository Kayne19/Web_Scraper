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

    let video_urls = ["https://www.reddit.com/r/funny/top.json",];
    let amount_of_top_posts = 55usize;
    let my_client = build_client();
    

    //clear_folder("redditStories").await.unwrap();

    let sem = Arc::new(Semaphore::new(3));
    let mut all_futures: Vec<BoxFuture<'static, ()>> = Vec::new();


    let mut iter = text_urls.iter();
    let mut text_futures = Vec::new();
    while let Some(&url) = iter.next() {
        text_futures.push({
            let client = my_client.clone();
            let url    = url.to_string();
            let sem    = sem.clone();
            async move {
            let _permit = sem.acquire().await.unwrap();
            //sleep(Duration::from_millis(100)).await;

            write_bulk_posts_to_file(get_posts(&client, &url, amount_of_top_posts, false).await.unwrap()).await.unwrap();
            
        }
        });
    }
join_all(text_futures).await;


    let mut video_iter = video_urls.iter();
    let mut video_futures = Vec::new();
    while let Some(&url) = video_iter.next() {
        video_futures.push({
            let client = my_client.clone();
            let url    = url.to_string();
            let sem    = sem.clone();
            async move {
            let _permit = sem.acquire().await.unwrap();
            //sleep(Duration::from_millis(100)).await;

            write_bulk_videos_to_file(get_posts(&client, &url, amount_of_top_posts, true).await.unwrap()).await.unwrap();
            
        }
        });
    }
join_all(video_futures).await;


}











