use bevy::prelude::*;
use bevy_inspector_egui::{prelude::*, quick::ResourceInspectorPlugin, InspectorOptions};
use bevy_spatial::{kdtree::KDTree2, AutomaticUpdate, SpatialAccess, SpatialStructure};
use itertools::Itertools;
use rand::{distributions::Standard, Rng};

use crate::{input::InputEvent, rng::RngSource, GameEvent, Health};

pub struct BoidPlugin;

impl Plugin for BoidPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AutomaticUpdate::<Boid>::new().with_spatial_ds(SpatialStructure::KDTree2));
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
        app.add_systems(Update, input);
        app.add_systems(Update, spawn);
        app.add_systems(Update, coherence);
        app.add_systems(Update, separation);
        app.add_systems(Update, alignment);
        app.add_systems(Update, hit);
        app.add_systems(Update, cooldown);
        app.add_systems(Update, step);
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

fn startup(mut rng: ResMut<RngSource>, mut events: EventWriter<GameEvent>) {
    events.send_batch(
        (&mut **rng)
            .sample_iter(Standard)
            .take(1000 * 2)
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
    mut boids: Query<(&Transform, &mut Velocity), With<Boid>>,
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
                let x = rng.gen::<f32>() * 200. - 100.;
                let y = rng.gen::<f32>() * 200. - 100.;
                let vel = Vec2 { x, y } * 20.;
                entity.insert(Velocity(vel));
                entity.insert(Coherence::default());
                entity.insert(Separation::default());
                entity.insert(Alignment::default());
                entity.insert(TransformBundle {
                    local: Transform::from_xyz(pos.x, pos.y, 0.0),
                    ..default()
                });
            }
            GameEvent::HurtNode(_, _) => {}
        }
    }
}

#[derive(Component, Default)]
struct Coherence {
    effect: Vec2,
}

fn coherence(
    settings: Res<BoidSettings>,
    quadtree: Res<KDTree2<Boid>>,
    mut boids: Query<(&Transform, &mut Coherence), With<Boid>>,
) {
    for (transform, mut coherence) in boids.iter_mut() {
        let this_pos = transform.translation.xy();

        let nearby = quadtree.within_distance(this_pos, settings.visual_range);
        let count = nearby.len();
        coherence.effect = if count > 1 {
            let masses: Vec2 =
                nearby.into_iter().map(|(pos, _entity)| pos).sum::<Vec2>() - this_pos;
            ((masses / (count - 1) as f32) - this_pos) * settings.coherence
        } else {
            Vec2::ZERO
        }
    }
}

#[derive(Component, Default)]
struct Separation {
    effect: Vec2,
}

fn separation(
    settings: Res<BoidSettings>,
    quadtree: Res<KDTree2<Boid>>,
    mut boids: Query<(Entity, &Transform, &mut Separation), With<Boid>>,
) {
    for (this_entity, transform, mut separation) in boids.iter_mut() {
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

#[derive(Component, Default)]
struct Alignment {
    effect: Vec2,
}

fn alignment(
    settings: Res<BoidSettings>,
    quadtree: Res<KDTree2<Boid>>,
    mut boids: Query<(&Transform, &Velocity, &mut Alignment), With<Boid>>,
    other: Query<&Velocity, With<Boid>>,
) {
    for (transform, vel, mut alignment) in boids.iter_mut() {
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

        if count == 0 {
            alignment.effect = Vec2::ZERO;
            return;
        };

        alignment.effect = (velocities / count as f32) * settings.alignment;
    }
}

fn step(
    settings: Res<BoidSettings>,
    mut boids: Query<
        (
            &mut Transform,
            &mut Velocity,
            &Coherence,
            &Separation,
            &Alignment,
        ),
        With<Boid>,
    >,
    time: Res<Time>,
) {
    for (mut transform, mut vel, coherence, separation, alignment) in boids.iter_mut() {
        vel.0 += coherence.effect;
        vel.0 += separation.effect;
        vel.0 += alignment.effect;

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
        vel.0 = vel.clamp_length_max(settings.max_velocity);
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
