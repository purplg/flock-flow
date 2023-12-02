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
            coherence: 0.1,
            separation: 0.0,
            alignment: 0.00,
            visual_range: 100.0,
            max_velocity: 200.0,
            show_range: true,
            show_direction: false,
        });
        app.add_plugins(ResourceInspectorPlugin::<BoidSettings>::default());
        app.add_systems(Startup, spawn);
        app.add_systems(
            Update,
            (nearby, coherence, separation, alignment, bounds, update).chain(),
        );
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
    #[inspector(min = 0.0)]
    visual_range: f32,
    #[inspector(min = 0.0)]
    max_velocity: f32,
    show_range: bool,
    show_direction: bool,
}

#[derive(Component)]
struct Boid;

#[derive(Component, Deref, DerefMut, Reflect)]
struct Velocity(pub Vec2);

#[derive(Component, Default, Deref, DerefMut)]
struct NextVelocity(pub Vec2);

#[derive(Component, Default, Deref, DerefMut)]
struct Nearby(pub Vec<Entity>);

fn spawn(mut commands: Commands, mut rng: ResMut<RngSource>) {
    let rng = &mut **rng;
    for i in 0..100 {
        let mut entity = commands.spawn_empty();
        entity.insert(Name::new(format!("Boid {}", i)));
        entity.insert(Boid);
        entity.insert(Nearby::default());
        let x = rng.gen::<f32>() * 200. - 100.;
        let y = rng.gen::<f32>() * 200. - 100.;
        entity.insert(Velocity(Vec2 { x, y }));
        entity.insert(NextVelocity::default());
        entity.insert(TransformBundle {
            local: Transform::from_xyz(
                rng.gen::<f32>() * 100.,
                rng.gen::<f32>() * 100.,
                rng.gen::<f32>() * 100.,
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
            if (other_pos - this_pos).length() > settings.visual_range {
                continue;
            }

            nearby.0.push(other_entity);
        }
    }
}

fn coherence(
    settings: Res<BoidSettings>,
    mut boids: Query<(&Transform, &Nearby, &mut NextVelocity), With<Boid>>,
    other: Query<&Transform, With<Boid>>,
) {
    for (this_trans, nearby, mut this_vel) in
        boids.iter_mut().filter(|(_, nearby, _)| nearby.len() > 0)
    {
        let count = nearby.len();
        let this_pos = this_trans.translation.xy();
        let center_of_mass: Vec2 = nearby
            .iter()
            .filter_map(|other_entity| other.get(*other_entity).ok())
            .map(|other_trans| other_trans.translation.xy())
            .sum::<Vec2>()
            / count as f32;
        this_vel.0 += (center_of_mass - this_pos) * 0.1 * settings.coherence;
    }
}

fn separation(
    settings: Res<BoidSettings>,
    mut boids: Query<(&Transform, &Nearby, &mut NextVelocity), With<Boid>>,
    other: Query<&Transform, With<Boid>>,
) {
    for (this_trans, nearby, mut this_vel) in
        boids.iter_mut().filter(|(_, nearby, _)| nearby.len() > 0)
    {
        let this_pos = this_trans.translation.xy();
        let mut c = Vec2::ZERO;
        for nearby_pos in nearby
            .iter()
            .filter_map(|entity| other.get(*entity).ok())
            .map(|other_trans| other_trans.translation.xy())
        {
            let diff = nearby_pos - this_pos;
            c = c - diff * 0.1 * settings.separation;
        }
        this_vel.0 += c;
    }
}

fn alignment(
    settings: Res<BoidSettings>,
    mut boids: Query<(&Transform, &Nearby, &mut NextVelocity), With<Boid>>,
    other: Query<&Velocity, With<Boid>>,
) {
    for (this_trans, nearby, mut this_vel) in
        boids.iter_mut().filter(|(_, nearby, _)| nearby.len() > 0)
    {
        let count = nearby.len();
        let this_pos = this_trans.translation.xy();
        let average_vel: Vec2 = nearby
            .iter()
            .filter_map(|other_entity| other.get(*other_entity).ok())
            .map(|other_vel| other_vel.0)
            .sum::<Vec2>()
            / count as f32;
        this_vel.0 += (average_vel - this_pos) * 0.1 * settings.alignment;
    }
}

fn bounds(
    settings: Res<BoidSettings>,
    mut boids: Query<(&mut Transform, &mut NextVelocity), With<Boid>>,
) {
    for (mut boid, mut vel) in boids.iter_mut() {
        let pos = boid.translation.xy();
        if pos.x < -500.0 {
            boid.translation.x = -500.0;
            vel.x *= -1.0;
        }
        if pos.x > 500.0 {
            boid.translation.x = 500.0;
            vel.x *= -1.0;
        }
        if pos.y < -300.0 {
            boid.translation.y = -300.0;
            vel.y *= -1.0;
        }
        if pos.y > 300.0 {
            boid.translation.y = 300.0;
            vel.y *= -1.0;
        }

        vel.0 = vel.0.clamp_length_max(settings.max_velocity);
    }
}

fn update(
    mut boids: Query<(&mut Transform, &NextVelocity, &mut Velocity), With<Boid>>,
    time: Res<Time>,
) {
    for (mut boid, next_vel, mut vel) in boids.iter_mut() {
        vel.0 = next_vel.0;
        boid.translation += vel.extend(0.0) * time.delta_seconds();
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
        gizmos.circle_2d(transform.translation.xy(), 2.0, Color::RED);
        if settings.show_direction {
            gizmos.line_2d(
                transform.translation.xy(),
                transform.translation.xy() + velocity.0.normalize_or_zero() * 10.0,
                Color::RED,
            );
        }
    }
}
