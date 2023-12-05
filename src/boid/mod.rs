mod boi;
mod calmboi;

use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_inspector_egui::{prelude::*, quick::ResourceInspectorPlugin, InspectorOptions};
use bevy_spatial::{kdtree::KDTree2, SpatialAccess};
use rand::{Rng, RngCore};

use crate::{collectible::Collectible, track::Tracked};

use self::calmboi::Home;

pub struct BoidPlugin;

impl Plugin for BoidPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Event>();
        app.register_type::<BoidSettings>();
        app.register_type::<Velocity>();
        app.register_type::<Coherence>();
        app.register_type::<Separation>();
        app.register_type::<Alignment>();
        app.insert_resource(BoidSettings {
            coherence: 0.192,
            separation: 0.487,
            alignment: 0.435,
            visual_range: 15.0,
            avoid_range: 10.0,
            max_velocity: 200.0,
            bounds: Rect::new(-500., -300., 500., 300.),
            centering_force: 20.,
            home_range: 200.,
            home_effect: 2.,
        });
        app.insert_resource(BoidDebugSettings {
            cluster_range: false,
            avoid_range: false,
            home_range: false,
            direction: false,
        });
        app.add_systems(
            Update,
            (
                (coherence, coherence_apply).chain(),
                (separation, separation_apply).chain(),
                (alignment, alignment_apply).chain(),
            ),
        );
        app.add_systems(PostUpdate, step);
        app.add_systems(Update, gizmo);
        app.add_plugins(ResourceInspectorPlugin::<BoidSettings>::default());
        app.add_plugins(ResourceInspectorPlugin::<BoidDebugSettings>::default());
        app.add_plugins(boi::Plugin);
        app.add_plugins(calmboi::Plugin);
    }
}

#[derive(Copy, Clone, Debug, Event)]
pub enum Event {
    SpawnBoi,
    SpawnCalmBoi,
}

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource)]
struct BoidDebugSettings {
    cluster_range: bool,
    avoid_range: bool,
    home_range: bool,
    direction: bool,
}

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct BoidSettings {
    #[inspector(min = 0.0, speed = 0.001)]
    pub coherence: f32,
    #[inspector(min = 0.0, speed = 0.001)]
    pub separation: f32,
    #[inspector(min = 0.0, speed = 0.001)]
    pub alignment: f32,
    #[inspector(min = 0.0)]
    pub visual_range: f32,
    #[inspector(min = 0.0)]
    pub avoid_range: f32,
    #[inspector(min = 0.0)]
    pub home_range: f32,
    #[inspector(min = 0.0)]
    pub home_effect: f32,
    #[inspector(min = 0.0)]
    pub max_velocity: f32,
    #[inspector(min = 0.0)]
    pub centering_force: f32,
    pub bounds: Rect,
}

#[derive(Component)]
struct Boid;

#[derive(Bundle)]
pub(self) struct BoidBundle {
    boid: Boid,
    tracked: Tracked,
    velocity: Velocity,
    coherence: Coherence,
    separation: Separation,
    alignemtn: Alignment,
    transform: Transform,
}

impl BoidBundle {
    pub(self) fn new<R: RngCore>(position: Vec2, rng: &mut R) -> Self {
        let x = rng.gen::<f32>() * 200. - 100.;
        let y = rng.gen::<f32>() * 200. - 100.;
        let vel = Vec2 { x, y } * 20.;
        BoidBundle {
            boid: Boid,
            tracked: Tracked,
            velocity: Velocity(vel),
            coherence: Coherence::default(),
            separation: Separation::default(),
            alignemtn: Alignment::default(),
            transform: Transform::from_xyz(position.x, position.y, 0.0),
        }
    }
}

#[derive(Component, Default, Deref, DerefMut, Reflect)]
pub struct Velocity(pub Vec2);

#[derive(Component, Reflect, Default)]
struct Coherence {
    effect: Vec2,
}

fn coherence(
    settings: Res<BoidSettings>,
    quadtree: Res<KDTree2<Tracked>>,
    mut boids: Query<(&Transform, &mut Coherence)>,
) {
    for (transform, mut coherence) in &mut boids {
        let this_pos = transform.translation.xy();

        let nearby = quadtree.within_distance(this_pos, settings.visual_range);
        let count = nearby.len();
        coherence.effect = if count > 1 {
            #[allow(clippy::cast_precision_loss)]
            let count = (count - 1) as f32;
            let masses: Vec2 =
                nearby.into_iter().map(|(pos, _entity)| pos).sum::<Vec2>() - this_pos;
            ((masses / count) - this_pos) * settings.coherence
        } else {
            Vec2::ZERO
        }
    }
}

fn coherence_apply(mut boids: Query<(&mut Velocity, &Coherence)>) {
    for (mut vel, coherence) in &mut boids {
        vel.0 += coherence.effect;
    }
}

#[derive(Component, Reflect, Default)]
struct Separation {
    effect: Vec2,
}

fn separation(
    settings: Res<BoidSettings>,
    quadtree: Res<KDTree2<Tracked>>,
    mut boids: Query<(Entity, &Transform, &mut Separation)>,
) {
    for (this_entity, transform, mut separation) in &mut boids {
        let this_pos = transform.translation.xy();
        let mut c = Vec2::ZERO;

        for (other_pos, _entity) in quadtree
            .within_distance(this_pos, settings.avoid_range)
            .into_iter()
            .filter(|(_pos, entity)| {
                entity
                    .map(|entity| entity != this_entity)
                    .unwrap_or_default()
            })
        {
            c += this_pos - other_pos;
        }

        separation.effect = c * settings.separation;
    }
}

fn separation_apply(mut boids: Query<(&mut Velocity, &Separation)>) {
    for (mut vel, separtion) in &mut boids {
        vel.0 += separtion.effect;
    }
}

#[derive(Component, Reflect, Default)]
pub struct Alignment {
    pub effect: Vec2,
}

fn alignment(
    settings: Res<BoidSettings>,
    quadtree: Res<KDTree2<Tracked>>,
    mut boids: Query<(&Transform, &Velocity, &mut Alignment)>,
    other: Query<&Velocity, With<Alignment>>,
) {
    for (transform, vel, mut alignment) in &mut boids {
        let this_pos = transform.translation.xy();
        let mut velocities = -vel.0;
        let mut count = -1;
        for vel in quadtree
            .within_distance(this_pos, settings.visual_range)
            .into_iter()
            .filter_map(|(_, entity)| entity)
            .filter_map(|entity| other.get(entity).ok())
        {
            velocities += vel.0;
            count += 1;
        }

        alignment.effect = if count > 0 {
            #[allow(clippy::cast_precision_loss)]
            let count = count as f32;
            (velocities / count) * settings.alignment
        } else {
            Vec2::ZERO
        }
    }
}

fn alignment_apply(mut boids: Query<(&mut Velocity, &Alignment)>) {
    for (mut vel, alignment) in &mut boids {
        vel.0 += alignment.effect;
    }
}

fn step(
    settings: Res<BoidSettings>,
    mut boids: Query<(&mut Transform, &mut Velocity), With<Boid>>,
    time: Res<Time>,
) {
    for (mut transform, mut vel) in &mut boids {
        let pos = transform.translation.xy();
        if !settings.bounds.contains(pos) {
            vel.0 += -transform.translation.xy().normalize_or_zero() * settings.centering_force;
            vel.0 = vel.clamp_length_max(settings.max_velocity * 5.);
        } else {
            vel.0 = vel.clamp_length_max(settings.max_velocity);
        }
        transform.translation += vel.extend(0.0) * time.delta_seconds();
        transform.rotation = Quat::from_axis_angle(Vec3::Z, vel.0.y.atan2(vel.0.x) + PI * 1.5);
    }
}

fn gizmo(
    mut gizmos: Gizmos,
    settings: Res<BoidSettings>,
    debug_settings: Res<BoidDebugSettings>,
    boids: Query<(&Transform, &Velocity), With<Boid>>,
    homing: Query<&Transform, With<Home<Collectible>>>,
) {
    for (transform, velocity) in boids.iter() {
        if debug_settings.cluster_range {
            gizmos.circle_2d(
                transform.translation.xy(),
                settings.visual_range,
                Color::rgba(1.0, 0.0, 0.0, 0.1),
            );
        }
        if debug_settings.avoid_range {
            gizmos.circle_2d(
                transform.translation.xy(),
                settings.avoid_range,
                Color::rgba(1.0, 1.0, 0.0, 0.1),
            );
        }
        if debug_settings.direction {
            gizmos.line_2d(
                transform.translation.xy(),
                transform.translation.xy() + velocity.0.normalize_or_zero() * 10.0,
                Color::RED,
            );
        }
    }

    if debug_settings.home_range {
        for transform in homing.iter() {
            gizmos.circle_2d(
                transform.translation.xy(),
                settings.home_range,
                Color::rgba(0.0, 0.0, 1.0, 0.1),
            );
        }
    }
}
