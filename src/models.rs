use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct RedditTextPost {
    pub title: String,
    pub body: String,
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

#[derive(Deserialize)]
pub struct Child {
    pub title: String,
    #[serde(rename = "selftext")]
    pub body: String,
    pub subreddit: String,
}