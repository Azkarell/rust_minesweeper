use bevy::prelude::*;
use crate::{CellComponent, CellState, ColorMaterial, Field, Mark, StateMaterials, Text2dBounds, TextFont};

pub(crate) fn update_render(mut query: Query<(Entity, &CellComponent, &Handle<ColorMaterial>, Option<&Children>)>,
                 mut materials: ResMut<Assets<ColorMaterial>>,
                 colors: Res<StateMaterials>,
                 mut field: ResMut<Field>,
                 mut commands: Commands,
                 font: Res<TextFont>) {
    for (e, cell, m, childs) in query.iter_mut() {
        if field[cell.handle].is_dirty {
            let color = match field[cell.handle].get_state() {
                CellState::Hidden => {
                    colors.hidden.clone()
                },
                CellState::Revealed => {
                    if field[cell.handle].is_mine {
                        colors.mine.clone()
                    } else {
                        if childs.is_none() {
                            let adj = field.get_adjacent_mines(cell.handle);
                            if adj > 0 {
                                let text = commands.spawn_bundle(Text2dBundle {
                                    text: Text::with_section(adj.to_string(),
                                                             TextStyle {
                                                                 font: font.0.clone(),
                                                                 color: colors.colors_numbers[&(adj as u8)],
                                                                 font_size: 50.0,
                                                             }, TextAlignment {
                                            vertical: VerticalAlign::Center,
                                            horizontal: HorizontalAlign::Center,
                                        }),
                                    text_2d_bounds: Text2dBounds {
                                        size: Size::new(100.0, 100.0)
                                    },
                                    transform: Transform::default().with_translation(Vec3::new(0.0, 0.0, 2.0)),
                                    ..default()
                                }).id();
                                commands.entity(e).add_child(text);
                            } else {
                                let empty = commands.spawn().id();
                                commands.entity(e).add_child(empty);
                            }
                        }
                        colors.revealed.clone()
                    }
                },
                CellState::Marked(m) => {
                    match m {
                        Mark::Mine => colors.marked_mine.clone(),
                        Mark::Empty => colors.marked_empty.clone(),
                    }
                }
            };
            materials.get_mut(m).expect("Material not found").color = color;
            field[cell.handle].is_dirty = false;
        }
    }
}