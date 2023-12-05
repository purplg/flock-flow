#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::needless_pass_by_value)]

mod boid;
mod camera;
mod collectible;
mod input;
mod player;
mod points;
mod rng;
mod shockwave;
mod track;

use bevy::{app::AppExit, prelude::*};

struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GameEvent>();
        app.add_plugins(input::InputPlugin);
        app.add_plugins(track::TrackPlugin);
        app.add_plugins(player::PlayerPlugin);
        app.add_plugins(points::PointsPlugin);
        app.add_plugins(rng::RngPlugin);
        app.add_plugins(camera::CameraPlugin);
        app.add_plugins(boid::BoidPlugin);
        app.add_plugins(collectible::Plugin);
        app.add_plugins(shockwave::Plugin);
        app.add_systems(Update, quit);
        app.add_systems(Update, waves);
    }
}

#[derive(Debug, Event)]
pub enum GameEvent {
    NextWave,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Bevy App".to_string(),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(CorePlugin);

    #[cfg(debug_assertions)]
    #[allow(clippy::items_after_statements)]
    {
        use bevy_editor_pls::{EditorPlugin, EditorWindowPlacement};
        app.add_plugins(EditorPlugin {
            window: EditorWindowPlacement::New(Window {
                title: "Bevy App Debug".to_string(),
                ..default()
            }),
        });
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

fn waves(mut events: EventReader<GameEvent>, mut boid_events: EventWriter<boid::Event>) {
    for event in events.read() {
        match event {
            GameEvent::NextWave => {
                boid_events.send_batch([boid::Event::SpawnBoi].repeat(90));
                boid_events.send_batch([boid::Event::SpawnCalmBoi].repeat(10));
            }
        }
    }
}
