use bevy::prelude::*;
use rand::Rng;

use crate::{
    boid::{Alignment, Velocity},
    input::InputEvent,
    rng::RngSource,
    Health,
};

#[derive(Component)]
struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_systems(Update, movement);
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
    entity.insert(Health(100));
    entity.insert(Velocity::default());
    entity.insert(Alignment::default());
    entity.insert(TransformBundle {
        local: Transform::from_xyz(
            rng.gen::<f32>() * 1000. - 500.,
            rng.gen::<f32>() * 600. - 300.,
            0.0,
        ),
        ..default()
    });
}

fn movement(
    mut input: EventReader<InputEvent>,
    mut player: Query<(&mut Transform, &mut Velocity, &Alignment), With<Player>>,
    time: Res<Time>,
) {
    let Ok((mut trans, mut vel, alignment)) = player.get_single_mut() else {
        return;
    };

    let mut moving = false;
    let movement = input
        .read()
        .map(|event| match event {
            InputEvent::Move(direction) => {
                moving = true;
                *direction
            }

            InputEvent::Schwack(_) => Vec2::ZERO,
            InputEvent::SpawnBoid => Vec2::ZERO,
        })
        .sum::<Vec2>();

    vel.0 += alignment.effect;
    if moving {
        vel.0 += movement * 50.;
    }
    vel.0 = vel.0.clamp_length_max(200.0);
    trans.translation.x += vel.0.x * time.delta_seconds();
    trans.translation.y += vel.0.y * time.delta_seconds();
    vel.0 = vel.0.lerp(Vec2::ZERO, time.delta_seconds() * 10.0);
}
