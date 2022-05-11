use bevy::prelude::*;
use crate::{CameraOptions, CellHandle, Field, GameState, MainCam, RenderTarget, Revealer, RevealerImpl, RevealResult, spawn_overlay, TextFont};

use leafwing_input_manager::{Actionlike};
use leafwing_input_manager::prelude::ActionState;

#[derive(Actionlike, Clone, Debug)]
pub enum FieldInteraction {
    Reveal,
    ToggleMark,
    RevealAll,
}

#[derive(Actionlike, Debug, Clone)]
pub enum GameInteractions {
    NewGame,
}


pub(crate) fn update_cell_interaction(mut query: Query<&ActionState<FieldInteraction>>,
                           mut field: ResMut<Field>,
                           mut state: ResMut<State<GameState>>,
                           cam_options: Res<CameraOptions>,
                           converter: Res<MousePositionToCellConverter>,
                           cam_query: Query<(&Camera, &GlobalTransform), With<MainCam>>,
                           windows: Res<Windows>,
                           mut commands: Commands, font: Res<TextFont>) {
    let action = query.single_mut();

    let (cam, trans) = cam_query.single();
    if let Some(cell) = converter.get_cell_coords(cam, trans, windows, &cam_options, &field) {
        if action.just_pressed(FieldInteraction::Reveal) {
            match field.reveal(cell) {
                RevealResult::Mine => {
                    spawn_overlay("You lost!", &mut commands, &font, &cam_options);
                    state.set(GameState::Menu).expect("Failed to set game state");
                }
                RevealResult::Empty(_) => {
                    RevealerImpl::reveal_area(&mut field, cell);
                }
                RevealResult::AlreadyRevealed => {}
            }
        }
        if action.just_pressed(FieldInteraction::ToggleMark) {
            info!("Toggling mark");
            field.toggle_mark(cell);
        }
    }
}



pub struct MousePositionToCellConverter {

}


impl MousePositionToCellConverter {
    pub fn new() -> Self {
        Self {}
    }

    fn screen_to_cell(&self, ndc: Vec2, mat: &Mat4, cam_options: &CameraOptions, field: &Field) -> CellHandle {
        let world_pos = mat.project_point3(ndc.extend(-1.0));
        let cell = cam_options.convert_screen_to_field(world_pos.x, world_pos.y, &field);
        cell
    }

    pub(crate) fn get_cell_coords(&self, cam: &Camera, trans: &GlobalTransform, windows: Res<Windows>, cam_options: &CameraOptions, field: &Field) -> Option<CellHandle> {
        let wnd = if let RenderTarget::Window(wnd) = &cam.target {
            windows.get(wnd.clone()).unwrap()
        } else {
            windows.primary()
        };
        if let Some(cur_pos) = wnd.cursor_position() {
            let window_size = Vec2::new(wnd.width(), wnd.height());
            let ndc = (cur_pos / window_size) * 2.0 - Vec2::ONE;
            let cell = self.screen_to_cell(ndc, &(trans.compute_matrix() * cam.projection_matrix.inverse()), cam_options, field);
            return Some(cell);
        }
        None
    }
}
