use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/* 
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]  // this adds a "type" field for JSON discrimination
pub enum RedditPost {
    Text(RedditTextPost),
    Video(RedditVideoPost)
}
*/

#[derive(Serialize, Deserialize)]
pub struct RedditPostsMap {
    /// String is the subreddit name
    pub text_posts:  HashMap<String, Vec<RedditTextPost>>,
    pub video_posts: HashMap<String, Vec<RedditVideoPost>>,
}

#[derive(Serialize, Deserialize)]
pub struct RedditTextPost {
    pub title: String,
    pub body: String,
    pub subreddit: String,
    pub unused_content: bool,
}

#[derive(Serialize, Deserialize)]
pub struct RedditVideoPost {
    pub title: String,
    pub video_link: String,
    pub sound_link: Option<String>,
    pub subreddit: String,
    pub unused_content: bool,
}

#[derive(Deserialize)]
pub struct WebPage {
    pub data: ListingData,
}

#[derive(Deserialize)]
pub struct ListingData {
    pub children: Vec<ChildWrapper>,
}

#[derive(Deserialize)]
pub struct ChildWrapper {
    pub data: Child,
}

#[derive(Serialize, Deserialize)]
pub struct Child {
    pub title: String,
    #[serde(rename = "selftext")]
    pub body: String,
    pub subreddit: String,
    pub secure_media: Option<SecureMediaWrapper>,
    pub is_video: bool
  
}

#[derive(Serialize, Deserialize)]
pub struct SecureMediaWrapper {
    pub reddit_video: Option<reddit_secure_media>,
}

#[derive(Serialize, Deserialize)]
pub struct reddit_secure_media {
    #[serde(rename = "fallback_url")]
    pub video_link: String,
    #[serde(rename = "hls_url")]
    pub audio_url: Option<String>,
    
}