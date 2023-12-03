use bevy::prelude::*;
use rand::Rng;

use crate::{rng::RngSource, Health};

#[derive(Component)]
struct Node;

pub struct NodePlugin;

impl Plugin for NodePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
    }
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, mut rng: ResMut<RngSource>) {
    let rng = &mut **rng;
    let mut entity = commands.spawn_empty();
    entity.insert(Name::new("Node"));
    entity.insert(SpriteBundle {
        texture: asset_server.load("node.png"),
        ..default()
    });
    entity.insert(Node);
    entity.insert(Health(100));
    entity.insert(TransformBundle {
        local: Transform::from_xyz(
            rng.gen::<f32>() * 1000. - 500.,
            rng.gen::<f32>() * 600. - 300.,
            0.0,
        ),
        ..default()
    });
}
