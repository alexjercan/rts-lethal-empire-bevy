use bevy::prelude::*;

use crate::{
    building::{BuildingKind, BuildingTool},
    core::{CursorActive, GameAssets, GameStates, ToolMode},
};

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
pub struct NoPointerCapture;

#[derive(Component)]
struct BuildingButton;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameStates::Playing), setup_ui)
            .add_systems(
                Update,
                (
                    ui_button_interaction,
                    building_button_interaction,
                    update_tool_mode_on_interraction,
                )
                    .run_if(in_state(GameStates::Playing)),
            );
    }
}

fn setup_ui(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::End,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(10.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(10.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    for (kind, icon) in &game_assets.ui_buildings {
                        parent
                            .spawn((
                                kind.clone(),
                                BuildingButton,
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Px(69.0),
                                        height: Val::Px(69.0),
                                        border: UiRect::all(Val::Px(5.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    border_color: BorderColor(Color::BLACK),
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                            ))
                            .with_children(|parent| {
                                parent.spawn(ImageBundle {
                                    style: Style {
                                        max_width: Val::Px(64.0),
                                        max_height: Val::Px(64.0),
                                        ..default()
                                    },
                                    image: UiImage::new(icon.clone()),
                                    ..default()
                                });
                            });
                    }
                });
        });
}

fn ui_button_interaction(
    mut q_interaction: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, mut border_color) in q_interaction.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::RED;
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn update_tool_mode_on_interraction(
    q_interaction: Query<
        &Interaction,
        (With<Node>, Changed<Interaction>, Without<NoPointerCapture>),
    >,
    mut cursor_active: ResMut<CursorActive>,
) {
    if q_interaction.is_empty() {
        return;
    }

    **cursor_active = q_interaction.iter().all(|i| matches!(i, Interaction::None));
}

fn building_button_interaction(
    q_interaction: Query<
        (&Interaction, &BuildingKind),
        (Changed<Interaction>, With<Button>, With<BuildingButton>),
    >,
    mut tool_mode: ResMut<ToolMode>,
    mut q_tool: Query<&mut BuildingKind, (With<BuildingTool>, Without<Button>)>,
) {
    let Ok(mut building_kind) = q_tool.get_single_mut() else {
        return;
    };

    for (interaction, kind) in q_interaction.iter() {
        match *interaction {
            Interaction::Pressed => {
                *tool_mode = ToolMode::Build;
                *building_kind = kind.clone();
            }
            _ => (),
        }
    }
}
