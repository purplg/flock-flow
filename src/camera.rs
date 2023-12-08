use bevy::{prelude::*, render::camera::ScalingMode};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)));
        app.add_systems(Startup, spawn);
        app.add_systems(Update, gizmo);
    }
}

#[derive(Component)]
pub struct PlayerCamera;

fn spawn(mut commands: Commands) {
    let mut entity = commands.spawn_empty();
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::AutoMin {
        min_width: 640.,
        min_height: 364.,
    };
    entity.insert(camera);
    entity.insert(PlayerCamera);
}

fn gizmo(mut gizmos: Gizmos) {
    gizmos.rect_2d(Vec2::ZERO, 0.0, Vec2::new(1000.0, 600.0), Color::BLUE);
}
