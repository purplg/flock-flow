use std::{f32::consts::PI, time::Duration};

use bevy::prelude::*;
use bevy_spatial::{kdtree::KDTree2, SpatialAccess};
use interpolation::{Ease, Lerp};
use rand::Rng;

use crate::{assets::Images, player::Player, rng::RngSource, track::Tracked, velocity::Velocity};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Event>();
        app.add_systems(Update, avoid);
        app.add_systems(Update, expiration);
        app.add_systems(Update, smoke);
        app.add_systems(Update, spawn.run_if(on_event::<Event>()));
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
struct Smoke {
    destination_offset: Vec3,
    scale: f32,
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

fn spawn(
    mut commands: Commands,
    images: Res<Images>,
    mut events: EventReader<Event>,
    mut rng: ResMut<RngSource>,
) {
    for event in events.read() {
        match event {
            Event::Spawn {
                position: center,
                radius,
                duration,
            } => {
                assert!(radius > &0.0);
                let mut entity = commands.spawn_empty();
                entity.insert(Name::new("Shockwave"));
                entity.insert(Shockwave::new(*duration, *radius));
                entity.insert(TransformBundle::from_transform(
                    Transform::from_translation(center.extend(0.0)),
                ));
                entity.insert(InheritedVisibility::VISIBLE);

                entity.with_children(|parent| {
                    let density = radius.floor();
                    #[allow(clippy::cast_possible_truncation)]
                    let count = density.floor() as i16;
                    for i in 0..count {
                        let offset = rng.gen::<f32>().elastic_out();
                        let angle = (f32::from(i) / density) * PI * 2.0;
                        let scale = rng.gen::<f32>() * 0.5;
                        let mut smoke = parent.spawn_empty();
                        smoke.insert(Name::new("Smoke"));
                        smoke.insert(Smoke {
                            destination_offset: Vec3 {
                                x: angle.cos(),
                                y: angle.sin(),
                                z: 0.0,
                            } * offset,
                            scale,
                        });
                        smoke.insert(SpriteBundle {
                            texture: images.smoke.clone(),
                            transform: Transform {
                                scale: Vec3::ONE * scale,
                                rotation: Quat::from_axis_angle(
                                    Vec3::Z,
                                    rng.gen::<f32>() * PI * 2.,
                                ),
                                ..default()
                            },
                            ..default()
                        });
                    }
                });
            }
        }
    }
}

fn expiration(
    mut commands: Commands,
    mut shockwaves: Query<(Entity, &mut Shockwave)>,
    time: Res<Time>,
) {
    for (entity, mut shockwave) in &mut shockwaves {
        shockwave.remaining -= time.delta_seconds();
        let progress = shockwave.remaining / shockwave.duration;
        shockwave.active_radius = shockwave.max_radius.lerp(&32.0, &progress.quadratic_in());
        if shockwave.remaining <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn smoke(
    mut smoke: Query<(&Parent, &mut Transform, &Smoke)>,
    shockwaves: Query<&Shockwave>,
    time: Res<Time>,
) {
    for (parent, mut transform, smoke) in &mut smoke {
        let shockwave = shockwaves.get(parent.get()).unwrap();
        transform.translation = smoke.destination_offset * shockwave.active_radius;
        transform.rotation *=
            Quat::from_axis_angle(Vec3::Z, time.delta_seconds() * 20.0 * shockwave.remaining);
        transform.scale = Vec3::ONE * smoke.scale * shockwave.remaining;
    }
}

fn avoid(
    quadtree: Res<KDTree2<Tracked>>,
    mut boids: Query<&mut Velocity, Without<Player>>,
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
