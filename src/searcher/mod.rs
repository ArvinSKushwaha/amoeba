mod file_search;
mod test_search;
mod utils;
mod wikipedia;

pub use test_search::TestSearch;
use tokio::sync::mpsc::Sender;
pub use wikipedia::WikipediaSearch;

use crate::query::{Query, SearchResult};

#[async_trait::async_trait]
pub trait Search: Send + Sync {
    async fn query(&self, query: Query, sender: Sender<SearchResult>);
}
