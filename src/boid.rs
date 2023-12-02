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
            separation: 0.01,
            alignment: 0.01,
            visual_range: 100.0,
            max_velocity: 200.0,
            show_range: true,
            show_direction: false,
        });
        app.add_plugins(ResourceInspectorPlugin::<BoidSettings>::default());
        app.add_systems(Startup, spawn);
        app.add_systems(
            Update,
            (coherence, separation, alignment, bounds, update).chain(),
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

fn spawn(mut commands: Commands, mut rng: ResMut<RngSource>) {
    let rng = &mut **rng;
    for i in 0..100 {
        let mut entity = commands.spawn_empty();
        entity.insert(Name::new(format!("Boid {}", i)));
        entity.insert(Boid);
        let x = rng.gen::<f32>() * 200. - 100.;
        let y = rng.gen::<f32>() * 200. - 100.;
        entity.insert(Velocity(Vec2 { x, y }));

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

fn coherence(
    settings: Res<BoidSettings>,
    mut boids: Query<(&Transform, &mut Velocity), With<Boid>>,
) {
    let count = (boids.iter().count() - 1) as f32;
    let all_masses: Vec2 = boids
        .iter()
        .map(|(transform, _vel)| transform.translation.xy())
        .sum();

    for (boid, mut vel) in boids.iter_mut() {
        let pos = boid.translation.xy();
        let center_of_mass = (all_masses - pos) / count;
        vel.0 += (center_of_mass - pos) * 0.1 * settings.coherence;
    }
}

fn separation(
    settings: Res<BoidSettings>,
    mut boids: Query<(Entity, &Transform, &mut Velocity), With<Boid>>,
    other_boids: Query<(Entity, &Transform), With<Boid>>,
) {
    for (this, this_transform, mut vel) in boids.iter_mut() {
        let mut c = Vec2::ZERO;
        for (other, other_transform) in other_boids.iter() {
            if this == other {
                continue;
            }

            let diff = other_transform.translation.xy() - this_transform.translation.xy();
            if diff.length() < settings.visual_range {
                c = c - diff * settings.separation;
            }
        }
        vel.0 += c;
    }
}

fn alignment(settings: Res<BoidSettings>, mut boids: Query<&mut Velocity, With<Boid>>) {
    let count = (boids.iter().count() - 1) as f32;
    let all_vels: Vec2 = boids.iter().map(|vel| vel.0).sum();

    for mut vel in boids.iter_mut() {
        let this_vel = vel.0;
        let d_vel = all_vels - this_vel;
        let average_vel = d_vel / count;
        vel.0 += ((average_vel - this_vel) / 8.0) * settings.alignment;
    }
}

fn bounds(
    settings: Res<BoidSettings>,
    mut boids: Query<(&mut Transform, &mut Velocity), With<Boid>>,
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

fn update(mut boids: Query<(&mut Transform, &Velocity), With<Boid>>, time: Res<Time>) {
    for (mut boid, vel) in boids.iter_mut() {
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
