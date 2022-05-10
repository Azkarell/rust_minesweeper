use std::io;
use std::ops::{IndexMut};
use std::collections::HashMap;
use bevy::prelude::*;
use bevy::asset::diagnostic::AssetCountDiagnosticsPlugin;
use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use bevy::render::camera::{CameraProjection, RenderTarget, ScalingMode, WindowOrigin};

use bevy::sprite::{MaterialMesh2dBundle, SpecializedMaterial2d};
use bevy::text::{Text2dBounds};
use bevy::window::WindowMode;

use leafwing_input_manager::InputManagerBundle;
use leafwing_input_manager::prelude::{InputManagerPlugin, InputMap, UserInput};
use leafwing_input_manager::user_input::InputButton;

use ron_config::{ConfigBuilder};

use crate::field::{CellHandle, CellState, Field, Mark, RevealResult};


use crate::generate::{FieldGenerationOptions, DefaultFieldGenerator};
use crate::interactions::{FieldInteraction, MousePositionToCellConverter, update_cell_interaction};
use crate::revealing::{Revealer, RevealerImpl};

use serde::{Deserialize, Serialize};
use crate::render::update_render;

mod field;
mod generate;
mod revealing;
mod interactions;
mod render;

#[derive(PartialEq, Debug, Clone, Copy, Hash, Eq)]
pub enum GameState {
    Playing,
    Menu,
    Init,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct WindowConfig {
    width: f32,
    height: f32,
    title: String,
    mode: WindowModeConfig,
    transparent: bool,
    resizable: bool,
    decorations: bool,
}

impl From<WindowDescriptor> for WindowConfig {
    fn from(descriptor: WindowDescriptor) -> Self {
        Self {
            width: descriptor.width,
            height: descriptor.height,
            title: descriptor.title,
            mode: descriptor.mode.into(),
            transparent: descriptor.transparent,
            resizable: descriptor.resizable,
            decorations: descriptor.decorations,
        }
    }
}

impl TryFrom<WindowConfig> for WindowDescriptor {
    type Error = io::Error;

    fn try_from(value: WindowConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            width: value.width,
            height: value.height,
            title: value.title,
            mode: value.mode.try_into()?,
            transparent: value.transparent,
            resizable: value.resizable,
            decorations: value.decorations,
            ..default()
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct WindowModeConfig(String);

impl From<WindowMode> for WindowModeConfig {
    fn from(mode: WindowMode) -> Self {
        match mode {
            WindowMode::Windowed => WindowModeConfig("Windowed".to_owned()),
            WindowMode::BorderlessFullscreen => WindowModeConfig("BorderlessFullscreen".to_owned()),
            WindowMode::SizedFullscreen => WindowModeConfig("SizedFullscreen".to_owned()),
            WindowMode::Fullscreen => WindowModeConfig("Fullscreen".to_owned()),
        }
    }
}

impl TryFrom<WindowModeConfig> for WindowMode {
    type Error = io::Error;

    fn try_from(value: WindowModeConfig) -> Result<Self, Self::Error> {
        match value.0.as_str() {
            "Windowed" => Ok(WindowMode::Windowed),
            "BorderlessFullscreen" => Ok(WindowMode::BorderlessFullscreen),
            "SizedFullscreen" => Ok(WindowMode::SizedFullscreen),
            "Fullscreen" => Ok(WindowMode::Fullscreen),
            _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid window mode")),
        }
    }
}


fn main() {
    let mut app = App::new();
    // #[cfg(not(target_arch = "wasm32"))]
    //     let config = ConfigBuilder::new()
    //     .folder("./config", None)
    //     .build();
    // #[cfg(target_arch = "wasm32")]
    // let config = ConfigBuilder::new()
    //     .http("http://localhost:3000/config/window.ron")
    //     .build();

    // println!("{:?}", config);

    app.insert_resource::<WindowDescriptor>(WindowDescriptor{
        height: 800.0,
        width: 800.0,
        title: "Minesweeper".to_owned(),
        ..default()
    });


    app.add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(EntityCountDiagnosticsPlugin::default())
        .add_plugin(AssetCountDiagnosticsPlugin::<Image>::default())
        .add_plugin(InputManagerPlugin::<FieldInteraction>::default());

    let field_option = Some(FieldGenerationOptions {
        width: 20,
        height: 20,
        mine_count: 20,
        ..default()
    });


    let field = DefaultFieldGenerator::generate(field_option);

    app.insert_resource(InitState::default());
    app.insert_resource(field);
    app.insert_resource(ClearColor(Color::rgb(0.9, 0.9, 0.9)));
    app.insert_resource(CameraOptions {
        scale: 1000.0,
        width: 1000.0,
        height: 1000.0,
    });
    app.add_state(GameState::Init);
    app.add_system_set(SystemSet::on_enter(GameState::Init).with_system(init_resources)
        .with_system(init_camera)
        .with_system(init_input));

    app.add_system_set(SystemSet::on_update(GameState::Init).with_system(check_init));
    app.add_system_set(SystemSet::on_update(GameState::Playing).with_system(update_render)
        .with_system(update_cell_interaction)
        .with_system(check_victory));
    app.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(init_render_field));
    app.add_system_set(SystemSet::on_update(GameState::Menu).with_system(update_render));
    app.run();
}

#[derive(Component, Debug)]
struct CellComponent {
    pub handle: CellHandle,
}

#[derive(Bundle)]
struct CellBundle<M: SpecializedMaterial2d> {
    cell: CellComponent,
    #[bundle]
    mesh: MaterialMesh2dBundle<M>,

}

#[derive(Debug)]
struct CameraOptions {
    pub width: f32,
    pub height: f32,
    pub scale: f32,
}

impl CameraOptions {
    pub fn convert_screen_to_field(&self, x: f32, y: f32, field: &Field) -> CellHandle {
        let cell_option = CellOptions::from_field_dimension_and_camera_options((field.width, field.height), &self);
        let cell_x = x / cell_option.total_width();
        let cell_y = y / cell_option.total_height();
        info!("{:?}", (cell_x, cell_y));
        let cell = CellHandle {
            x: cell_x as usize,
            y: cell_y as usize,
        };
        cell
    }
}

#[derive(Debug)]
struct CellOptions {
    pub width: f32,
    pub height: f32,
    pub padding: f32,
}


impl CellOptions {
    pub fn from_field_dimension_and_camera_options(field_dimension: (usize, usize), camera_options: &CameraOptions) -> Self {
        let width = camera_options.width / (field_dimension.0 as f32);
        let height = camera_options.height / (field_dimension.1 as f32);
        Self {
            width,
            height,
            padding: 0.0,
        }
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self.width = self.width - padding;
        self.height = self.height - padding;
        self
    }

    pub fn get_translation(&self, handle: CellHandle) -> Vec3 {
        let tran = Vec3::new(
            self.total_width() / 2.0 + (handle.x as f32) * self.total_width(),
            self.total_height() / 2.0 + (handle.y as f32) * self.total_height(),
            0.0,
        );
        tran
    }

    pub fn total_width(&self) -> f32 {
        self.width + self.padding
    }
    pub fn total_height(&self) -> f32 {
        self.height + self.padding
    }
}

struct StateMaterials {
    hover: HoverColor,
    revealed: RevealedColor,
    hidden: HiddenColor,
    mine: MineColor,
    marked_mine: MarkedMineColor,
    marked_empty: MarkedEmptyColor,
    colors_numbers: HashMap<u8, Color>,
}

#[derive(Deref, DerefMut)]
struct HoverColor(Color);

#[derive(Deref, DerefMut)]
struct HiddenColor(Color);

#[derive(Deref, DerefMut)]
struct RevealedColor(Color);

#[derive(Deref, DerefMut)]
struct MineColor(Color);

#[derive(Deref, DerefMut)]
struct MarkedMineColor(Color);

#[derive(Deref, DerefMut)]
struct MarkedEmptyColor(Color);

#[derive(Debug, Component)]
struct MainCam;


struct TextFont(Handle<Font>);

struct InitState {
    pub camera: bool,
    pub input: bool,
    pub resources: bool,
}

impl InitState {
    fn is_ready(&self) -> bool {
        self.camera && self.input && self.resources
    }
}

impl Default for InitState {
    fn default() -> Self {
        Self {
            camera: false,
            input: false,
            resources: false,
        }
    }
}

fn check_init(init_state: Res<InitState>, mut state: ResMut<State<GameState>>) {
    if init_state.is_ready() {
        state.set(GameState::Playing).expect("Failed to set game state");
    }
}

fn init_camera(mut commands: Commands, camera_options: Res<CameraOptions>, mut init_state: ResMut<InitState>) {
    info!("cam_options: {:?}", camera_options);
    let mut cam = OrthographicCameraBundle::new_2d();
    cam.orthographic_projection.scaling_mode = ScalingMode::FixedVertical;
    cam.orthographic_projection.window_origin = WindowOrigin::BottomLeft;
    cam.orthographic_projection.update(camera_options.width, camera_options.height);
    cam.orthographic_projection.scale = camera_options.scale;

    commands.spawn_bundle(cam).insert(MainCam);
    init_state.camera = true;
}

fn init_resources(mut commands: Commands, mut asset_server: ResMut<AssetServer>, mut init_state: ResMut<InitState>) {
    let color_hidden = Color::rgb(0.4, 0.4, 0.4);
    let color_revealed = Color::rgb(0.8, 0.8, 0.8);
    let color_mine = Color::rgb(0.8, 0.0, 0.0);
    let color_hover = Color::rgb(0.8, 0.8, 0.0);
    let color_marked_mine = Color::rgb(0.6, 0.1, 0.1);
    let color_marked_empty = Color::rgb(0.1, 0.6, 0.1);

    let state_materials = StateMaterials {
        hover: HoverColor(color_hover),
        revealed: RevealedColor(color_revealed),
        hidden: HiddenColor(color_hidden),
        mine: MineColor(color_mine),
        marked_mine: MarkedMineColor(color_marked_mine),
        marked_empty: MarkedEmptyColor(color_marked_empty),
        colors_numbers: HashMap::from([
            (1 as u8, Color::rgb(0.0, 0.0, 1.0)),
            (2 as u8, Color::rgb(0.0, 1.0, 0.0)),
            (3 as u8, Color::rgb(0.0, 1.0, 1.0)),
            (4 as u8, Color::rgb(1.0, 0.0, 0.0)),
            (5 as u8, Color::rgb(1.0, 0.0, 1.0)),
            (6 as u8, Color::rgb(1.0, 1.0, 0.0)),
            (7 as u8, Color::rgb(1.0, 1.0, 1.0)),
            (8 as u8, Color::rgb(0.5, 0.5, 0.5))
        ]),
    };

    commands.insert_resource(state_materials);
    let font: Handle<Font> = asset_server.load("fonts\\pixelated_arial_regular_11.ttf");
    commands.insert_resource(TextFont(font));
    init_state.resources = true;
}

fn init_input(mut commands: Commands, mut init_state: ResMut<InitState>) {
    let mut input_map = InputMap::default();
    input_map.insert(FieldInteraction::Reveal, InputButton::Mouse(MouseButton::Left));
    input_map.insert(FieldInteraction::ToggleMark, InputButton::Mouse(MouseButton::Right));
    commands.spawn_bundle(InputManagerBundle {
        input_map,
        ..Default::default()
    });
    init_state.input = true;
}

fn init_render_field(mut commands: Commands, field: Res<Field>, mut materials: ResMut<Assets<ColorMaterial>>, mut meshes: ResMut<Assets<Mesh>>,
                     camera_options: Res<CameraOptions>, state_materials: Res<StateMaterials>,
                     old: Query<Entity, With<CellComponent>>) {
    for old in old.iter() {
        commands.entity(old).despawn_recursive();
    }

    let converter = MousePositionToCellConverter::new();
    commands.insert_resource(converter);

    let cell_options = CellOptions::from_field_dimension_and_camera_options((field.width, field.height), &camera_options)
        .with_padding(1.0);

    let m = Mesh::from(shape::Quad::new(Vec2::new(cell_options.width, cell_options.height)));

    let mesh = meshes.add(m);

    for cell in field.get_handles() {
        let cell_component = CellComponent {
            handle: cell,
        };

        let mb = MaterialMesh2dBundle {
            mesh: mesh.clone().into(),
            material: materials.add(ColorMaterial::from(state_materials.hidden.0)),
            transform: Transform::default().with_translation(cell_options.get_translation(cell)),
            ..default()
        };

        let bundle = CellBundle {
            cell: cell_component,
            mesh: mb,

        };
        commands.spawn_bundle(bundle);
    };
}


fn check_victory(field: Res<Field>, mut commands: Commands, mut state: ResMut<State<GameState>>, font: Res<TextFont>, cam_options: Res<CameraOptions>) {
    if field.is_won() {
        spawn_overlay("You won!", &mut commands, &font, &cam_options);

        state.set(GameState::Menu).expect("Failed to set game state");
    }
}

fn spawn_overlay(txt: &str, commands: &mut Commands, font: &Res<TextFont>, cam_options: &CameraOptions) {
    commands.spawn_bundle(Text2dBundle {
        text: Text::with_section(txt,
                                 TextStyle {
                                     font: font.0.clone(),
                                     color: Color::RED,
                                     font_size: 150.0,
                                 }, TextAlignment {
                vertical: VerticalAlign::Center,
                horizontal: HorizontalAlign::Center,
            }),
        text_2d_bounds: Text2dBounds {
            size: Size::new(cam_options.width / 1.5, cam_options.height / 1.5)
        },
        transform: Transform::default().with_translation(Vec3::new(cam_options.width / 2.0, cam_options.height / 2.0, 5.0)),
        ..default()
    });
}