#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

mod errors;
mod gui;
mod query;
mod searcher;

use std::process::ExitCode;

use crate::{
    gui::Amoeba,
    searcher::{QueryEngine, WikipediaSearch},
};

const CONNECT_ATTEMPTS_MAX: usize = 3;

fn main() -> ExitCode {
    init_logger();

    let query_engine = {
        let qe = QueryEngine::new().register("wiki", WikipediaSearch);
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
        log::LevelFilter::Info,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Always,
    )
    .expect("Failed to initialize logger, exiting...");
}
