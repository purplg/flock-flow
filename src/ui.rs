use bevy::prelude::*;

use crate::points::Points;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, update_points);
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
            border_color: BorderColor(Color::GREEN),
            style: Style {
                border: UiRect::px(1.0, 1.0, 1.0, 1.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                padding: UiRect::percent(1.0, 1.0, 1.0, 1.0),
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
}

#[derive(Component)]
struct PointText;

fn update_points(mut text: Query<&mut Text, With<PointText>>, points: Res<Points>) {
    for mut text in text.iter_mut() {
        text.sections[1].value = format!("{}", points.0);
    }
}