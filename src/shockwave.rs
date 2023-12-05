use std::time::Duration;

use bevy::prelude::*;
use bevy_spatial::{kdtree::KDTree2, SpatialAccess};
use interpolation::{Ease, Lerp};

use crate::{boid::Velocity, track::Tracked};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Event>();
        app.add_systems(Update, avoid);
        app.add_systems(Update, expiration);
        app.add_systems(Update, spawn.run_if(on_event::<Event>()));
        app.add_systems(Update, gizmo);
    }
}

#[derive(Debug, Event)]
pub enum Event {
    Spawn {
        position: Vec2,
        radius: f32,
        duration: Duration,
    },
}

#[derive(Component)]
struct Shockwave {
    duration: f32,
    remaining: f32,
    max_radius: f32,
    active_radius: f32,
}

impl Shockwave {
    fn new(duration: Duration, radius: f32) -> Self {
        Self {
            duration: duration.as_secs_f32(),
            remaining: duration.as_secs_f32(),
            max_radius: radius,
            active_radius: 0.0,
        }
    }
}

fn spawn(mut commands: Commands, mut events: EventReader<Event>) {
    for event in events.read() {
        match event {
            Event::Spawn {
                position,
                radius,
                duration,
            } => {
                let mut entity = commands.spawn_empty();
                entity.insert(Name::new("Shockwave"));
                entity.insert(Shockwave::new(*duration, *radius));
                entity.insert(TransformBundle::from_transform(
                    Transform::from_translation(position.extend(0.0)),
                ));
            }
        }
    }
}

fn expiration(
    mut commands: Commands,
    mut shockwaves: Query<(Entity, &mut Shockwave)>,
    time: Res<Time>,
) {
    for (entity, mut shockwave) in shockwaves.iter_mut() {
        shockwave.remaining -= time.delta_seconds();
        let progress = shockwave.remaining / shockwave.duration;
        shockwave.active_radius = shockwave.max_radius.lerp(&32.0, &progress.quadratic_in());
        if shockwave.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn avoid(
    quadtree: Res<KDTree2<Tracked>>,
    mut boids: Query<&mut Velocity>,
    shockwaves: Query<(&Transform, &Shockwave)>,
) {
    for (shockwave_trans, shockwave) in shockwaves.iter() {
        let shock_pos = shockwave_trans.translation.xy();
        for (boid_pos, boid_entity) in quadtree
            .within_distance(shock_pos, shockwave.active_radius)
            .into_iter()
            .filter_map(|(pos, entity)| entity.map(|entity| (pos, entity)))
        {
            if let Ok(mut vel) = boids.get_mut(boid_entity) {
                vel.0 -= (shock_pos - boid_pos).normalize_or_zero() * 100.0;
            }
        }
    }
}

fn gizmo(mut gizmos: Gizmos, shockwaves: Query<(&Transform, &Shockwave)>) {
    for (transform, shockwave) in shockwaves.iter() {
        gizmos.circle_2d(
            transform.translation.xy(),
            shockwave.active_radius,
            Color::ORANGE_RED,
        );
    }
}
