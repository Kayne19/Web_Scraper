use crate::models::{WebPage, Child};
use reqwest::{Client, Error};
use tokio::time::{sleep, Duration};
use async_stream::try_stream;
use futures::stream::Stream;
use futures::pin_mut;
use futures::StreamExt;
use rand::Rng;



pub fn build_client() -> Client{
    let my_client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
        .cookie_store(false)
        .build()
        .unwrap();

    my_client
}

pub fn stream_posts_to_channel<'a>(client: &'a reqwest::Client, url: &'a str, total_number_of_posts_to_grab: usize, videos_only: bool) -> impl Stream<Item = Result<Child, Error>> + 'a {
    try_stream! {
        let mut current_url = url.to_string();
        let mut posts_grabbed_so_far = 0;

        while posts_grabbed_so_far < total_number_of_posts_to_grab {
            let start = std::time::Instant::now();
            let website_response = client.get(current_url.clone()).send().await?;
            let listing: WebPage = website_response.error_for_status()?.json().await?;
            println!("HTTP request took: {:?}", start.elapsed());

            let posts = fetch_posts(&listing, videos_only);
            pin_mut!(posts);
            while let Some(res) = posts.next().await {
                let current_post = res?;
                posts_grabbed_so_far += 1;
                yield current_post;
                if posts_grabbed_so_far >= total_number_of_posts_to_grab {return;}
            }
            
            if let Some(next_page_id) = listing.data.after.as_ref() {
                let next_url = format!("{}&after={}", url, next_page_id);
                current_url = next_url;
            } else {
                println!("No more pages to fetch.");
                break;
            }

            println!("Fetched {} posts so far, waiting for next page...", posts_grabbed_so_far);
            sleep(Duration::from_millis(3192)).await;
            
        }
    }
}

pub fn fetch_posts<'a>(web_page: &'a WebPage, videos_only: bool) -> impl Stream<Item = Result<Child, Error>> + 'a {
    try_stream! {
        for wrapper in web_page.data.children.iter() {
            let post = wrapper.data.clone();
            if post.title.to_lowercase().contains("update") || post.is_video != videos_only {
                continue;
            }
        yield post
        }
    }
}