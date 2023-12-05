use super::{BoidBundle, Velocity};

use bevy::prelude::*;
use bevy_spatial::{kdtree::KDTree2, SpatialAccess};
use rand::Rng;

use crate::{input::InputEvent, rng::RngSource, track::Tracked, GameEvent};

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, input);
        app.add_systems(Update, spawn);
    }
}

#[derive(Component)]
struct Boi;

fn input(
    mut input_events: EventReader<InputEvent>,
    mut game_events: EventWriter<GameEvent>,
    quadtree: Res<KDTree2<Tracked>>,
    mut bois: Query<&mut Velocity, With<Boi>>,
) {
    for event in input_events.read() {
        match event {
            InputEvent::SpawnBoi => {
                game_events.send(GameEvent::NextWave);
            }
            InputEvent::Schwack(schwack_pos) => {
                for (pos, entity) in quadtree
                    .within_distance(*schwack_pos, 100.)
                    .into_iter()
                    .filter_map(|(pos, entity)| entity.map(|entity| (pos, entity)))
                {
                    let Ok(mut vel) = bois.get_mut(entity) else {
                        continue;
                    };
                    vel.0 += (pos - *schwack_pos) * 10.;
                }
            }
            InputEvent::Turn(_) => {}
        }
    }
}

fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: ResMut<RngSource>,
    mut events: EventReader<super::Event>,
) {
    for event in events.read() {
        if let super::Event::SpawnBoi = event {
            let angle: f32 = rng.gen();
            let pos = Vec2 {
                x: angle.cos(),
                y: angle.sin(),
            } * 1000.;
            let mut entity = commands.spawn_empty();
            entity.insert(Name::new("Boi"));
            entity.insert(Boi);
            entity.insert(SpriteBundle {
                texture: asset_server.load("boid.png"),
                ..default()
            });
            entity.insert(BoidBundle::new(pos, &mut **rng));
        }
    }
}
