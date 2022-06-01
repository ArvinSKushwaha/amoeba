use std::path::PathBuf;

use reqwest::Url;

use std::ops::Deref;

pub struct Query(pub String);

impl Query {
    pub fn new(query: impl Into<String>) -> Self {
        Query(query.into())
    }
}

impl Deref for Query {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub enum SearchResult {
    Site {
        title: String,
        url: Url,
        excerpt: Option<String>,
    },
    File {
        title: String,
        location: PathBuf,
        excerpt: Option<String>,
    },
}
