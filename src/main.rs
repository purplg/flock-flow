#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::needless_pass_by_value)]

mod assets;
mod boid;
mod camera;
mod collectible;
mod input;
mod player;
mod points;
mod rng;
mod shockwave;
mod track;
mod ui;
mod velocity;

use bevy::{app::AppExit, prelude::*};

struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GameEvent>();
        app.add_plugins(assets::Plugin);
        app.add_plugins(input::InputPlugin);
        app.add_plugins(track::TrackPlugin);
        app.add_plugins(player::PlayerPlugin);
        app.add_plugins(points::PointsPlugin);
        app.add_plugins(rng::RngPlugin);
        app.add_plugins(camera::CameraPlugin);
        app.add_plugins(boid::BoidPlugin);
        app.add_plugins(collectible::Plugin);
        app.add_plugins(shockwave::Plugin);
        app.add_plugins(ui::Plugin);
        app.add_plugins(velocity::Plugin);
        app.add_systems(Update, quit);
        app.add_systems(Update, waves);
    }
}

#[derive(Debug, Event)]
pub enum GameEvent {
    NextWave { position: Vec2, velocity: Vec2 },
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Flock Flow".to_string(),
            resolution: (640., 364.).into(),
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(CorePlugin);

    #[cfg(feature = "inspector")]
    #[allow(clippy::items_after_statements)]
    {
        use bevy_editor_pls::{EditorPlugin, EditorWindowPlacement};
        app.add_plugins(EditorPlugin {
            window: EditorWindowPlacement::New(Window {
                title: "Bevy Debug".to_string(),
                ..default()
            }),
        });
    }

    #[cfg(debug_assertions)]
    #[allow(clippy::items_after_statements)]
    {
        use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
        app.add_plugins(FrameTimeDiagnosticsPlugin);
        use bevy::diagnostic::LogDiagnosticsPlugin;
        app.add_plugins(LogDiagnosticsPlugin::default());
    }
    app.run();
}

fn quit(keys: Res<Input<KeyCode>>, mut app_exit_events: ResMut<Events<AppExit>>) {
    if keys.just_pressed(KeyCode::Escape) {
        app_exit_events.send(AppExit);
    }
}

fn waves(mut events: EventReader<GameEvent>, mut boid_events: EventWriter<boid::SpawnEvent>) {
    for event in events.read() {
        match event {
            GameEvent::NextWave { position, velocity } => {
                boid_events.send(boid::SpawnEvent {
                    kind: boid::BoidKind::Boi,
                    count: 40,
                    position: *position,
                    velocity: *velocity,
                });
                boid_events.send(boid::SpawnEvent {
                    kind: boid::BoidKind::CalmBoi,
                    count: 10,
                    position: *position,
                    velocity: *velocity,
                });
            }
        }
    }
}
