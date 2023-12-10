use bevy::{prelude::*, sprite::Anchor};

use crate::{
    boid::{BoidKind, SpawnEvent},
    points::Points,
    GameState,
};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, update_points.run_if(resource_changed::<Points>()));
        app.add_systems(Update, update_entity_count.run_if(on_event::<SpawnEvent>()));
        app.add_systems(OnEnter(GameState::Paused), show_menu);
        app.add_systems(OnExit(GameState::Paused), hide_menu);
        app.add_systems(OnEnter(GameState::GameOver), gameover);
        app.add_systems(OnExit(GameState::GameOver), reset);
        app.add_systems(
            Update,
            try_again_button.run_if(in_state(GameState::GameOver)),
        );
    }
}

#[derive(Component)]
struct UIRoot;

fn setup(mut commands: Commands) {
    let mut points = commands.spawn_empty();
    points.insert(Name::new("Points"));
    points.insert(UIRoot);
    points
        .insert(NodeBundle {
            // border_color: BorderColor(Color::GREEN),
            style: Style {
                // border: UiRect::px(1.0, 1.0, 1.0, 1.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Points
            let mut entity = parent.spawn_empty();
            entity
                .insert(NodeBundle {
                    // border_color: BorderColor(Color::GREEN),
                    style: Style {
                        // border: UiRect::px(1.0, 1.0, 1.0, 1.0),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        width: Val::Percent(100.),
                        height: Val::Percent(50.),
                        justify_content: JustifyContent::End,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(
                            TextBundle::from_sections([
                                TextSection::new(
                                    "Points: ",
                                    TextStyle {
                                        font_size: 36.0,
                                        ..default()
                                    },
                                ),
                                TextSection::new(
                                    "",
                                    TextStyle {
                                        font_size: 36.0,
                                        ..default()
                                    },
                                ),
                            ])
                            .with_text_alignment(TextAlignment::Center),
                        )
                        .insert(PointText);
                });

            // Pause menu
            let mut entity = parent.spawn_empty();
            entity.insert(StateNode);
            entity.insert(NodeBundle {
                // border_color: BorderColor(Color::RED),
                style: Style {
                    // border: UiRect::px(1.0, 1.0, 1.0, 1.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    width: Val::Percent(100.),
                    height: Val::Percent(50.),
                    justify_content: JustifyContent::Start,
                    ..default()
                },
                ..default()
            });
        });

    let mut boids = commands.spawn_empty();
    boids.insert(Name::new("Boid Count"));
    boids.with_children(|parent| {
        let mut entity = parent.spawn_empty();
        entity.insert(Text2dBundle {
            text: Text::from_sections([
                TextSection::new(
                    "Boids: ",
                    TextStyle {
                        font_size: 24.0,
                        ..default()
                    },
                ),
                TextSection::new(
                    "0",
                    TextStyle {
                        font_size: 24.0,
                        ..default()
                    },
                ),
            ]),
            text_anchor: Anchor::BottomLeft,
            global_transform: GlobalTransform::from_xyz(-499., -299., 10.0),
            ..default()
        });
        entity.insert(EntityCount(0));
    });
}

#[derive(Component)]
struct PointText;

fn update_points(mut text: Query<&mut Text, With<PointText>>, points: Res<Points>) {
    for mut text in &mut text {
        text.sections[1].value = format!("{}", points.0);
    }
}

#[derive(Component)]
struct EntityCount(u32);

fn update_entity_count(
    mut text: Query<(&mut Text, &mut EntityCount)>,
    mut events: EventReader<SpawnEvent>,
) {
    let Ok((mut text, mut count)) = text.get_single_mut() else {
        return;
    };

    for event in events.read() {
        match event.kind {
            BoidKind::Boi | BoidKind::CalmBoi => {
                count.0 += event.count;
                text.sections[1].value = format!("{}", count.0);
            }
            BoidKind::AngryBoi => {}
        }
    }
}

#[derive(Component)]
struct GameOverNode;

#[derive(Component)]
struct TryAgainButton;

fn gameover(mut commands: Commands, ui: Query<Entity, With<StateNode>>) {
    let Ok(ui) = ui.get_single() else {
        return;
    };

    commands.entity(ui).with_children(|parent| {
        let mut entity = parent.spawn_empty();
        entity.insert(GameOverNode);
        entity
            .insert(NodeBundle {
                // border_color: BorderColor(Color::PURPLE),
                style: Style {
                    // border: UiRect::px(1.0, 1.0, 1.0, 1.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    width: Val::Percent(100.),
                    height: Val::Percent(50.),
                    justify_content: JustifyContent::Start,
                    ..default()
                },
                ..default()
            })
            .with_children(|parent| {
                parent.spawn(
                    TextBundle::from_section(
                        "Game Over",
                        TextStyle {
                            font_size: 36.0,
                            ..default()
                        },
                    )
                    .with_text_alignment(TextAlignment::Center),
                );
                parent
                    .spawn((
                        TryAgainButton,
                        ButtonBundle {
                            background_color: BackgroundColor(Color::rgb(0.1, 0.1, 0.44)),
                            style: Style {
                                padding: UiRect::all(Val::Px(8.)),
                                ..default()
                            },
                            ..default()
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn(
                            TextBundle::from_section(
                                "Try Again",
                                TextStyle {
                                    font_size: 36.0,
                                    ..default()
                                },
                            )
                            .with_text_alignment(TextAlignment::Center),
                        );
                    });
            });
    });
}

#[derive(Component)]
struct StateNode;

#[derive(Component)]
struct MenuText;

fn show_menu(mut commands: Commands, ui: Query<Entity, With<StateNode>>) {
    let Ok(ui) = ui.get_single() else {
        return;
    };

    commands.entity(ui).with_children(|parent| {
        parent.spawn((
            MenuText,
            TextBundle::from_section(
                "Paused",
                TextStyle {
                    font_size: 36.0,
                    ..default()
                },
            )
            .with_text_alignment(TextAlignment::Center),
        ));
    });
}

fn hide_menu(mut commands: Commands, ui: Query<Entity, With<MenuText>>) {
    let Ok(menu) = ui.get_single() else {
        return;
    };

    commands.entity(menu).despawn_recursive();
}

fn try_again_button(
    mut button: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
    mut state: ResMut<NextState<GameState>>,
) {
    let Ok((interaction, mut background)) = button.get_single_mut() else {
        return;
    };

    match interaction {
        Interaction::Pressed => {
            background.0 = Color::rgb(0.0, 0.0, 0.44);
            state.set(GameState::Playing)
        }
        Interaction::Hovered => {
            background.0 = Color::rgb(0.2, 0.2, 0.44);
        }
        Interaction::None => {
            background.0 = Color::rgb(0.1, 0.1, 0.44);
        }
    }
}

fn reset(mut commands: Commands, ui: Query<Entity, With<GameOverNode>>) {
    let Ok(menu) = ui.get_single() else {
        return;
    };

    commands.entity(menu).despawn_recursive();
}
