use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)));
        app.add_systems(Startup, spawn);
        app.add_systems(Update, gizmo);
    }
}

fn spawn(mut commands: Commands) {
    let mut entity = commands.spawn_empty();
    entity.insert(Camera2dBundle::default());
}

fn gizmo(mut gizmos: Gizmos, cameras: Query<&Camera2d>) {
    gizmos.rect_2d(Vec2::ZERO, 0.0, Vec2::new(1000.0, 600.0), Color::BLUE);
}
