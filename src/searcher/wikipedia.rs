use std::{error::Error, time::Duration};

use eyre::Result;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::{
    query::{Query, SearchResult},
    CONNECT_ATTEMPTS_MAX,
};

use super::Search;

lazy_static::lazy_static! {
    static ref USER_AGENT: String = format!("{}/{} ({})", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_AUTHORS"));
    static ref WIKIPEDIA_URL: Url = "https://en.wikipedia.org".parse().expect("Invalid Wikipedia URL");
}

const WIKIPEDIA_SEARCH_RESULTS: &str = "5";

pub struct WikipediaSearch;

#[derive(Serialize, Deserialize)]
struct Pages {
    pages: Vec<Article>,
}

#[derive(Serialize, Deserialize)]
struct Article {
    id: usize,
    key: String,
    title: String,
    excerpt: String,
    matched_title: Option<String>,
    description: Option<String>,
    thumbnail: Option<Thumbnail>,
}

#[derive(Serialize, Deserialize)]
struct Thumbnail {
    mimetype: String,
    size: Option<usize>,
    width: usize,
    height: usize,
    duration: Option<usize>,
    url: String,
}

struct WikipediaSearchError<E> {
    message: String,
    context: Option<E>,
}

impl<E> std::fmt::Display for WikipediaSearchError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl<E: std::fmt::Debug> std::fmt::Debug for WikipediaSearchError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(context) = &self.context {
            write!(f, ": {:?}", context)?;
        }
        Ok(())
    }
}

struct NonError;
impl std::fmt::Display for NonError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl std::fmt::Debug for NonError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl std::error::Error for NonError {}

impl<E: Error> Error for WikipediaSearchError<E> {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.context.as_ref().map(|e| e as _)
    }
}

impl Search for WikipediaSearch {
    fn query(&self, query: Query) -> Result<Vec<SearchResult>> {
        let encoded_query = encode(&query);
        let mut url = WIKIPEDIA_URL.clone();
        url.set_path("/w/rest.php/v1/search/page");
        url.query_pairs_mut()
            .append_pair("q", &encoded_query)
            .append_pair("limit", WIKIPEDIA_SEARCH_RESULTS);

        log::info!("Querying @ Wikipedia URL: {}", url);

        let mut response = None;

        for _ in 0..CONNECT_ATTEMPTS_MAX {
            match reqwest::blocking::get(url.clone()) {
                Ok(resp) => {
                    if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        std::thread::sleep(Duration::from_secs(1));
                    } else if resp.status() == reqwest::StatusCode::OK {
                        std::thread::sleep(Duration::from_millis(500));
                        response.replace(resp);
                        break;
                    } else {
                        log::info!(
                            "Failed to fetch Wikipedia search results: {}. Retrying...",
                            resp.status()
                        );
                    }
                }
                Err(e) => {
                    log::info!(
                        "Failed to fetch Wikipedia search results: {}. Retrying...",
                        e
                    );
                }
            };
        }

        if response.is_none() {
            return Err(WikipediaSearchError::<NonError> {
                message: "Failed to fetch Wikipedia search results, aborting search.".to_string(),
                context: None,
            })?;
        }

        let response = response.unwrap();
        let json = response.json::<Pages>();

        if let Err(e) = json {
            return Err(WikipediaSearchError {
                message: "Failed to parse Wikipedia search results, aborting search".to_string(),
                context: Some(e),
            })?;
        }

        let json = json.unwrap().pages;

        Ok(json
            .iter()
            .filter_map(|arr| {
                let title = arr.title.clone();
                let url: Option<Url> = format!("https://en.wikipedia.org/wiki/{}", &arr.key)
                    .parse()
                    .ok();
                let excerpt = Some(arr.excerpt.clone());

                match url {
                    None => {
                        log::info!(
                            "Failed to parse Wikipedia search result URL with key \"{}\", skipping",
                            &arr.key
                        );
                        None
                    }
                    Some(url) => Some(SearchResult::Site {
                        title,
                        url,
                        excerpt,
                    }),
                }
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_wikipedia() {
        use super::*;

        let search = WikipediaSearch.query(Query::new("test"));
        println!("{:#?}", search);
    }
}
