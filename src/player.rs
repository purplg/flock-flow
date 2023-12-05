use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_spatial::{kdtree::KDTree2, SpatialAccess};
use rand::Rng;

use crate::{
    boid::{Alignment, BoidSettings, Velocity},
    collectible::{self, Collectible},
    input::InputEvent,
    points::PointEvent,
    rng::RngSource,
    track::Tracked,
    GameEvent, Health,
};

#[derive(Component)]
pub struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_systems(Update, movement);
        app.add_systems(Update, collect);
    }
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, mut rng: ResMut<RngSource>) {
    let pos = Vec3::new(
        rng.gen::<f32>() * 1000. - 500.,
        rng.gen::<f32>() * 600. - 300.,
        1.0,
    );
    let mut entity = commands.spawn_empty();
    entity.insert(Name::new("player"));
    entity.insert(SpriteBundle {
        texture: asset_server.load("player.png"),
        ..default()
    });
    entity.insert(Player);
    entity.insert(Tracked);
    entity.insert(Health(100));
    entity.insert(Velocity(-pos.xy().normalize_or_zero()));
    entity.insert(Alignment::default());
    entity.insert(TransformBundle {
        local: Transform::from_translation(pos),
        ..default()
    });
}

fn movement(
    settings: Res<BoidSettings>,
    mut input: EventReader<InputEvent>,
    mut player: Query<(&mut Velocity, &mut Transform), With<Player>>,
    time: Res<Time>,
) {
    let Ok((mut vel, mut transform)) = player.get_single_mut() else {
        return;
    };

    let mut turn = 0.0;
    for event in input.read() {
        match event {
            InputEvent::Turn(angvel) => turn += angvel,
            InputEvent::Schwack(_) | InputEvent::SpawnBoi => {}
        }
    }

    let radians = vel.0.y.atan2(vel.0.x);
    let pos = transform.translation.xy();
    let mut angle = radians + turn * time.delta_seconds() * 5.0;
    if !settings.bounds.contains(pos) {
        let up = -transform.up().xy();
        angle -= pos.angle_between(up) * time.delta_seconds() * 3.;
    }
    vel.0 = Vec2::from_angle(angle) * settings.max_velocity;

    transform.translation += vel.extend(0.0) * time.delta_seconds();
    transform.rotation = Quat::from_axis_angle(Vec3::Z, vel.0.y.atan2(vel.0.x) + PI * 1.5);
    vel.0 = vel.0.lerp(Vec2::ZERO, time.delta_seconds() * 10.0);
}

fn collect(
    quadtree: Res<KDTree2<Tracked>>,
    boids: Query<&Transform, With<Player>>,
    collectibles: Query<(Entity, &Collectible), Without<collectible::Cooldown>>,
    mut point_event: EventWriter<PointEvent>,
    mut collectible_event: EventWriter<collectible::Event>,
    mut game_events: EventWriter<GameEvent>,
) {
    for player in boids.iter() {
        let pos = player.translation.xy();
        for entity in quadtree
            .within_distance(pos, 32.0)
            .into_iter()
            .filter_map(|(_, entity)| entity)
        {
            if let Ok((entity, collectible)) = collectibles.get(entity) {
                point_event.send(PointEvent::Add(collectible.value));
                collectible_event.send(collectible::Event::Collect(entity));
                game_events.send(GameEvent::NextWave);
            }
        }
    }
}
