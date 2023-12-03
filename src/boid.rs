use bevy::prelude::*;
use bevy_inspector_egui::{prelude::*, quick::ResourceInspectorPlugin, InspectorOptions};
use itertools::Itertools;
use rand::{distributions::Standard, Rng};

use crate::{input::InputEvent, rng::RngSource, GameEvent, Health};

pub struct BoidPlugin;

impl Plugin for BoidPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<BoidSettings>();
        app.register_type::<Velocity>();
        app.insert_resource(BoidSettings {
            coherence: 0.005,
            separation: 0.05,
            alignment: 0.005,
            visual_range: 50.0,
            avoid_range: 50.0 * 0.3,
            max_velocity: 200.0,
        });
        app.insert_resource(BoidDebugSettings {
            show_cluster_range: false,
            show_avoid_range: false,
            show_direction: true,
        });
        app.add_plugins(ResourceInspectorPlugin::<BoidSettings>::default());
        app.add_plugins(ResourceInspectorPlugin::<BoidDebugSettings>::default());
        app.add_systems(Startup, startup);
        app.add_systems(PreUpdate, nearby);
        app.add_systems(Update, input);
        app.add_systems(Update, spawn);
        app.add_systems(Update, coherence);
        app.add_systems(Update, separation);
        app.add_systems(Update, alignment);
        app.add_systems(Update, hit);
        app.add_systems(Update, cooldown);
        app.add_systems(PostUpdate, (bounds, step).chain());
        app.add_systems(Update, gizmo);
    }
}

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource)]
struct BoidDebugSettings {
    show_cluster_range: bool,
    show_avoid_range: bool,
    show_direction: bool,
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
    avoid_range: f32,
    #[inspector(min = 0.0)]
    max_velocity: f32,
}

#[derive(Component)]
struct Boid;

#[derive(Component, Default, Deref, DerefMut, Reflect)]
struct Velocity(pub Vec2);

#[derive(Component, Default, Deref, DerefMut)]
struct NextVelocity(pub Vec2);

#[derive(Component, Default, Deref, DerefMut)]
struct Nearby(pub Vec<Entity>);

fn startup(mut rng: ResMut<RngSource>, mut events: EventWriter<GameEvent>) {
    events.send_batch(
        (&mut **rng)
            .sample_iter(Standard)
            .take(100 * 2)
            .tuples()
            .map(|(x, y): (f32, f32)| {
                GameEvent::SpawnBoid(Vec2 {
                    x: x * 1000. - 500.,
                    y: y * 600. - 300.,
                })
            }),
    )
}

fn input(
    mut rng: ResMut<RngSource>,
    mut input_events: EventReader<InputEvent>,
    mut game_events: EventWriter<GameEvent>,
    mut boids: Query<(&Transform, &mut NextVelocity), With<Boid>>,
) {
    for event in input_events.read() {
        match event {
            InputEvent::SpawnBoid => {
                let rng = &mut **rng;
                game_events.send(GameEvent::SpawnBoid(Vec2 {
                    x: rng.gen::<f32>() * 1000. - 500.,
                    y: rng.gen::<f32>() * 600. - 300.,
                }));
            }
            InputEvent::Schwack(schwak_pos) => {
                for (pos, mut vel) in boids
                    .iter_mut()
                    .map(|(trans, vel)| (trans.translation.xy(), vel))
                    .filter(|(pos, _)| (*pos - *schwak_pos).length_squared() < 100. * 100.)
                {
                    vel.0 += (pos - *schwak_pos) * 10.;
                }
            }
        }
    }
}

fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: ResMut<RngSource>,
    mut events: EventReader<GameEvent>,
) {
    for event in events.read() {
        match event {
            GameEvent::SpawnBoid(pos) => {
                let rng = &mut **rng;
                let mut entity = commands.spawn_empty();
                entity.insert(Name::new("Boid"));
                entity.insert(SpriteBundle {
                    texture: asset_server.load("boid.png"),
                    ..default()
                });
                entity.insert(Boid);
                entity.insert(Nearby::default());
                let x = rng.gen::<f32>() * 200. - 100.;
                let y = rng.gen::<f32>() * 200. - 100.;
                let vel = Vec2 { x, y } * 20.;
                entity.insert(Velocity(vel));
                entity.insert(NextVelocity(vel));
                entity.insert(TransformBundle {
                    local: Transform::from_xyz(pos.x, pos.y, 0.0),
                    ..default()
                });
            }
            GameEvent::HurtNode(_, _) => {}
        }
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
    settings: Res<BoidSettings>,
    mut boids: Query<(&Transform, &mut NextVelocity, &Nearby), With<Boid>>,
    other: Query<(&Transform, &Velocity), With<Boid>>,
) {
    for (transform, mut vel, nearby) in boids.iter_mut() {
        let count = nearby.len();
        if count == 0 {
            continue;
        }

        let this_pos = transform.translation.xy();
        let center_of_mass: Vec2 = nearby
            .iter()
            .filter_map(|other_entity| other.get(*other_entity).ok())
            .map(|(other_trans, _)| other_trans.translation.xy())
            .sum::<Vec2>()
            / count as f32;
        vel.0 += (center_of_mass - this_pos) * settings.coherence;
    }
}

fn separation(
    settings: Res<BoidSettings>,
    mut boids: Query<(&Transform, &mut NextVelocity, &Nearby), With<Boid>>,
    other: Query<(&Transform, &Velocity), With<Boid>>,
) {
    for (transform, mut vel, nearby) in boids.iter_mut() {
        let count = nearby.len();
        if count == 0 {
            continue;
        }

        let this_pos = transform.translation.xy();
        let mut c = Vec2::ZERO;
        for other_pos in nearby
            .iter()
            .filter_map(|entity| other.get(*entity).ok())
            .map(|(other_trans, _)| other_trans.translation.xy())
            .filter(|other_pos| {
                (*other_pos - this_pos).length_squared()
                    < settings.avoid_range * settings.avoid_range
            })
        {
            c += this_pos - other_pos;
        }
        vel.0 += c * settings.separation;
    }
}

fn alignment(
    settings: Res<BoidSettings>,
    mut boids: Query<(&Transform, &mut NextVelocity, &Nearby), With<Boid>>,
    other: Query<(&Transform, &Velocity), With<Boid>>,
) {
    for (transform, mut vel, nearby) in boids.iter_mut() {
        let count = nearby.len();
        if count == 0 {
            continue;
        }

        let this_pos = transform.translation.xy();
        let average_vel: Vec2 = nearby
            .iter()
            .filter_map(|other_entity| other.get(*other_entity).ok())
            .map(|(_, other_vel)| other_vel.0)
            .sum::<Vec2>()
            / count as f32;

        vel.0 += (average_vel - this_pos) * settings.alignment;
    }
}

fn bounds(
    settings: Res<BoidSettings>,
    mut boids: Query<(&Transform, &mut NextVelocity), With<Boid>>,
) {
    for (transform, mut vel) in boids.iter_mut() {
        const CENTER_FORCE: f32 = 100.0;
        let pos = transform.translation.xy();
        if pos.x < -500.0 {
            vel.0.x += CENTER_FORCE;
        } else if pos.x > 500.0 {
            vel.0.x -= CENTER_FORCE;
        }

        if pos.y < -300.0 {
            vel.0.y += CENTER_FORCE;
        } else if pos.y > 300.0 {
            vel.0.y -= CENTER_FORCE;
        }
        vel.0 = vel.0.clamp_length_max(settings.max_velocity);
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

#[derive(Component)]
struct HitCooldown(f32);

fn cooldown(
    mut commands: Commands,
    mut cooldown: Query<(Entity, &mut HitCooldown)>,
    time: Res<Time>,
) {
    for (entity, mut cooldown) in cooldown.iter_mut() {
        cooldown.0 -= time.delta_seconds();
        if cooldown.0 < 0.0 {
            commands.entity(entity).remove::<HitCooldown>();
        }
    }
}

fn hit(
    mut commands: Commands,
    boids: Query<(Entity, &Transform), (With<Boid>, Without<HitCooldown>)>,
    healths: Query<(Entity, &Transform), With<Health>>,
    mut events: EventWriter<GameEvent>,
) {
    for (boid_entity, boid) in boids.iter() {
        for (health_entity, node) in healths.iter() {
            if boid.translation.distance_squared(node.translation) < 10.0 * 10.0 {
                events.send(GameEvent::HurtNode(health_entity, 10));
                commands.entity(boid_entity).insert(HitCooldown(5.));
            }
        }
    }
}

fn gizmo(
    mut gizmos: Gizmos,
    settings: Res<BoidSettings>,
    debug_settings: Res<BoidDebugSettings>,
    boids: Query<(&Transform, &Velocity), With<Boid>>,
) {
    for (transform, velocity) in boids.iter() {
        if debug_settings.show_cluster_range {
            gizmos.circle_2d(
                transform.translation.xy(),
                settings.visual_range,
                Color::rgba(1.0, 0.0, 0.0, 0.1),
            );
        }
        if debug_settings.show_avoid_range {
            gizmos.circle_2d(
                transform.translation.xy(),
                settings.avoid_range,
                Color::rgba(1.0, 1.0, 0.0, 0.1),
            );
        }
        if debug_settings.show_direction {
            gizmos.line_2d(
                transform.translation.xy(),
                transform.translation.xy() + velocity.0.normalize_or_zero() * 10.0,
                Color::RED,
            );
        }
    }
}
