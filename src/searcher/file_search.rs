// use std::path::PathBuf;

// use regex::RegexSet;

// use crate::query::{Query, SearchResult};

// use super::Search;

// pub(crate) struct FileSearcher {
//     paths: Vec<PathBuf>,
//     valid_files: RegexSet,
// }

// impl FileSearcher {
//     pub fn new(paths: Vec<PathBuf>, valid_files: RegexSet) -> Self {
//         Self { paths, valid_files }
//     }
// }

// impl Search for FileSearcher {
//     fn query(&self, query: Query) -> Vec<SearchResult> {
//         unimplemented!()
//     }
// }
