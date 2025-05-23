use crate::models::{WebPage, Child};
use reqwest::{Client, Error};
use tokio::time::{sleep, Duration};



pub fn build_client() -> Client{
    let my_client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
        .build()
        .unwrap();

    my_client
}


pub async fn get_text_posts(current_client: &reqwest::Client, current_url: &str, amount_of_top_posts: usize) -> Result<Vec<Child>, reqwest::Error> {
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
        children.push(wrapper.data);
        i += 1;
    }
    
    Ok(children)
}