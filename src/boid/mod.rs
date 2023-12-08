mod angryboi;
mod boi;
mod calmboi;

use std::{f32::consts::PI, marker::PhantomData};

use bevy::prelude::*;
use bevy_spatial::{kdtree::KDTree2, SpatialAccess};

#[cfg(feature = "inspector")]
use crate::collectible::Collectible;
#[cfg(feature = "inspector")]
use bevy_inspector_egui::{prelude::*, quick::ResourceInspectorPlugin, InspectorOptions};

use crate::{track::Tracked, velocity::Velocity};

pub struct BoidPlugin;

impl Plugin for BoidPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEvent>();
        app.insert_resource(BoidSettings {
            coherence: 0.192,
            separation: 0.487,
            alignment: 0.435,
            visual_range: 15.0,
            avoid_range: 10.0,
            max_speed: 200.0,
            bounds: Rect::new(-500., -300., 500., 300.),
            centering_force: 20.,
            home_range: 300.,
            home_effect: 2.,
        });
        app.add_systems(
            Update,
            (
                (coherence, coherence_apply).chain(),
                (separation, separation_apply).chain(),
                (alignment, alignment_apply).chain(),
            ),
        );
        app.add_systems(PostUpdate, update);
        app.add_plugins(boi::Plugin);
        app.add_plugins(calmboi::Plugin);
        app.add_plugins(angryboi::Plugin);

        #[cfg(feature = "inspector")]
        {
            app.register_type::<BoidSettings>();
            app.register_type::<Coherence>();
            app.register_type::<Separation>();
            app.register_type::<Alignment>();
            app.add_plugins(ResourceInspectorPlugin::<BoidSettings>::default());
            app.add_plugins(ResourceInspectorPlugin::<BoidDebugSettings>::default());
            app.add_systems(Update, gizmo);
            app.insert_resource(BoidDebugSettings {
                cluster_range: false,
                avoid_range: false,
                home_range: false,
                direction: false,
            });
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum BoidKind {
    Boi,
    CalmBoi,
    AngryBoi,
}

#[derive(Copy, Clone, Debug, Event)]
pub struct SpawnEvent {
    pub kind: BoidKind,
    pub count: u32,
    pub position: Vec2,
    pub velocity: Vec2,
}

#[cfg(feature = "inspector")]
#[derive(Resource, Default, Reflect, InspectorOptions)]
#[reflect(Resource)]
#[allow(clippy::struct_excessive_bools)]
struct BoidDebugSettings {
    cluster_range: bool,
    avoid_range: bool,
    home_range: bool,
    direction: bool,
}

#[derive(Resource, Default)]
#[cfg_attr(
    feature = "inspector",
    derive(Reflect, InspectorOptions),
    reflect(Resource, InspectorOptions)
)]
pub struct BoidSettings {
    #[cfg_attr(feature = "inspector", inspector(min = 0.0, speed = 0.001))]
    pub coherence: f32,
    #[cfg_attr(feature = "inspector", inspector(min = 0.0, speed = 0.001))]
    pub separation: f32,
    #[cfg_attr(feature = "inspector", inspector(min = 0.0, speed = 0.001))]
    pub alignment: f32,
    #[cfg_attr(feature = "inspector", inspector(min = 0.0))]
    pub visual_range: f32,
    #[cfg_attr(feature = "inspector", inspector(min = 0.0))]
    pub avoid_range: f32,
    #[cfg_attr(feature = "inspector", inspector(min = 0.0))]
    pub home_range: f32,
    #[cfg_attr(feature = "inspector", inspector(min = 0.0))]
    pub home_effect: f32,
    #[cfg_attr(feature = "inspector", inspector(min = 0.0))]
    pub max_speed: f32,
    #[cfg_attr(feature = "inspector", inspector(min = 0.0))]
    pub centering_force: f32,
    pub bounds: Rect,
}

#[derive(Component)]
struct Boid;

#[derive(Bundle)]
struct BoidBundle {
    boid: Boid,
    tracked: Tracked,
    velocity: Velocity,
    coherence: Coherence,
    separation: Separation,
    alignemtn: Alignment,
    transform: Transform,
}

impl BoidBundle {
    pub(self) fn new(position: Vec2, velocity: Vec2) -> Self {
        BoidBundle {
            boid: Boid,
            tracked: Tracked,
            velocity: Velocity(velocity),
            coherence: Coherence::default(),
            separation: Separation::default(),
            alignemtn: Alignment::default(),
            transform: Transform::from_xyz(position.x, position.y, 0.0),
        }
    }
}

#[derive(Component, Default)]
#[cfg_attr(feature = "inspector", derive(Reflect))]
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

#[derive(Component, Default)]
#[cfg_attr(feature = "inspector", derive(Reflect))]
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

#[derive(Component, Default)]
#[cfg_attr(feature = "inspector", derive(Reflect))]
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

#[derive(Component, Default)]
struct Home<T: Component + Default> {
    influence: f32,
    _target: PhantomData<T>,
}

impl<T: Component + Default> Home<T> {
    pub fn new(influence: f32) -> Self {
        Self {
            influence,
            ..default()
        }
    }
}

fn home<T: Component + Default>(
    settings: Res<BoidSettings>,
    quadtree: Res<KDTree2<Tracked>>,
    mut homing: Query<(&Transform, &mut Velocity, &Home<T>)>,
    other: Query<&T>,
) {
    for (transform, mut vel, home) in &mut homing {
        let this_pos = transform.translation.xy();
        let mut effect = Vec2::ZERO;
        for target_pos in quadtree
            .within_distance(this_pos, settings.home_range)
            .into_iter()
            .filter_map(|(pos, entity)| entity.map(|entity| (pos, entity)))
            .filter_map(|(pos, entity)| other.get(entity).map(|_| pos).ok())
        {
            let dir = (target_pos - this_pos).normalize_or_zero();
            effect += dir * home.influence;
        }

        vel.0 += effect * settings.home_effect;
    }
}

fn update(
    settings: Res<BoidSettings>,
    mut boids: Query<(&mut Transform, &mut Velocity), With<Boid>>,
) {
    for (mut transform, mut vel) in &mut boids {
        if settings.bounds.contains(transform.translation.xy()) {
            vel.0 = vel.clamp_length_max(settings.max_speed);
        } else {
            vel.0 += -transform.translation.xy().normalize_or_zero() * settings.centering_force;
            vel.0 = vel.clamp_length_max(settings.max_speed * 3.);
        }
        transform.rotation = Quat::from_axis_angle(Vec3::Z, vel.0.y.atan2(vel.0.x) + PI * 1.5);
    }
}

#[cfg(feature = "inspector")]
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
