use crate::models;
use models::{RedditTextPost, RedditVideoPost, WebPage, ChildWrapper, Child, ListingData};
use std::path::PathBuf;
use tokio::fs::File;
use futures::future::{BoxFuture, FutureExt, join_all};
use tokio::sync::Semaphore;
use tokio::sync::mpsc::{UnboundedReceiver, Receiver};
use std::sync::Arc;
use tokio::io::{self, AsyncWriteExt};
use serde::Serialize;
use std::time::Duration;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::QueryBuilder;
use std::str::FromStr;

const BATCH_SIZE: usize = 100;

//pub async fn write_to_output_db
pub async fn stream_posts_to_database(mut incoming_list: Receiver<Child>) -> sqlx::Result<()> {
    let mut connect_opts = SqliteConnectOptions::from_str("sqlite://./mydatabase.db")?
    .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .connect_with(connect_opts)
        .await?; 
    sqlx::query("PRAGMA journal_mode = WAL;").execute(&pool).await?;
    sqlx::query("PRAGMA synchronous = NORMAL;").execute(&pool).await?;
    sqlx::query("PRAGMA temp_store = MEMORY;").execute(&pool).await?;
    sqlx::query("PRAGMA cache_size = -64000;").execute(&pool).await?;
    // Initialize schema (run this once at startup)
    sqlx::query(r#"
    CREATE TABLE IF NOT EXISTS posts (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        title       TEXT    NOT NULL,
        subreddit   TEXT    NOT NULL,
        body        TEXT,
        is_video    INTEGER NOT NULL DEFAULT 0,
        UNIQUE(subreddit, title)
    );
    "#)
    .execute(&pool)
    .await?;

    let mut posts_written = 0;
    let mut batch: Vec<Child> = Vec::new();
    while let Some(post) = incoming_list.recv().await {
        if batch.len() >= BATCH_SIZE {

            let mut query_buider = QueryBuilder::new("INSERT INTO posts (title, subreddit, body, is_video) ");
            query_buider.push_values(batch.iter(), |mut b, post| {
                b.push_bind(&post.title)
                .push_bind(&post.subreddit)
                .push_bind(&post.body)
                .push_bind(post.is_video as i64);
            });
            query_buider.push(" ON CONFLICT(subreddit, title) DO NOTHING");
            let query = query_buider.build();
            query.execute(&pool).await?;
            posts_written += batch.len();
            batch.clear();
            println!("Batch written. Wrote {} posts in total.", posts_written);
        }
        batch.push(post);
    }
    if !batch.is_empty() {
        let mut query_buider = QueryBuilder::new("INSERT INTO posts (title, subreddit, body, is_video) ");
        query_buider.push_values(batch.iter(), |mut b, post| {
            b.push_bind(&post.title)
            .push_bind(&post.subreddit)
            .push_bind(&post.body)
            .push_bind(post.is_video as i64);
        });
        query_buider.push(" ON CONFLICT(subreddit, title) DO NOTHING");
        let query = query_buider.build();
        query.execute(&pool).await?;
        posts_written += batch.len();
    }


    println!("Database writer finished. Wrote {} posts.", posts_written);
    let (total_chars,): (i64,) = sqlx::query_as("SELECT SUM(LENGTH(title) + LENGTH(body) + LENGTH(subreddit)) FROM posts")
        .fetch_one(&pool)
        .await?;

    println!("Total characters in DB: {}", total_chars);
    Ok(())

}

fn clean_title(title: &str) -> String {
    title.replace(' ', "_").chars().filter(|c| c.is_alphanumeric() || *c == '_').collect::<String>()
}