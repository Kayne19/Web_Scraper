use crate::models;
use models::{RedditTextPost, RedditVideoPost, WebPage, ChildWrapper, Child, ListingData};
use std::path::PathBuf;
use tokio::fs::File;
use futures::future::{BoxFuture, FutureExt, join_all};
use tokio::sync::Semaphore;
use tokio::sync::mpsc::UnboundedReceiver;
use std::sync::Arc;
use tokio::io::{self, AsyncWriteExt};
use serde::Serialize;

//pub async fn write_to_output_file



pub async fn stream_posts_to_file(mut incoming_list: Vec<UnboundedReceiver<Vec<u8>>>,output_path: &str,) -> io::Result<()> {
    let mut file = File::create(output_path).await?;
    file.write_all(b"{").await?; // start root object

    // for each subreddit’s channel, drain it completely before moving on
    for mut incoming in incoming_list.drain(..) {
        while let Some(bytes) = incoming.recv().await {
            file.write_all(&bytes).await?;
        }
    }

    file.write_all(b"}").await?; // close root object
    file.flush().await?;
    Ok(())
}



pub fn open_subarray(subreddit: &str, is_first: bool) -> Vec<u8> {
    // build the prefix string
    let s = if is_first {
        // no leading comma
        format!(r#""{}":["#, subreddit)
    } else {
        // comma before each subsequent subreddit
        format!(r#","{}":["#, subreddit)
    };
    s.into_bytes()
}

pub fn serialize_post<T: Serialize>(post: &T, is_first: bool) -> Vec<u8> {
    let json = serde_json::to_vec(post)
        .expect("serialization should never fail");
    if is_first {
        json
    } else {
        let mut v = Vec::with_capacity(1 + json.len());
        v.push(b',');
        v.extend(json);
        v
    }
}

fn clean_title(title: &str) -> String {
    title.replace(' ', "_").chars().filter(|c| c.is_alphanumeric() || *c == '_').collect::<String>()
}