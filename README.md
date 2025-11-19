# Reddit Webscraper

A Rust-based asynchronous Reddit data collector that streams posts from multiple subreddits and stores them in a local SQLite database.

## Overview
This project demonstrates:
- Asynchronous networking with **Tokio** and **Reqwest**
- Streaming pagination using **async-stream**
- Concurrency control with **Semaphores** and bounded channels
- Persistent storage using **SQLx** and SQLite
- Modular Rust code structure with clear separation between data models, API calls, and database handling

It was built as a learning project to collect Reddit data efficiently as well as to learn the Rust language.

## Project Structure
reddit_webscraper/
├── Cargo.toml           # Project dependencies and metadata
├── src/
│   ├── main.rs          # Entry point and async orchestration
│   ├── models.rs        # Data structures for Reddit JSON
│   ├── myredditapi.rs   # Functions for API calls and pagination
│   └── readnwrite.rs    # Database interaction logic

## How to Run

1. **Install Rust (if you don’t already have it):**
   https://rustup.rs

2. **Clone or extract the project**, then open the folder in VS Code or a terminal.

3. **Build the project:**
   ```bash
   cargo build
   ```

4. **Run the scraper:**
   ```bash
   cargo run
   ```

   Optionally, you can specify a custom database path:
   ```bash
   DATABASE_URL="sqlite://./data/reddit.db" cargo run
   ```

5. The scraped data will be stored in a local SQLite database file (default: `mydatabase.db`).

## Notes
- Make sure the folder where the database is created is writable.

## Dependencies
- tokio
- reqwest
- async-stream
- serde, serde_json
- sqlx
- anyhow

## Future Improvements

This project is still a work in progress. Planned updates include:
- Support for videos and images
- Improving error propagation
- Clean up deprecated imports
- Adding retry and rate-limiting mechanisms for API calls
- Adding configuration options (CLI arguments or environment variables)
- Writing unit and integration tests for stability