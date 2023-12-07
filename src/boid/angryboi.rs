use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_spatial::kdtree::KDTree2;
use bevy_spatial::SpatialAccess;

use crate::assets::Images;

use crate::player::Player;
use crate::track::Tracked;

use super::{BoidBundle, BoidSettings, Velocity};

#[derive(Component)]
struct AngryBoi;

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn);
        app.add_systems(Update, home::<Player>);
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

fn spawn(mut commands: Commands, images: Res<Images>, mut events: EventReader<super::SpawnEvent>) {
    for event in events.read() {
        if let super::SpawnEvent::AngryBoi { position, velocity } = event {
            let mut entity = commands.spawn_empty();
            entity.insert(Name::new("AngryBoi"));
            entity.insert(AngryBoi);
            let home: Home<Player> = Home::default();
            entity.insert(home);
            entity.insert(SpriteBundle {
                texture: images.angryboi.clone(),
                ..default()
            });
            entity.insert(BoidBundle::new(*position, *velocity));
        }
    }
}
