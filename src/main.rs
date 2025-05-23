use futures::future::join_all;
use tokio::sync::Semaphore;
use std::sync::Arc;

mod models;
mod myredditapi;
use myredditapi::{build_client, get_text_posts};
mod readnwrite;
use readnwrite::{write_bulk_posts_to_file, clear_folder};
// THIS IS FOR REDDIT

#[tokio::main]
async fn main() {
    let urls = ["https://www.reddit.com/r/MaliciousCompliance/top.json", 
    "https://www.reddit.com/r/tifu/top.json", "https://www.reddit.com/r/entitledparents/top.json"];

    let amount_of_top_posts = 12usize;
    let my_client = build_client();
    

    //clear_folder("redditStories").await.unwrap();

    let sem = Arc::new(Semaphore::new(3));
    let tasks = urls.iter().map(|&url| {
        let client = my_client.clone();
        let url    = url.to_string();
        let sem    = sem.clone();

        async move {
            let _permit = sem.acquire().await.unwrap();
            //sleep(Duration::from_millis(100)).await;

            write_bulk_posts_to_file(get_text_posts(&client, &url, amount_of_top_posts).await.unwrap()).await.unwrap();
            
        }
   });
    join_all(tasks).await;
}










