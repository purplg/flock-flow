mod boid;
mod camera;
mod input;
mod node;
mod player;
mod points;
mod rng;

use bevy::{app::AppExit, prelude::*};

struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GameEvent>();
        app.add_plugins(input::InputPlugin);
        app.add_plugins(player::PlayerPlugin);
        app.add_plugins(points::PointsPlugin);
        app.add_plugins(rng::RngPlugin);
        app.add_plugins(camera::CameraPlugin);
        app.add_plugins(boid::BoidPlugin);
        app.add_plugins(node::NodePlugin);
        app.add_systems(Update, quit);
        app.add_systems(Update, health);
    }
}

#[derive(Debug, Event)]
pub enum GameEvent {
    SpawnBoid(Vec2),
    HurtNode {
        entity: Entity,
        amount: u32,
        velocity: Vec2,
    },
    Collect(Entity),
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
    {
        use bevy_editor_pls::{EditorPlugin, EditorWindowPlacement};
        app.add_plugins(EditorPlugin {
            window: EditorWindowPlacement::New(Window {
                title: "Bevy App Debug".to_string(),
                ..default()
            }),
        });
        use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
        app.add_plugins(LogDiagnosticsPlugin::default());
        app.add_plugins(FrameTimeDiagnosticsPlugin);
    }
    app.run();
}

fn quit(keys: Res<Input<KeyCode>>, mut app_exit_events: ResMut<Events<AppExit>>) {
    if keys.just_pressed(KeyCode::Escape) {
        app_exit_events.send(AppExit);
    }
}
