#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

mod errors;
mod query;
mod searcher;

use eframe::egui;

const CONNECT_ATTEMPTS_MAX: usize = 3;

fn main() {
    init_logger();

    let options = eframe::NativeOptions {
        min_window_size: Some((0., 0.).into()),
        initial_window_size: Some((500., 300.).into()),
        ..Default::default()
    };
    eframe::run_native(
        env!("CARGO_PKG_NAME"),
        options,
        Box::new(|_cc| Box::new(Amoeba::default())),
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

#[derive(Default)]
struct Amoeba {
    query: String,
}

impl eframe::App for Amoeba {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::style::Visuals::dark());

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Label::new("🔍"));
                ui.text_edit_singleline(&mut self.query);
            });
        });
    }
}
