use crate::models::{WebPage, Child};
use reqwest::{Client, Error};
use tokio::time::{sleep, Duration};
use async_stream::try_stream;
use futures::stream::Stream;



pub fn build_client() -> Client{
    let my_client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
        .cookie_store(false)
        .build()
        .unwrap();

    my_client
}

pub fn stream_posts<'a>(client: &'a reqwest::Client, url: &'a str, total_number_of_posts_to_grab: usize, videos_only: bool) -> impl Stream<Item = Result<Child, Error>> + 'a {
    try_stream! {
        let mut current_url = url.to_string();
        

        let mut posts_grabbed_so_far = 0;
        while posts_grabbed_so_far < total_number_of_posts_to_grab {

            let website_response = client.get(current_url.clone()).send().await?;
            let listing: WebPage = website_response.error_for_status()?.json().await?;

            println!("Fetched {} items from {}", listing.data.children.len(), current_url);
            for wrapper in listing.data.children {
                if posts_grabbed_so_far >= total_number_of_posts_to_grab {
                    break;
                }
                let post = wrapper.data;
                if post.title.to_lowercase().contains("update") || post.is_video != videos_only {
                    continue;
                }
                posts_grabbed_so_far += 1;
                yield post
            }
            if let Some(after) = listing.data.after.as_ref() {
                let next_url = format!("{}&after={}", url, after);
                println!("Fetching next page: {}", next_url);
                current_url = next_url;
            } else {
                println!("No more pages to fetch.");
                break; // Stop the loop
            }
            // Sleep for a short duration to avoid hitting the API too hard
            sleep(Duration::from_millis(10000)).await;

            
        }
    }
}


pub async fn get_posts(current_client: &reqwest::Client, current_url: &str, amount_of_top_posts: usize, videos_only: bool) -> Result<Vec<Child>, reqwest::Error> {
    let response = current_client
        .get(current_url)
        .send()
        .await?;
    let status = response.status();
    eprintln!("HTTP {}", status);


    let listing: WebPage = response.error_for_status()?.json::<WebPage>().await?;
    let child_wrapper_vector = listing.data.children;


    let mut i = 0;
    let mut children = Vec::new();
    for wrapper in child_wrapper_vector {
        if i >= amount_of_top_posts {
            break;
        }

        if wrapper.data.title.to_lowercase().contains("update") || wrapper.data.is_video != videos_only {
            println!("Post failed update/video check: {}", wrapper.data.title);
            continue;
        }

        children.push(wrapper.data);
        i += 1;
    }

    Ok(children)
}