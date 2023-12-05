use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_spatial::kdtree::KDTree2;
use bevy_spatial::SpatialAccess;
use rand::Rng;

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
    other: Query<&Collectible, Without<Home<T>>>,
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
    asset_server: Res<AssetServer>,
    mut rng: ResMut<RngSource>,
    mut events: EventReader<super::Event>,
) {
    for event in events.read() {
        if let super::Event::SpawnCalmBoi = event {
            let angle: f32 = rng.gen();
            let pos = Vec2 {
                x: angle.cos(),
                y: angle.sin(),
            } * 1000.;
            let mut entity = commands.spawn_empty();
            entity.insert(Name::new("CalmBoi"));
            entity.insert(CalmBoi);
            let home: Home<Collectible> = Home::default();
            entity.insert(home);
            entity.insert(SpriteBundle {
                texture: asset_server.load("calmboi.png"),
                ..default()
            });
            entity.insert(BoidBundle::new(pos, &mut **rng));
        }
    }
}

fn collect(
    quadtree: Res<KDTree2<Tracked>>,
    boid: Query<&Transform, With<CalmBoi>>,
    collectibles: Query<Entity, (With<Collectible>, Without<collectible::Cooldown>)>,
    mut collectible_event: EventWriter<collectible::Event>,
    mut game_events: EventWriter<GameEvent>,
) {
    for trans in boid.iter() {
        let pos = trans.translation.xy();
        for entity in quadtree
            .within_distance(pos, 32.0)
            .into_iter()
            .filter_map(|(_, entity)| entity)
            .filter(|entity| collectibles.contains(*entity))
        {
            collectible_event.send(collectible::Event::Collect(entity));
            game_events.send(GameEvent::NextWave);
        }
    }
}
