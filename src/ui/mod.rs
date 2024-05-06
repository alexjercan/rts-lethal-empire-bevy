use bevy::prelude::*;

use crate::{
    building::{BuildingKind, BuildingTool},
    core::{CursorActive, GameAssets, GameStates, ToolMode},
    quota::{Quota, QuotaSuccess, QuotaTimer, ResourceCount},
};

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
pub struct NoPointerCapture;

#[derive(Component)]
struct BuildingButton;

#[derive(Component)]
struct QuotaInformation;

#[derive(Component)]
struct QuotaSuccessDisplay;

#[derive(Component)]
struct QuotaSuccessDisplayRoot;

#[derive(Component)]
struct HideMeIn(Timer);

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameStates::Playing), setup_ui)
            .add_systems(
                Update,
                (
                    ui_button_interaction,
                    building_button_interaction,
                    update_cursor_on_interraction,
                    update_quota_information,
                    update_quota_success_display,
                    update_hide_me_in,
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
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
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
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(50.0),
                                height: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                QuotaInformation,
                                TextBundle::from_section(
                                    "TIME LEFT: 10:00 QUOTA: 500/1000",
                                    TextStyle {
                                        font_size: 40.0,
                                        color: Color::rgb(0.9, 0.9, 0.9),
                                        ..default()
                                    },
                                ),
                            ));
                        });
                });

            parent
                .spawn((
                    QuotaSuccessDisplayRoot,
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(80.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(50.0),
                                height: Val::Percent(50.0),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                QuotaSuccessDisplay,
                                TextBundle::from_section(
                                    "STATUS",
                                    TextStyle {
                                        font_size: 40.0,
                                        color: Color::rgb(0.9, 0.9, 0.9),
                                        ..default()
                                    },
                                ),
                            ));
                        });
                });

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

fn update_cursor_on_interraction(
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

fn update_quota_information(
    mut q_quota: Query<&mut Text, With<QuotaInformation>>,
    quota_timer: Res<QuotaTimer>,
    resource_count: Res<ResourceCount>,
    quota: Res<Quota>,
) {
    for mut text in q_quota.iter_mut() {
        let seconds = quota_timer.remaining().as_secs();
        let minutes = seconds / 60;
        let seconds = seconds % 60;

        text.sections[0].value = format!(
            "TIME LEFT: {:02}:{:02} QUOTA: {}/{}",
            minutes, seconds, **resource_count, **quota
        );
    }
}

fn update_quota_success_display(
    mut commands: Commands,
    mut q_display: Query<&mut Text, With<QuotaSuccessDisplay>>,
    mut q_display_root: Query<(Entity, &mut Visibility), With<QuotaSuccessDisplayRoot>>,
    quota_success: Res<QuotaSuccess>,
) {
    if !quota_success.is_changed() || quota_success.is_added() {
        return;
    }

    for (display, mut visibility) in q_display_root.iter_mut() {
        *visibility = Visibility::Visible;
        commands.entity(display).insert(HideMeIn(Timer::from_seconds(5.0, TimerMode::Once)));
    }

    for mut text in q_display.iter_mut() {
        text.sections[0].value = if **quota_success {
            "You met the quota!".to_string()
        } else {
            "You didn't meet the quota!".to_string()
        };
        text.sections[0].style.color = if **quota_success {
            Color::rgb(0.0, 1.0, 0.0)
        } else {
            Color::rgb(1.0, 0.0, 0.0)
        };
    }
}

fn update_hide_me_in(
    mut commands: Commands,
    time: Res<Time>,
    mut q_hide_me_in: Query<(Entity, &mut Visibility, &mut HideMeIn)>,
) {
    for (entity, mut visibility, mut timer) in q_hide_me_in.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            commands.entity(entity).remove::<HideMeIn>();
            *visibility = Visibility::Hidden;
        }
    }
}
