use bevy::{prelude::*, sprite::Anchor};

use crate::{
    boid::{BoidKind, SpawnEvent},
    points::Points,
};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, update_points.run_if(resource_changed::<Points>()));
        app.add_systems(Update, update_entity_count.run_if(on_event::<SpawnEvent>()));
    }
}

#[derive(Component)]
struct UI;

fn setup(mut commands: Commands) {
    let mut entity = commands.spawn_empty();
    entity.insert(Name::new("HotBar"));
    entity.insert(UI);
    entity
        .insert(NodeBundle {
            // border_color: BorderColor(Color::RED),
            style: Style {
                // border: UiRect::px(1.0, 1.0, 1.0, 1.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
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

    let mut entity = commands.spawn_empty();
    entity.with_children(|parent| {
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
