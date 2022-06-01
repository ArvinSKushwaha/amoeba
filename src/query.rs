use futures::StreamExt;
use http::Uri;
use std::path::PathBuf;

use tokio::sync::mpsc::{channel, Receiver, Sender};

use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct Query(pub String);

impl Query {
    pub fn new(query: impl Into<String>) -> Self {
        Query(query.into())
    }
}

impl Deref for Query {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub enum SearchResult {
    Site {
        title: String,
        url: Uri,
        excerpt: Option<String>,
    },
    File {
        title: String,
        location: PathBuf,
        excerpt: Option<String>,
    },
}

use std::{collections::BTreeMap, sync::Arc, time::Duration};

use eyre::Result;

use crate::searcher::Search;

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

pub struct QueryEngine {
    searchers: Vec<Arc<dyn Search>>,
    registry: BTreeMap<String, Arc<dyn Search>>,
    channel: (Sender<SearchResult>, Receiver<SearchResult>),
}

pub struct QueryEngineBuilder {
    searchers: Vec<(String, Box<dyn Search>)>,
}

impl QueryEngine {
    pub fn new() -> Self {
        QueryEngine {
            searchers: Vec::new(),
            registry: BTreeMap::new(),
            channel: channel(100),
        }
    }

    pub fn builder() -> QueryEngineBuilder {
        QueryEngineBuilder {
            searchers: Vec::new(),
        }
    }

    pub fn register(&mut self, modifier: &str, searcher: Arc<dyn Search + 'static>) -> Result<()> {
        if self.registry.contains_key(modifier) {
            return Err(QueryEngineError::AlreadyRegistered(modifier.to_string()))?;
        }

        self.searchers.push(searcher);
        self.registry
            .insert(modifier.to_string(), self.searchers.last().unwrap().clone());

        Ok(())
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

    pub fn reset_channels(&mut self) {
        self.channel = channel(100);
    }

    pub async fn query(&mut self, query: Query, modifier: Option<&str>, timeout: Duration) {
        use futures::stream::iter;
        let searchers: Vec<_> = if let Some(modifier) = modifier {
            if let Some(searcher) = self.registry.get(modifier) {
                std::iter::once(searcher).collect()
            } else {
                std::iter::empty().collect()
            }
        } else {
            self.searchers.iter().collect()
        };

        match tokio::time::timeout(
            timeout,
            iter(searchers)
                .for_each_concurrent(0, |s| s.query(query.clone(), self.channel.0.clone())),
        )
        .await
        {
            Ok(_) => {}
            Err(e) => {
                log::error!("{}", e);
            }
        }
    }

    pub fn recv_any(&mut self) -> Vec<SearchResult> {
        let mut recv = Vec::new();
        while let Ok(rcvd) = self.channel.1.try_recv() {
            recv.push(rcvd);
        }
        recv
    }
}

impl QueryEngineBuilder {
    pub fn new() -> Self {
        QueryEngineBuilder {
            searchers: Vec::new(),
        }
    }

    pub fn build(self) -> Result<QueryEngine> {
        let mut engine = QueryEngine::new();
        for (name, searcher) in self.searchers {
            engine.register(&name, searcher.into())?;
        }

        Ok(engine)
    }

    pub fn register(mut self, modifier: &'static str, searcher: impl Search + 'static) -> Self {
        self.searchers
            .push((modifier.to_string(), Box::new(searcher)));

        self
    }
}
