use bevy::prelude::*;
use rand::Rng;

use crate::rng::RngSource;

pub struct BoidPlugin;

impl Plugin for BoidPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BoidSettings {
            coherence: 0.5,
            separation: 0.0,
            alignment: 0.0,
        });
        app.add_systems(Startup, spawn);
        app.add_systems(
            Update,
            (start, coherence, separation, alignment, update).chain(),
        );
        app.add_systems(Update, gizmo);
    }
}

#[derive(Resource)]
struct BoidSettings {
    coherence: f32,
    separation: f32,
    alignment: f32,
}

#[derive(Component)]
struct Boid;

#[derive(Component, Deref, DerefMut)]
struct Velocity(pub Vec2);

fn spawn(mut commands: Commands, mut rng: ResMut<RngSource>) {
    let rng = &mut **rng;
    for i in 0..10 {
        let mut entity = commands.spawn_empty();
        entity.insert(Name::new(format!("Boid {}", i)));
        entity.insert(Boid);
        entity.insert(Velocity(Vec2 {
            x: rng.gen::<f32>() * 10. - 5.,
            y: rng.gen::<f32>() * 10. - 5.,
        }));

        entity.insert(TransformBundle {
            local: Transform::from_xyz(
                rng.gen::<f32>() * 10.,
                rng.gen::<f32>() * 10.,
                rng.gen::<f32>() * 10.,
            ),
            ..default()
        });
    }
}

fn start(mut boids: Query<(&Transform, &mut Velocity), With<Boid>>) {
    for (boid, mut vel) in boids.iter_mut() {
        let pos = boid.translation.xy();
        if pos.x < -400.0 {
            vel.x *= -1.0;
        }
        if pos.x > 400.0 {
            vel.x *= -1.0;
        }
        if pos.y < -300.0 {
            vel.y *= -1.0;
        }
        if pos.y > 300.0 {
            vel.y *= -1.0;
        }
    }
}

fn coherence(
    settings: Res<BoidSettings>,
    mut boids: Query<(&Transform, &mut Velocity), With<Boid>>,
    time: Res<Time>,
) {
    let count = (boids.iter().count() - 1) as f32;
    let all_masses: Vec2 = boids
        .iter()
        .map(|(transform, _vel)| transform.translation.xy())
        .sum();

    for (boid, mut vel) in boids.iter_mut() {
        let pos = boid.translation.xy();
        let center_of_mass = (all_masses - pos) / count;
        vel.0 += (center_of_mass - pos) * 0.1 * time.delta_seconds();
    }
}

fn separation(
    settings: Res<BoidSettings>,
    mut boids: Query<(Entity, &Transform, &mut Velocity), With<Boid>>,
    other_boids: Query<(Entity, &Transform), With<Boid>>,
    time: Res<Time>,
) {
    for (this, this_transform, mut vel) in boids.iter_mut() {
        let mut c = Vec2::ZERO;
        for (other, other_transform) in other_boids.iter() {
            if this == other {
                continue;
            }

            let diff = other_transform.translation.xy() - this_transform.translation.xy();
            if diff.length() < 100. {
                c = c - diff;
            }
        }
        vel.0 += c * 0.1 * time.delta_seconds();
    }
}

fn alignment(
    settings: Res<BoidSettings>,
    mut boids: Query<(&Transform, &mut Velocity), With<Boid>>,
) {
    for (boid, mut vel) in boids.iter_mut() {}
}

fn update(settings: Res<BoidSettings>, mut boids: Query<(&mut Transform, &Velocity), With<Boid>>) {
    for (mut boid, vel) in boids.iter_mut() {
        boid.translation += vel.extend(0.0);
    }
}

fn gizmo(mut gizmos: Gizmos, boids: Query<&Transform, With<Boid>>) {
    for boid in boids.iter() {
        gizmos.circle(boid.translation, Vec3::Z, 10.0, Color::RED);
    }
}
