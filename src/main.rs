mod boid;
mod camera;
mod rng;

use bevy::{log::LogPlugin, prelude::*};
use bevy_editor_pls::{EditorPlugin, EditorWindowPlacement};

struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(rng::RngPlugin);
        app.add_plugins(camera::CameraPlugin);
        app.add_plugins(boid::BoidPlugin);
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Bevy App".to_string(),
                    ..default()
                }),
                ..default()
            })
            .set(LogPlugin {
                filter: "error,procedural=debug".into(),
                ..default()
            }),
    );
    app.add_plugins(CorePlugin);
    // app.add_plugins(EditorPlugin {
    //     window: EditorWindowPlacement::New(Window {
    //         title: "Bevy App Debug".to_string(),
    //         ..default()
    //     }),
    // });
    app.run();
}
