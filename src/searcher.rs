mod file_search;
mod utils;
mod wikipedia;

pub use wikipedia::WikipediaSearch;

use eyre::Result;

use crate::query::{Query, SearchResult};

pub trait Search {
    fn query(&self, query: Query) -> Result<Vec<SearchResult>>;
}
