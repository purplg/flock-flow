use std::{f32::consts::PI, time::Duration};

use bevy::prelude::*;
use bevy_spatial::{kdtree::KDTree2, SpatialAccess};
use interpolation::Lerp;
use rand::Rng;

use crate::{
    assets::Images,
    boid::{Alignment, BoidSettings},
    collectible::{self, Collectible},
    input::InputEvent,
    points::PointEvent,
    rng::RngSource,
    shockwave,
    track::Tracked,
    velocity::Velocity,
    GameEvent,
};

#[derive(Component, Default)]
pub struct Player {
    target_speed: f32,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_systems(Update, movement);
        app.add_systems(Update, collect);
        app.add_systems(Update, boost_cooldown);
        app.add_systems(Update, speed);

        #[cfg(feature = "inspector")]
        app.register_type::<Boost>();
    }
}

fn startup(
    mut commands: Commands,
    settings: Res<BoidSettings>,
    images: Res<Images>,
    mut rng: ResMut<RngSource>,
) {
    let pos = Vec3::new(
        rng.gen::<f32>() * 1000. - 500.,
        rng.gen::<f32>() * 600. - 300.,
        1.0,
    );
    let mut entity = commands.spawn_empty();
    entity.insert(Name::new("player"));
    entity.insert(SpriteBundle {
        texture: images.player.clone(),
        ..default()
    });
    entity.insert(Player {
        target_speed: settings.max_speed,
    });
    entity.insert(Tracked);
    entity.insert(Velocity(-pos.xy().normalize_or_zero()));
    entity.insert(Alignment::default());
    entity.insert(Boost::new(3.0));
    entity.insert(TransformBundle {
        local: Transform::from_translation(pos),
        ..default()
    });
}

#[derive(Component)]
#[cfg_attr(feature = "inspector", derive(Reflect))]
struct Boost {
    cooldown: f32,
    multiplier: f32,
}

impl Boost {
    pub fn new(multiplier: f32) -> Self {
        Self {
            cooldown: 0.0,
            multiplier,
        }
    }
}

fn boost_cooldown(mut boost: Query<&mut Boost>, time: Res<Time>) {
    for mut boost in &mut boost {
        if boost.cooldown > 0.0 {
            boost.cooldown -= time.delta_seconds();
        }
    }
}

fn speed(
    mut player: Query<(&mut Player, &Transform, &mut Boost)>,
    mut input: EventReader<InputEvent>,
    mut shockwave_events: EventWriter<shockwave::Event>,
    settings: Res<BoidSettings>,
    time: Res<Time>,
) {
    for (mut player, transform, mut boost) in &mut player {
        for event in input.read() {
            match event {
                InputEvent::Boost => {
                    if boost.cooldown <= 0.0 {
                        boost.cooldown = 1.0;
                        player.target_speed = settings.max_speed * boost.multiplier;
                        shockwave_events.send(shockwave::Event::Spawn {
                            position: transform.translation.xy(),
                            radius: 100.,
                            duration: Duration::from_secs_f32(0.5),
                        });
                    }
                }
                InputEvent::SlowDown => {
                    player.target_speed = settings.max_speed * 0.5;
                }
                InputEvent::Turn(_) | InputEvent::Schwack(_) | InputEvent::NextWave => {}
            }
        }

        player.target_speed = player
            .target_speed
            .lerp(&settings.max_speed, &time.delta_seconds());
    }
}

fn movement(
    settings: Res<BoidSettings>,
    mut input: EventReader<InputEvent>,
    mut player: Query<(&Player, &mut Velocity, &mut Transform)>,
    time: Res<Time>,
) {
    let Ok((player, mut vel, mut transform)) = player.get_single_mut() else {
        return;
    };

    let mut turn = 0.0;
    for event in input.read() {
        if let InputEvent::Turn(angvel) = event {
            turn += angvel;
        }
    }

    let radians = vel.0.y.atan2(vel.0.x);
    let pos = transform.translation.xy();
    let mut angle = radians + turn * time.delta_seconds() * 5.0;
    if !settings.bounds.contains(pos) {
        let up = -transform.up().xy();
        angle -= pos.angle_between(up) * time.delta_seconds() * 3.;
    }

    vel.0 = Vec2::from_angle(angle) * vel.0.length();
    vel.0 = vel.0.lerp(
        vel.0.normalize_or_zero() * player.target_speed,
        time.delta_seconds() * 10.0,
    );
    transform.translation += vel.extend(0.0) * time.delta_seconds();
    transform.rotation = Quat::from_axis_angle(Vec3::Z, vel.0.y.atan2(vel.0.x) + PI * 1.5);
}

fn collect(
    quadtree: Res<KDTree2<Tracked>>,
    player: Query<(&Transform, &Velocity), With<Player>>,
    collectibles: Query<(Entity, &Collectible), Without<collectible::Cooldown>>,
    mut point_event: EventWriter<PointEvent>,
    mut collectible_event: EventWriter<collectible::Event>,
    mut game_events: EventWriter<GameEvent>,
) {
    for (transform, velocity) in player.iter() {
        let pos = transform.translation.xy();
        for entity in quadtree
            .within_distance(pos, 32.0)
            .into_iter()
            .filter_map(|(_, entity)| entity)
        {
            if let Ok((entity, collectible)) = collectibles.get(entity) {
                point_event.send(PointEvent::Add(collectible.value));
                collectible_event.send(collectible::Event::Collect(entity));
                game_events.send(GameEvent::NextWave {
                    position: transform.translation.xy(),
                    velocity: velocity.0,
                });
            }
        }
    }
}
