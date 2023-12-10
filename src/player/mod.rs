mod offscreen_marker;

use std::{f32::consts::PI, time::Duration};

use bevy::{audio::PlaybackMode, prelude::*};
use bevy_spatial::{kdtree::KDTree2, SpatialAccess};
use interpolation::{Ease, Lerp};
use rand::{seq::IteratorRandom, Rng};

use crate::{
    assets::{Images, Sounds},
    boid::{Alignment, BoidSettings},
    collectible::{self, Collectible},
    health::{self, Health},
    input::InputEvent,
    points::PointEvent,
    rng::RngSource,
    shockwave,
    track::Tracked,
    velocity::Velocity,
    GameEvent,
};

const MIN_SCALE: Vec2 = Vec2::new(0.2, 0.2);
const MAX_SCALE: Vec2 = Vec2::new(1.0, 1.0);

#[derive(Component, Default)]
pub struct Player {
    target_linvel: f32,
    angvel: f32,
    turn_speed: f32,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_systems(OnExit(crate::GameState::GameOver), startup);
        app.add_systems(Update, movement.run_if(in_state(crate::GameState::Playing)));
        app.add_systems(OnEnter(crate::GameState::Paused), pause);
        app.add_systems(OnExit(crate::GameState::Paused), unpause);
        app.add_systems(Update, collect);
        app.add_systems(
            Update,
            boost_cooldown.run_if(in_state(crate::GameState::Playing)),
        );
        app.add_systems(PreUpdate, input.run_if(on_event::<InputEvent>()));
        app.add_systems(Update, fast_removes_alignment);
        app.add_systems(Update, slow_adds_alignment);
        app.add_systems(Update, die.run_if(on_event::<health::Event>()));
        app.add_systems(
            Update,
            engine_audio.run_if(in_state(crate::GameState::Playing)),
        );
        app.add_systems(OnEnter(crate::GameState::GameOver), gameover);
        app.add_plugins(offscreen_marker::Plugin);

        #[cfg(feature = "inspector")]
        app.register_type::<Boost>();
    }
}

fn startup(
    mut commands: Commands,
    settings: Res<BoidSettings>,
    images: Res<Images>,
    sounds: Res<Sounds>,
    mut rng: ResMut<RngSource>,
) {
    let pos = Vec3::new(
        rng.gen::<f32>() * 1000. - 500.,
        rng.gen::<f32>() * 600. - 300.,
        10.0,
    );
    let mut entity = commands.spawn_empty();
    entity.insert(Name::new("player"));
    entity.insert(SpriteBundle {
        sprite: Sprite {
            color: Color::GREEN,
            ..default()
        },
        texture: images.player.clone(),
        ..default()
    });
    entity.insert(Player {
        target_linvel: settings.max_speed,
        angvel: 0.0,
        turn_speed: 1.5,
    });
    entity.insert(Tracked);
    entity.insert(Health(1));
    entity.insert(Velocity(-pos.xy().normalize_or_zero()));
    entity.insert(Alignment::default());
    entity.insert(Boost::new(4.));
    entity.insert(Brake::new(2000.));
    entity.insert(TransformBundle {
        local: Transform::from_translation(pos),
        ..default()
    });
    entity.insert(AudioBundle {
        source: sounds.player_engine.clone(),
        settings: PlaybackSettings {
            mode: PlaybackMode::Loop,
            ..default()
        },
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

#[derive(Component)]
#[cfg_attr(feature = "inspector", derive(Reflect))]
struct Brake {
    power: f32,
}

impl Brake {
    pub fn new(multiplier: f32) -> Self {
        Self { power: multiplier }
    }
}

fn boost_cooldown(mut boost: Query<&mut Boost>, time: Res<Time>) {
    for mut boost in &mut boost {
        if boost.cooldown > 0.0 {
            boost.cooldown -= time.delta_seconds();
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn input(
    mut commands: Commands,
    mut player: Query<(&mut Player, &Transform, &mut Boost, &Brake)>,
    mut input: EventReader<InputEvent>,
    mut shockwave_events: EventWriter<shockwave::Event>,
    mut rng: ResMut<RngSource>,
    sounds: Res<Sounds>,
    settings: Res<BoidSettings>,
    time: Res<Time>,
) {
    let Ok((mut player, transform, mut boost, brake)) = player.get_single_mut() else {
        return;
    };

    for event in input.read() {
        match event {
            InputEvent::Boost => {
                if boost.cooldown <= 0.0 {
                    boost.cooldown = 1.0;
                    player.target_linvel = settings.max_speed * boost.multiplier;
                    shockwave_events.send(shockwave::Event::Spawn {
                        position: transform.translation.xy(),
                        radius: 100.,
                        duration: Duration::from_secs_f32(0.5),
                        color: Color::YELLOW,
                        repel: true,
                    });
                    commands.spawn(AudioBundle {
                        source: sounds.boost.iter().choose(&mut **rng).unwrap().clone(),
                        settings: PlaybackSettings {
                            mode: PlaybackMode::Remove,
                            ..default()
                        },
                    });
                }
            }
            InputEvent::Brake => {
                player.target_linvel -= time.delta_seconds() * brake.power;
            }
            InputEvent::Turn(dir) => {
                player.angvel += dir * player.turn_speed * 2.;
                player.angvel = player.angvel.clamp(-player.turn_speed, player.turn_speed);
            }
            InputEvent::Pause => {}
        }
    }
}

fn movement(
    settings: Res<BoidSettings>,
    mut player: Query<(&mut Player, &mut Velocity, &mut Transform, &Boost)>,
    time: Res<Time>,
) {
    let Ok((mut player, mut vel, mut transform, boost)) = player.get_single_mut() else {
        return;
    };

    // Bounds
    let radians = vel.0.y.atan2(vel.0.x);
    let pos = transform.translation.xy();
    let mut angle = radians + player.angvel * time.delta_seconds() * 5.0;
    if !settings.bounds.contains(pos) {
        let up = -transform.up().xy();
        angle -= pos.angle_between(up) * time.delta_seconds() * 3.;
    }

    // Translation
    vel.0 = Vec2::from_angle(angle) * vel.0.length();
    let target_speed = vel.0.normalize_or_zero() * player.target_linvel;
    vel.0 = vel.0.lerp(target_speed, time.delta_seconds() * 10.0);

    // Rotation
    transform.rotation = Quat::from_axis_angle(Vec3::Z, vel.0.y.atan2(vel.0.x) + PI * 1.5);

    // Scale
    let ratio = vel.0.length() / (settings.max_speed * boost.multiplier - 0.5);
    transform.scale = (MIN_SCALE)
        .lerp(MAX_SCALE, ratio.quadratic_out())
        .extend(0.0);

    // Friction
    player.target_linvel = player
        .target_linvel
        .lerp(&settings.max_speed, &(time.delta_seconds() * 0.5));
    player.angvel = 0.0;

    // Clamp
    player.target_linvel = player.target_linvel.clamp(
        settings.max_speed * 0.5,
        settings.max_speed * boost.multiplier,
    );
}

#[allow(clippy::type_complexity)]
fn fast_removes_alignment(
    mut commands: Commands,
    settings: Res<BoidSettings>,
    player: Query<(Entity, &Velocity), (With<Player>, With<Alignment>)>,
) {
    let Ok((entity, vel)) = player.get_single() else {
        return;
    };

    if vel.length_squared() > (settings.max_speed * settings.max_speed) * 4.0 {
        commands.entity(entity).remove::<Alignment>();
    }
}

#[allow(clippy::type_complexity)]
fn slow_adds_alignment(
    mut commands: Commands,
    settings: Res<BoidSettings>,
    player: Query<(Entity, &Velocity), (With<Player>, Without<Alignment>)>,
) {
    let Ok((entity, vel)) = player.get_single() else {
        return;
    };

    if vel.length_squared() < (settings.max_speed * settings.max_speed) * 4.0 {
        commands.entity(entity).insert(Alignment::default());
    }
}

fn collect(
    quadtree: Res<KDTree2<Tracked>>,
    player: Query<(&Transform, &Velocity), With<Player>>,
    collectibles: Query<(Entity, &Collectible), Without<collectible::Cooldown>>,
    mut point_event: EventWriter<PointEvent>,
    mut collectible_event: EventWriter<collectible::Event>,
    mut game_events: EventWriter<GameEvent>,
    mut shockwave_events: EventWriter<shockwave::Event>,
) {
    for (transform, velocity) in player.iter() {
        let pos = transform.translation.xy();
        for (collectible_position, entity) in quadtree
            .within_distance(pos, 32.0)
            .into_iter()
            .filter_map(|(pos, entity)| entity.map(|entity| (pos, entity)))
        {
            if let Ok((entity, collectible)) = collectibles.get(entity) {
                point_event.send(PointEvent::Add(collectible.value));
                collectible_event.send(collectible::Event::Collect(entity));

                game_events.send(GameEvent::NextWave {
                    position: transform.translation.xy(),
                    velocity: velocity.0,
                });

                shockwave_events.send(shockwave::Event::Spawn {
                    position: collectible_position,
                    radius: 100.,
                    duration: Duration::from_secs_f32(1.),
                    color: Color::GREEN,
                    repel: false,
                });
            }
        }
    }
}

fn die(
    player: Query<Entity, With<Player>>,
    mut events: EventReader<health::Event>,
    mut gamestate: ResMut<NextState<crate::GameState>>,
) {
    for event in events.read() {
        let health::Event::Die(entity) = event;

        let Ok(player) = player.get(*entity) else {
            continue;
        };

        if &player == entity {
            gamestate.set(crate::GameState::GameOver);
        }
    }
}

fn gameover(
    mut commands: Commands,
    sounds: Res<Sounds>,
    player: Query<(Entity, &Transform), With<Player>>,
    mut shockwave_events: EventWriter<shockwave::Event>,
    mut rng: ResMut<RngSource>,
) {
    let Ok((entity, transform)) = player.get_single() else {
        return;
    };

    commands.entity(entity).despawn();

    shockwave_events.send(shockwave::Event::Spawn {
        position: transform.translation.xy(),
        radius: 1000.,
        duration: Duration::from_secs_f32(1.0),
        color: Color::RED,
        repel: true,
    });
    commands.spawn(AudioBundle {
        source: sounds.gameover.iter().choose(&mut **rng).unwrap().clone(),
        settings: PlaybackSettings {
            mode: PlaybackMode::Remove,
            ..default()
        },
    });
}

fn engine_audio(
    settings: Res<BoidSettings>,
    player: Query<(&AudioSink, &Velocity, &Boost), With<Player>>,
) {
    let Ok((playback, vel, boost)) = player.get_single() else {
        return;
    };
    playback.set_speed(vel.0.length() / (settings.max_speed * boost.multiplier - 0.5) * 2.);
}

fn pause(player: Query<&AudioSink, With<Player>>) {
    let Ok(playback) = player.get_single() else {
        return;
    };

    playback.pause();
}

fn unpause(player: Query<&AudioSink, With<Player>>) {
    let Ok(playback) = player.get_single() else {
        return;
    };

    playback.play();
}
