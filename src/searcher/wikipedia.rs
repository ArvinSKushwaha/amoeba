use http::{StatusCode, Uri};
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use urlencoding::encode;

use hyper::{Body, Client, Request};

use crate::{
    query::{Query, SearchResult},
    CONNECT_ATTEMPTS_MAX,
};

use super::Search;

lazy_static::lazy_static! {
    static ref USER_AGENT: String = format!("{}/{} ({})", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_AUTHORS"));
    static ref WIKIPEDIA_URL: String = "https://en.wikipedia.org".to_string();
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

#[async_trait::async_trait]
impl Search for WikipediaSearch {
    async fn query(&self, query: Query, sender: Sender<SearchResult>) {
        let client = Client::builder().build::<_, Body>(HttpsConnector::new());
        let encoded_query = encode(&query);
        let request = match Request::builder()
            .method("GET")
            .header("User-Agent", USER_AGENT.as_str())
            .uri(format!(
                "{}/w/rest.php/v1/search/page?limit={}&q={}",
                WIKIPEDIA_URL.as_str(),
                WIKIPEDIA_SEARCH_RESULTS,
                encoded_query
            ))
            .body(Body::empty())
        {
            Ok(request) => request,
            Err(err) => {
                log::error!("Could not create request: {}", err);
                return;
            }
        };

        let craft_request = || {
            let mut req = Request::builder().method(request.method());

            for (key, value) in request.headers() {
                req = req.header(key, value);
            }

            req.uri(request.uri()).body(Body::empty()).unwrap()
        };

        log::info!("Querying @ Wikipedia URL: {}", craft_request().uri());

        let mut response = None;

        for _ in 0..CONNECT_ATTEMPTS_MAX {
            match client.request(craft_request()).await {
                Ok(resp) => {
                    if resp.status() == StatusCode::TOO_MANY_REQUESTS {
                        std::thread::sleep(Duration::from_secs(1));
                    } else if resp.status() == StatusCode::OK {
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
            log::info!("Failed to fetch Wikipedia search results, aborting search.");
        }

        let response = response.unwrap();
        let json = match hyper::body::to_bytes(response.into_body()).await {
            Ok(bytes) => bytes,
            Err(err) => {
                log::error!("Could not read Wikipedia response: {}", err);
                return;
            }
        }
        .to_vec();
        let json = String::from_utf8_lossy(&json);

        let pages = match serde_json::from_str::<Pages>(&json) {
            Ok(pages) => pages,
            Err(err) => {
                log::error!("Could not parse Wikipedia search results: {}", err);
                return;
            }
        }
        .pages;

        pages
            .iter()
            .filter_map(|arr| {
                let title = arr.title.clone();
                let url: Option<Uri> = format!("https://en.wikipedia.org/wiki/{}", &arr.key)
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
            .for_each(|result| {
                let sender = sender.clone();
                tokio::spawn(async move { sender.send(result).await });
            })
    }
}
