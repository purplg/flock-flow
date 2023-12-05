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
mod track;

use std::f32::consts::PI;

use bevy::{app::AppExit, prelude::*};
use bevy_spatial::kdtree::KDTree2;
use collectible::Collectible;
use rand::{distributions::Standard, Rng};
use rng::RngSource;
use track::Tracked;

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
        app.add_systems(Update, quit);
        app.add_systems(Update, health);
        app.add_systems(Update, waves);
    }
}

#[derive(Debug, Event)]
pub enum GameEvent {
    NextWave,
    HurtNode {
        entity: Entity,
        amount: u32,
        velocity: Vec2,
    },
}

#[derive(Component)]
pub struct Health(pub u32);

fn health(
    mut commands: Commands,
    mut events: EventReader<GameEvent>,
    mut health: Query<&mut Health>,
) {
    for event in events.read() {
        if let GameEvent::HurtNode {
            entity,
            amount,
            velocity: _,
        } = event
        {
            let Ok(mut health) = health.get_mut(*entity) else {
                continue;
            };

            match health.0.checked_sub(*amount) {
                Some(remaining) => health.0 = remaining,
                None => commands.entity(*entity).despawn(),
            }
        }
    }
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

fn waves(
    mut events: EventReader<GameEvent>,
    mut boid_events: EventWriter<boid::Event>,
    mut collectible_events: EventWriter<collectible::Event>,
    mut rng: ResMut<RngSource>,
) {
    for event in events.read() {
        match event {
            GameEvent::NextWave => {
                boid_events.send_batch(
                    (&mut **rng)
                        .sample_iter(Standard)
                        .take(99)
                        .map(|sample: f32| sample * PI * 2.0)
                        .map(|angle: f32| (angle.cos(), angle.sin()))
                        .map(|(x, y): (f32, f32)| {
                            let pos = Vec2 { x, y } * 1000.;
                            boid::Event::SpawnBoi(pos)
                        }),
                );
                boid_events.send(boid::Event::SpawnCalmBoi);
            }
            GameEvent::HurtNode {
                entity: _,
                amount: _,
                velocity: _,
            } => {}
        }
    }
}
