use bevy::prelude::*;
use rand::Rng;

use crate::{input::InputEvent, rng::RngSource, Health};

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
    // entity.insert(Health(100));
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
    mut player: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let Ok(mut player) = player.get_single_mut() else {
        return;
    };

    for event in input.read() {
        match event {
            InputEvent::Move(direction) => {
                player.translation += direction.extend(0.0) * 100.0 * time.delta_seconds()
            }
            InputEvent::Schwack(_) => {}
            InputEvent::SpawnBoid => {}
        }
    }
}
