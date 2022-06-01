mod file_search;
mod utils;
mod wikipedia;

use std::{collections::BTreeMap, sync::Arc};

pub use wikipedia::WikipediaSearch;

use eyre::Result;

use crate::query::{Query, SearchResult};

pub enum QueryEngineError {
    AlreadyRegistered(String),
}

impl std::fmt::Display for QueryEngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryEngineError::AlreadyRegistered(_) => write!(f, "Query engine already registered"),
        }
    }
}

impl std::fmt::Debug for QueryEngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryEngineError::AlreadyRegistered(name) => {
                write!(f, "Query engine already registered: {}", name)
            }
        }
    }
}

impl std::error::Error for QueryEngineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            QueryEngineError::AlreadyRegistered(_) => None,
        }
    }
}

#[async_trait::async_trait]
pub trait Search {
    async fn query(&self, query: Query) -> Result<Vec<SearchResult>>;
}

pub struct QueryEngine {
    searchers: Vec<Arc<dyn Search>>,
    registry: BTreeMap<String, Arc<dyn Search>>,
}

impl QueryEngine {
    pub fn new() -> Self {
        QueryEngine {
            searchers: Vec::new(),
            registry: BTreeMap::new(),
        }
    }

    pub fn register(
        mut self,
        modifier: &'static str,
        searcher: impl Search + 'static,
    ) -> Result<Self> {
        if self.registry.contains_key(modifier) {
            return Err(QueryEngineError::AlreadyRegistered(modifier.to_string()))?;
        }

        self.searchers.push(Arc::new(searcher));
        self.registry
            .insert(modifier.to_string(), self.searchers.last().unwrap().clone());

        Ok(self)
    }

    pub fn in_registry(&self, modifier: &str) -> bool {
        self.registry.contains_key(modifier)
    }

    pub fn registry(&self) -> &BTreeMap<String, Arc<dyn Search>> {
        &self.registry
    }

    pub fn modifiers(&self) -> impl Iterator<Item = &str> {
        self.registry.keys().map(|s| s.as_str())
    }
}
