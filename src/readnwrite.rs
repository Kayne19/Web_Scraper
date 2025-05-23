use crate::models;
use models::{RedditTextPost, WebPage, ChildWrapper, Child, ListingData};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{self, AsyncWriteExt};



pub async fn write_bulk_posts_to_file(posts: Vec<Child>) -> io::Result<()> {

    for post in posts {
        write_text_post_to_file(&post.title, &post.body, &post.subreddit).await?;
    }
    Ok(())
}

async fn write_text_post_to_file(title: &str, body: &str, subreddit: &str) -> io::Result<()> {
    let the_story = RedditTextPost { title: title.to_string(), body: body.to_string(), subreddit: subreddit.to_string(), unused_content: true };

    let safe_title = title
        .replace(' ', "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>();
    let filename = format!("{}.json", safe_title);

    let json = serde_json::to_string_pretty(&the_story)?;

    tokio::fs::create_dir_all("redditStories").await?;
    
    let mut path = PathBuf::from("redditStories");
    path.push(the_story.subreddit);
    tokio::fs::create_dir_all(&path).await?;
    path.push(&filename);

    
    
    if tokio::fs::metadata(&path).await.is_ok() {
    println!("File already exists: {}", path.display());
    return Ok(());
}

    let mut file = File::create(path.clone()).await?;
    file.write_all(json.as_bytes()).await?;

    Ok(())
}

pub async fn clear_folder(path: &str) -> io::Result<()> {
    tokio::fs::remove_dir_all(path).await.ok();
    tokio::fs::create_dir_all(path).await?;
    Ok(())
}