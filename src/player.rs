use bevy::prelude::*;
use rand::Rng;

use crate::{
    boid::{Alignment, Boid, Velocity},
    input::InputEvent,
    node::Collectible,
    rng::RngSource,
    GameEvent, Health,
};

#[derive(Component)]
pub struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_systems(Update, movement);
        app.add_systems(Update, collect);
    }
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, mut rng: ResMut<RngSource>) {
    let rng = &mut **rng;
    let mut entity = commands.spawn_empty();
    entity.insert(Name::new("player"));
    entity.insert(SpriteBundle {
        texture: asset_server.load("player.png"),
        ..default()
    });
    entity.insert(Player);
    entity.insert(Boid);
    entity.insert(Health(100));
    entity.insert(Velocity::default());
    entity.insert(Alignment::default());
    entity.insert(TransformBundle {
        local: Transform::from_xyz(
            rng.gen::<f32>() * 1000. - 500.,
            rng.gen::<f32>() * 600. - 300.,
            1.0,
        ),
        ..default()
    });
}

fn movement(
    mut input: EventReader<InputEvent>,
    mut player: Query<&mut Velocity, With<Player>>,
    time: Res<Time>,
) {
    let Ok(mut vel) = player.get_single_mut() else {
        return;
    };

    let movement = input
        .read()
        .map(|event| match event {
            InputEvent::Move(direction) => *direction,
            InputEvent::Schwack(_) => Vec2::ZERO,
            InputEvent::SpawnBoid => Vec2::ZERO,
        })
        .sum::<Vec2>();

    if movement.length_squared() > 0.0 {
        vel.0 += movement * 50.;
    }
    vel.0 = vel.0.lerp(Vec2::ZERO, time.delta_seconds() * 10.0);
}

fn collect(
    boids: Query<&Transform, With<Player>>,
    nodes: Query<(Entity, &Transform), With<Collectible>>,
    mut events: EventWriter<GameEvent>,
) {
    for boid in boids.iter() {
        for (node_entity, node_trans) in nodes.iter() {
            if boid.translation.distance_squared(node_trans.translation) < 10.0 * 10.0 {
                events.send(GameEvent::Collect(node_entity));
            }
        }
    }
}
