use bevy::{prelude::*, render::camera::ScalingMode};

use crate::assets::Images;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)));
        app.add_systems(Startup, spawn);
    }
}

#[derive(Component)]
pub struct PlayerCamera;

fn spawn(mut commands: Commands, assets: Res<Images>) {
    let mut entity = commands.spawn_empty();
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::FixedVertical(720.);
    entity.insert(camera);
    entity.insert(PlayerCamera);
    commands.spawn(SpriteBundle {
        texture: assets.background.clone(),
        ..default()
    });
}
