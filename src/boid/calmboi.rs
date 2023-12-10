use std::time::Duration;

use bevy::prelude::*;
use bevy_spatial::kdtree::KDTree2;
use bevy_spatial::SpatialAccess;
use rand::Rng;

use crate::assets::Images;
use crate::collectible::{self, Collectible};

use crate::shockwave;
use crate::{rng::RngSource, track::Tracked, GameEvent};

use super::{BoidBundle, BoidKind, Home, Velocity};

#[derive(Component)]
struct CalmBoi;

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn.run_if(on_event::<super::SpawnEvent>()));
        app.add_systems(Update, collect);
        app.add_systems(
            Update,
            super::home::<Collectible>.run_if(in_state(crate::GameState::Playing)),
        );
    }
}

fn spawn(
    mut commands: Commands,
    images: Res<Images>,
    mut rng: ResMut<RngSource>,
    mut events: EventReader<super::SpawnEvent>,
) {
    for event in events.read() {
        if let super::BoidKind::CalmBoi = event.kind {
            for _ in 0..event.count {
                let mut entity = commands.spawn_empty();

                entity.insert(Name::new("CalmBoi"));
                entity.insert(CalmBoi);
                let home: Home<Collectible> = Home::new(2.0);
                entity.insert(home);
                entity.insert(SpriteBundle {
                    texture: images.calmboi.clone(),
                    ..default()
                });

                let offset = Vec2 {
                    x: 16. * rng.gen::<f32>() - 8.,
                    y: 16. * rng.gen::<f32>() - 8.,
                };
                entity.insert(BoidBundle::new(
                    (event.position + offset).extend(1.0),
                    event.velocity,
                ));
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn collect(
    mut commands: Commands,
    quadtree: Res<KDTree2<Tracked>>,
    boid: Query<(Entity, &Transform, &Velocity), With<CalmBoi>>,
    collectibles: Query<Entity, (With<Collectible>, Without<collectible::Cooldown>)>,
    mut collectible_event: EventWriter<collectible::Event>,
    mut game_events: EventWriter<GameEvent>,
    mut boi_events: EventWriter<super::SpawnEvent>,
    mut shockwave_events: EventWriter<shockwave::Event>,
) {
    for (boid_entity, trans, vel) in boid.iter() {
        let pos = trans.translation.xy();
        if let Some((position, entity)) = quadtree
            .within_distance(pos, 32.0)
            .into_iter()
            .filter_map(|(_pos, entity)| entity.map(|entity| (pos, entity)))
            .find(|(_pos, entity)| collectibles.contains(*entity))
        {
            collectible_event.send(collectible::Event::Collect(entity));
            game_events.send(GameEvent::NextWave {
                position,
                velocity: vel.0,
            });
            commands.entity(boid_entity).despawn();
            boi_events.send(super::SpawnEvent {
                kind: BoidKind::AngryBoi,
                count: 1,
                position: trans.translation.xy(),
                velocity: vel.0,
            });

            shockwave_events.send(shockwave::Event::Spawn {
                position,
                radius: 100.,
                duration: Duration::from_secs_f32(1.),
                color: Color::BLUE,
                repel: false,
            });
            return;
        }
    }
}
