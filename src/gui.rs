use crate::searcher::QueryEngine;
use eframe::{egui, Frame};
use egui::{
    CentralPanel, Color32, Context, Key, Label, Response, RichText, Style, TextEdit, TextStyle,
    TopBottomPanel, Widget,
};

pub struct Amoeba {
    query_engine: QueryEngine,
    query: String,
    modifier: Option<String>,
    last_update: Option<std::time::Instant>,
}

impl Amoeba {
    pub fn init(query_engine: QueryEngine) -> Self {
        Amoeba {
            query_engine,
            query: String::new(),
            modifier: None,
            last_update: None,
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
                    let modifier = self.modifier.as_ref().map_or("", |m| m.as_str());
                    ui.vertical(|ui| {
                        ui.add_space(2.0);
                        ui.add(Label::new(
                            RichText::new(format!("🔍 {}", modifier)).color(Color32::GOLD),
                        ));
                    });

                    if ctx.input().key_pressed(Key::Tab)
                        && self
                            .query_engine
                            .in_registry(&self.query.to_ascii_lowercase())
                    {
                        self.modifier = Some(self.query.to_ascii_lowercase());
                        self.query = String::new();
                    }

                    if ctx.input().key_pressed(Key::Backspace) && self.query.is_empty() {
                        self.modifier = None;
                    }

                    if !ctx.input().keys_down.is_empty() {
                        self.last_update = Some(std::time::Instant::now());
                    }

                    let resp = ui.add_sized(
                        [
                            ui.available_width(),
                            ui.text_style_height(&TextStyle::Monospace) + 4.0,
                        ],
                        TextEdit::singleline(&mut self.query)
                            .desired_width(f32::INFINITY)
                            .font(TextStyle::Monospace)
                            .lock_focus(true)
                            .frame(false),
                    );

                    // TODO: Deal with pressing enter
                    resp.request_focus();
                });
            });
        });

        if self.modifier.is_none() {
            CentralPanel::default().show(ctx, |ui| {
                self.query_engine
                    .modifiers()
                    .filter(|modifier| modifier.starts_with(&self.query.to_ascii_lowercase()))
                    .for_each(|modifier| {
                        ui.add(ModifierItem {
                            modifier: modifier.to_string(),
                        });
                    });
            });
        }

        frame.set_window_size(ctx.used_size());
    }
}

pub struct ModifierItem {
    modifier: String,
}

impl Widget for ModifierItem {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        ui.horizontal(|ui| {
            ui.add_sized(
                [
                    ui.available_width(),
                    ui.text_style_height(&TextStyle::Monospace) + 4.0,
                ],
                Label::new(
                    RichText::new(format!("Search using {}", self.modifier)).color(Color32::GOLD),
                ),
            )
        })
        .response
    }
}
