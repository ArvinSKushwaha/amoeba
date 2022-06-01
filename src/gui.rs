use std::time::Duration;

use crate::{
    query::{Query, QueryEngine, SearchResult},
    QUERY_DELAY_MS, QUERY_TIMEOUT_MS,
};
use eframe::{egui, Frame};
use egui::{
    Color32, Context, Key, Label, Response, RichText, Sense, Style, TextEdit, TextStyle,
    TopBottomPanel, Widget,
};
use tokio::runtime::{Builder, Runtime};

pub struct Amoeba {
    query_engine: QueryEngine,
    query: String,
    modifier: Option<String>,
    last_update: Option<std::time::Instant>,
    results: Vec<SearchResult>,
    rt: Runtime,
}

impl Amoeba {
    pub fn init(query_engine: QueryEngine) -> Self {
        Amoeba {
            query_engine,
            query: String::new(),
            modifier: None,
            last_update: None,
            results: Vec::new(),
            rt: Builder::new_multi_thread().enable_all().build().unwrap(),
        }
    }
}

impl eframe::App for Amoeba {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let mut style = Style::default();
        style.visuals.dark_mode = true;
        style.override_text_style = Some(TextStyle::Monospace);
        style
            .text_styles
            .get_mut(&TextStyle::Monospace)
            .unwrap()
            .size = 20.0;
        ctx.set_style(style);

        if ctx.input().key_pressed(Key::Escape) {
            frame.quit();
        }

        TopBottomPanel::top("Top Panel").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.add_space(2.0);
                        let modifier = self.modifier.as_ref().map_or("", |m| m.as_str());
                        ui.horizontal(|ui| {
                            ui.label("🔍");
                            ui.add(Label::new(RichText::new(modifier).color(Color32::GOLD)));
                        });
                    });

                    let resp = ui.add_sized(
                        [
                            ui.available_width(),
                            ui.text_style_height(&TextStyle::Monospace) + 4.0,
                        ],
                        TextEdit::singleline(&mut self.query)
                            .desired_width(f32::INFINITY)
                            .font(TextStyle::Monospace)
                            .lock_focus(true)
                            .frame(false)
                            .id_source("query_edit"),
                    );

                    if resp.changed() {
                        self.results.clear();
                        self.query_engine.reset_channels();
                        self.last_update = Some(std::time::Instant::now());
                    }

                    if ctx.input().key_pressed(Key::Tab)
                        && self
                            .query_engine
                            .in_registry(&self.query.to_ascii_lowercase())
                    {
                        self.modifier = Some(self.query.to_ascii_lowercase());
                        self.query = String::new();

                        self.results.clear();
                        self.query_engine.reset_channels();
                        self.last_update = Some(std::time::Instant::now());
                    }

                    if ctx.input().key_pressed(Key::Backspace) && self.query.is_empty() {
                        self.modifier = None;

                        self.results.clear();
                        self.query_engine.reset_channels();
                        self.last_update = Some(std::time::Instant::now());
                    }

                    // Catch focus
                });
            });
        });

        TopBottomPanel::top("modifiers").show(ctx, |ui| {
            if self.modifier.is_none() {
                let modifiers = self
                    .query_engine
                    .modifiers()
                    .filter(|modifier| modifier.starts_with(&self.query.to_ascii_lowercase()))
                    .map(|modifier| modifier.to_string())
                    .collect::<Vec<_>>();

                modifiers.iter().for_each(|modifier| {
                    if ui
                        .add(ModifierItem {
                            modifier: modifier.to_string(),
                        })
                        .clicked()
                    {
                        self.modifier = Some(modifier.to_string());

                        self.results.clear();
                        self.query_engine.reset_channels();
                        self.last_update = Some(std::time::Instant::now());
                    }
                });
            }
            self.results.iter().for_each(|result| {
                ui.add(ResultItem {
                    result: result.clone(),
                });
            });
        });

        frame.set_window_size(ctx.used_size());

        self.results.extend(self.query_engine.recv_any());

        if let Some(last_update) = self.last_update {
            if last_update.elapsed() > Duration::from_millis(QUERY_DELAY_MS)
                && !self.query.is_empty()
            {
                self.results.clear();
                self.query_engine.reset_channels();
                self.last_update = None;

                self.rt.block_on(self.query_engine.query(
                    Query::new(self.query.clone()),
                    self.modifier.as_deref(),
                    Duration::from_millis(QUERY_TIMEOUT_MS),
                ));
            }
        }

        ctx.request_repaint();
    }
}

pub struct ModifierItem {
    modifier: String,
}

pub struct ResultItem {
    result: SearchResult,
}

impl Widget for ModifierItem {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        ui.add_sized(
            [
                ui.available_width(),
                ui.text_style_height(&TextStyle::Monospace) + 4.0,
            ],
            Label::new(
                RichText::new(format!("Search using {}", self.modifier)).color(Color32::GOLD),
            )
            .sense(Sense::click()),
        )
    }
}

impl Widget for ResultItem {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let result = &self.result;
        match result {
            SearchResult::File { .. } => {
                unimplemented!()
            }
            SearchResult::Site {
                title,
                url,
                excerpt,
            } => {
                ui.vertical(|ui| {
                    ui.hyperlink_to(RichText::new(title), url.to_string());
                    if let Some(excerpt) = excerpt {
                        ui.label(RichText::new(excerpt));
                    }
                })
                .response
            }
        }
    }
}
