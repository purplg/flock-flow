use std::f32::consts::PI;
use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_spatial::kdtree::KDTree2;
use bevy_spatial::SpatialAccess;
use rand::Rng;

use crate::assets::Images;
use crate::collectible::{self, Collectible};

use crate::{rng::RngSource, track::Tracked, GameEvent};

use super::{BoidBundle, BoidSettings, Velocity};

#[derive(Component)]
struct CalmBoi;

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn);
        app.add_systems(Update, collect);
        app.add_systems(Update, home::<Collectible>);
    }
}

#[derive(Component, Default)]
pub struct Home<T: Component + Default> {
    _target: PhantomData<T>,
}

fn home<T: Component + Default>(
    settings: Res<BoidSettings>,
    quadtree: Res<KDTree2<Tracked>>,
    mut homing: Query<(&Transform, &mut Velocity), With<Home<T>>>,
    other: Query<&T>,
) {
    for (transform, mut vel) in &mut homing {
        let this_pos = transform.translation.xy();
        let mut effect = Vec2::ZERO;
        for target_pos in quadtree
            .within_distance(this_pos, settings.home_range)
            .into_iter()
            .filter_map(|(pos, entity)| entity.map(|entity| (pos, entity)))
            .filter_map(|(pos, entity)| other.get(entity).map(|_| pos).ok())
        {
            let dir = (target_pos - this_pos).normalize_or_zero();
            effect += dir;
        }

        vel.0 += effect * settings.home_effect;
    }
}

fn spawn(
    mut commands: Commands,
    images: Res<Images>,
    mut rng: ResMut<RngSource>,
    mut events: EventReader<super::Event>,
) {
    for event in events.read() {
        if let super::Event::SpawnCalmBoi = event {
            let mut entity = commands.spawn_empty();

            entity.insert(Name::new("CalmBoi"));
            entity.insert(CalmBoi);
            let home: Home<Collectible> = Home::default();
            entity.insert(home);
            entity.insert(SpriteBundle {
                texture: images.calmboi.clone(),
                ..default()
            });

            let angle = rng.gen::<f32>() * PI * 2.;
            entity.insert(BoidBundle::new(
                Vec2 {
                    x: angle.cos(),
                    y: angle.sin(),
                } * 1000.,
                Vec2 {
                    x: rng.gen::<f32>() * 200. - 100.,
                    y: rng.gen::<f32>() * 200. - 100.,
                } * 20.,
            ));
        }
    }
}

fn collect(
    mut commands: Commands,
    quadtree: Res<KDTree2<Tracked>>,
    boid: Query<(Entity, &Transform, &Velocity), With<CalmBoi>>,
    collectibles: Query<Entity, (With<Collectible>, Without<collectible::Cooldown>)>,
    mut collectible_event: EventWriter<collectible::Event>,
    mut game_events: EventWriter<GameEvent>,
    mut boi_events: EventWriter<super::Event>,
) {
    for (boid_entity, trans, vel) in boid.iter() {
        let pos = trans.translation.xy();
        for entity in quadtree
            .within_distance(pos, 32.0)
            .into_iter()
            .filter_map(|(_, entity)| entity)
            .filter(|entity| collectibles.contains(*entity))
        {
            collectible_event.send(collectible::Event::Collect(entity));
            game_events.send(GameEvent::NextWave);
            commands.entity(boid_entity).despawn();
            boi_events.send(super::Event::SpawnAngryBoi {
                position: trans.translation.xy(),
                velocity: vel.0,
            });
        }
    }
}
