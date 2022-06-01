#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

mod errors;
mod gui;
mod query;
mod searcher;

use std::process::ExitCode;

use crate::{
    gui::Amoeba,
    query::QueryEngine,
    searcher::{TestSearch, WikipediaSearch},
};

const CONNECT_ATTEMPTS_MAX: usize = 3;
const QUERY_DELAY_MS: u64 = 1000;
const QUERY_TIMEOUT_MS: u64 = 2000;

fn main() -> ExitCode {
    init_logger();

    let query_engine = {
        let qe = QueryEngine::builder()
            .register("test", TestSearch)
            .register("wiki", WikipediaSearch)
            .build();

        match qe {
            Ok(qe) => qe,
            Err(e) => {
                log::error!("{}", e);
                return ExitCode::FAILURE;
            }
        }
    };

    let options = eframe::NativeOptions {
        min_window_size: Some((0., 0.).into()),
        initial_window_size: Some((700., 10.).into()),
        decorated: false,
        ..Default::default()
    };

    eframe::run_native(
        env!("CARGO_PKG_NAME"),
        options,
        Box::new(|_cc| Box::new(Amoeba::init(query_engine))),
    );
}

fn init_logger() {
    simplelog::TermLogger::init(
        log::LevelFilter::Trace,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Always,
    )
    .expect("Failed to initialize logger, exiting...");
}
