use bevy::prelude::*;
use bevy_inspector_egui::{prelude::*, quick::ResourceInspectorPlugin, InspectorOptions};
use rand::Rng;

use crate::rng::RngSource;

pub struct BoidPlugin;

impl Plugin for BoidPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<BoidSettings>();
        app.register_type::<Velocity>();
        app.insert_resource(BoidSettings {
            coherence: 0.7,
            separation: 0.01,
            alignment: 0.34,
            center_force_stength: 0.00,
            visual_range: 100.0,
            max_velocity: 200.0,
            show_range: false,
            show_direction: false,
        });
        app.add_plugins(ResourceInspectorPlugin::<BoidSettings>::default());
        app.add_systems(Startup, spawn);
        app.add_systems(PreUpdate, nearby);
        app.add_systems(Update, (update, step).chain());
        app.add_systems(Update, gizmo);
    }
}

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct BoidSettings {
    #[inspector(min = 0.0, speed = 0.001)]
    coherence: f32,
    #[inspector(min = 0.0, speed = 0.001)]
    separation: f32,
    #[inspector(min = 0.0, speed = 0.001)]
    alignment: f32,
    #[inspector(min = 0.0, speed = 0.001)]
    center_force_stength: f32,
    #[inspector(min = 0.0)]
    visual_range: f32,
    #[inspector(min = 0.0)]
    max_velocity: f32,
    show_range: bool,
    show_direction: bool,
}

#[derive(Component)]
struct Boid;

#[derive(Component, Default, Deref, DerefMut, Reflect)]
struct Velocity(pub Vec2);

#[derive(Component, Default, Deref, DerefMut)]
struct NextVelocity(pub Vec2);

#[derive(Component, Default, Deref, DerefMut)]
struct Nearby(pub Vec<Entity>);

fn spawn(mut commands: Commands, asset_server: Res<AssetServer>, mut rng: ResMut<RngSource>) {
    let rng = &mut **rng;
    for i in 0..1000 {
        let mut entity = commands.spawn_empty();
        entity.insert(Name::new(format!("Boid {}", i)));
        entity.insert(SpriteBundle {
            texture: asset_server.load("boid.png"),
            ..default()
        });
        entity.insert(Boid);
        entity.insert(Nearby::default());
        let x = rng.gen::<f32>() * 200. - 100.;
        let y = rng.gen::<f32>() * 200. - 100.;
        let vel = Vec2 { x, y };
        entity.insert(Velocity(vel * 20.0));
        entity.insert(NextVelocity(vel * 20.0));
        entity.insert(TransformBundle {
            local: Transform::from_xyz(
                rng.gen::<f32>() * 1000. - 500.,
                rng.gen::<f32>() * 600. - 300.,
                0.0,
            ),
            ..default()
        });
    }
}

fn nearby(
    settings: Res<BoidSettings>,
    mut boids: Query<(Entity, &Transform, &mut Nearby), With<Boid>>,
    other: Query<(Entity, &Transform), With<Boid>>,
) {
    for (this_entity, this_trans, mut nearby) in boids.iter_mut() {
        nearby.0.clear();
        let this_pos = this_trans.translation.xy();
        for (other_entity, other_trans) in other.iter() {
            if this_entity == other_entity {
                continue;
            }

            let other_pos = other_trans.translation.xy();
            if (other_pos - this_pos).length_squared()
                > settings.visual_range * settings.visual_range
            {
                continue;
            }

            nearby.0.push(other_entity);
        }
    }
}

fn coherence(
    settings: &BoidSettings,
    transform: &Transform,
    nearby: &Nearby,
    vel: &mut NextVelocity,
    other: &Query<(&Transform, &Velocity), With<Boid>>,
) {
    let count = nearby.len();
    if count == 0 {
        return;
    }
    let this_pos = transform.translation.xy();
    let center_of_mass: Vec2 = nearby
        .iter()
        .filter_map(|other_entity| other.get(*other_entity).ok())
        .map(|(other_trans, _)| other_trans.translation.xy())
        .sum::<Vec2>()
        / count as f32;
    vel.0 += (center_of_mass - this_pos) * 0.1 * settings.coherence;
}

fn separation(
    settings: &BoidSettings,
    transform: &Transform,
    nearby: &Nearby,
    vel: &mut NextVelocity,
    other: &Query<(&Transform, &Velocity), With<Boid>>,
) {
    let this_pos = transform.translation.xy();
    let mut c = Vec2::ZERO;
    for nearby_pos in nearby
        .iter()
        .filter_map(|entity| other.get(*entity).ok())
        .map(|(other_trans, _)| other_trans.translation.xy())
    {
        let diff = nearby_pos - this_pos;
        c -= diff * 0.1 * settings.separation;
    }
    vel.0 += c;
}

fn alignment(
    settings: &BoidSettings,
    transform: &Transform,
    nearby: &Nearby,
    vel: &mut NextVelocity,
    other: &Query<(&Transform, &Velocity), With<Boid>>,
) {
    let count = nearby.len();
    if count == 0 {
        return;
    }
    let this_pos = transform.translation.xy();
    let average_vel: Vec2 = nearby
        .iter()
        .filter_map(|other_entity| other.get(*other_entity).ok())
        .map(|(_, other_vel)| other_vel.0)
        .sum::<Vec2>()
        / count as f32;
    vel.0 += (average_vel - this_pos) * 0.1 * settings.alignment;
}

fn bounds(settings: &BoidSettings, transform: &Transform, vel: &mut Vec2) {
    const CENTER_FORCE: f32 = 100.0;
    let pos = transform.translation.xy();
    if pos.x < -500.0 {
        vel.x += CENTER_FORCE;
    } else if pos.x > 500.0 {
        vel.x -= CENTER_FORCE;
    }

    if pos.y < -300.0 {
        vel.y += CENTER_FORCE;
    } else if pos.y > 300.0 {
        vel.y -= CENTER_FORCE;
    }

    *vel = vel.clamp_length_max(settings.max_velocity);
}

fn update(
    settings: Res<BoidSettings>,
    mut boids: Query<(&Transform, &mut NextVelocity, &Nearby), With<Boid>>,
    other: Query<(&Transform, &Velocity), With<Boid>>,
) {
    for (transform, mut next_vel, nearby) in boids.iter_mut() {
        coherence(&settings, &transform, nearby, &mut next_vel, &other);
        separation(&settings, &transform, nearby, &mut next_vel, &other);
        alignment(&settings, &transform, nearby, &mut next_vel, &other);
        bounds(&settings, &transform, &mut next_vel.0);
    }
}

fn step(
    mut boids: Query<(&mut Transform, &NextVelocity, &mut Velocity), With<Boid>>,
    time: Res<Time>,
) {
    for (mut transform, next_vel, mut vel) in boids.iter_mut() {
        vel.0 = next_vel.0;
        transform.translation += vel.extend(0.0) * time.delta_seconds();
    }
}

fn gizmo(
    mut gizmos: Gizmos,
    settings: Res<BoidSettings>,
    boids: Query<(&Transform, &Velocity), With<Boid>>,
) {
    for (transform, velocity) in boids.iter() {
        if settings.show_range {
            gizmos.circle_2d(
                transform.translation.xy(),
                settings.visual_range,
                Color::rgba(1.0, 0.0, 0.0, 0.1),
            );
        }
        if settings.show_direction {
            gizmos.line_2d(
                transform.translation.xy(),
                transform.translation.xy() + velocity.0.normalize_or_zero() * 10.0,
                Color::RED,
            );
        }
    }
}
