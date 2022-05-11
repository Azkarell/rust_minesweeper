use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use bevy_egui::egui::{Align2, Color32, FontData, FontDefinitions, FontFamily, Frame, Grid, Rgba, RichText, Slider};
use bevy_egui::egui::style::Margin;
use iyes_loopless::state::NextState;
use crate::{FieldGenerationOptions, GameState};

#[derive(Clone)]
pub struct UiState {
    pub mines: usize,
    pub rows: usize,
    pub columns: usize,
    pub seed: String,
}

impl From<UiState> for FieldGenerationOptions {
    fn from(o: UiState) -> Self {
        let mut hasher = DefaultHasher::new();
        o.seed.hash(&mut hasher);
        let seed = hasher.finish();
        FieldGenerationOptions {
            mine_count: o.mines,
            height: o.rows,
            width: o.columns,
            seed,
        }
    }
}



#[derive(Component, Debug, Default)]
pub(crate) struct Overlay;

pub(crate) struct TitleText(pub String, pub Color);

pub(crate) fn init_visuals(mut egui_ctx: ResMut<EguiContext>) {
    egui_ctx.ctx_mut().set_visuals(egui::Visuals {
        window_rounding: 0.15.into(),
        ..default()
    });
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert("pixelated".to_string(), FontData::from_static(include_bytes!("../assets/fonts/pixelated_arial_regular_11.ttf")));

    fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "pixelated".to_string());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("pixelated".to_owned());
    egui_ctx.ctx_mut().set_fonts(fonts);
}

pub(crate) fn init_seed(mut ui_state: ResMut<UiState>) {
    ui_state.seed = fastrand::u64(0..u64::MAX).to_string();
}

pub(crate) fn show_overlay(mut commands: Commands, txt: Res<TitleText>, mut egui_ctx: ResMut<EguiContext>, mut ui_state: ResMut<UiState>) {
    let ctx = egui_ctx.ctx_mut();
    egui::Area::new("MineSweeper")
        .anchor(Align2::CENTER_CENTER, egui::vec2(0.0, -150.0))
        .show(ctx, |ui| {
            let color = Rgba::from_rgba_unmultiplied(txt.1.r(), txt.1.g(), txt.1.b(), txt.1.a());

            Frame::none()
                .inner_margin(Margin::same(5.0))
                .fill(egui::Color32::from_rgba_unmultiplied(0, 125, 125, 120))
                .rounding(0.15)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.heading(RichText::new(txt.0.clone())
                            .color(Color32::from(color))
                            .size(75.0));


                        Grid::new("gird")
                            .min_row_height(10.0)
                            .show(ui, |ui| {
                                ui.label(RichText::new("mines").size(25.0));
                                ui.add(Slider::new(&mut ui_state.mines, 1..=100));
                                ui.end_row();

                                ui.label(RichText::new("rows").size(25.0));
                                ui.add(Slider::new(&mut ui_state.rows, 1..=30));
                                ui.end_row();

                                ui.label(RichText::new("columns").size(25.0));
                                ui.add(Slider::new(&mut ui_state.columns, 1..=30));
                                ui.end_row();
                                ui.label(RichText::new("seed").size(25.0));
                                ui.text_edit_singleline(&mut ui_state.seed);
                                ui.end_row();

                                ui.end_row();

                                Grid::new("grid2")
                                    .show(ui, |_| {});
                                if ui.button(RichText::new("New Game").size(50.0)).clicked() {
                                    commands.insert_resource::<FieldGenerationOptions>(ui_state.clone().into());
                                    commands.insert_resource(NextState(GameState::Playing))
                                }
                            });
                    });
                });
        });
}