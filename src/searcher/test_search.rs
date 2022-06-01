use std::time::Duration;

use crate::{
    query::{Query, SearchResult},
    searcher::Search,
};

use futures_timer::Delay;
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub struct TestSearch;

#[async_trait::async_trait]
impl Search for TestSearch {
    async fn query(&self, query: Query, sender: Sender<SearchResult>) {
        println!("TestSearch::query: {:?}", query);
        Delay::new(Duration::from_millis(400)).await;
        sender
            .send(SearchResult::Site {
                title: format!("Test search result ({})", query.as_str()),
                url: "https://www.google.com".parse().unwrap(),
                excerpt: Some("TestSearch::query".to_string()),
            })
            .await
            .map_err(|e| {
                log::error!("Error sending search result: {}", e);
            })
            .ok();
        println!("TestSearch::query done");
    }
}
