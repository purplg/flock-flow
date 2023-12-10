use super::BoidBundle;
use crate::{assets::Images, input::InputEvent, rng::RngSource, shockwave, GameEvent};
use bevy::prelude::*;
use rand::Rng;
use std::{f32::consts::PI, time::Duration};

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, input.run_if(on_event::<InputEvent>()));
        app.add_systems(Update, spawn.run_if(on_event::<super::SpawnEvent>()));
    }
}

#[derive(Component)]
struct Boi;

fn input(
    mut input_events: EventReader<InputEvent>,
    mut game_events: EventWriter<GameEvent>,
    mut shock_events: EventWriter<shockwave::Event>,
    mut rng: ResMut<RngSource>,
) {
    for event in input_events.read() {
        match event {
            InputEvent::NextWave => {
                let angle = rng.gen::<f32>() * PI * 2.;
                let position = Vec2 {
                    x: angle.cos(),
                    y: angle.sin(),
                } * 1000.;
                game_events.send(GameEvent::NextWave {
                    position,
                    velocity: -position,
                });
            }
            InputEvent::Schwack(schwack_pos) => shock_events.send(shockwave::Event::Spawn {
                position: *schwack_pos,
                radius: 100.,
                duration: Duration::from_secs(1),
            }),
            InputEvent::SlowDown | InputEvent::Boost | InputEvent::Turn(_) => {}
        }
    }
}

fn spawn(
    mut commands: Commands,
    images: Res<Images>,
    mut rng: ResMut<RngSource>,
    mut events: EventReader<super::SpawnEvent>,
) {
    for event in events.read() {
        if let super::BoidKind::Boi = event.kind {
            for _ in 0..event.count {
                let mut entity = commands.spawn_empty();
                entity.insert(Name::new("Boi"));
                entity.insert(Boi);
                entity.insert(SpriteBundle {
                    texture: images.boi.clone(),
                    ..default()
                });
                let offset = Vec2 {
                    x: 16. * rng.gen::<f32>() - 8.,
                    y: 16. * rng.gen::<f32>() - 8.,
                };
                entity.insert(BoidBundle::new(
                    (event.position + offset).extend(0.0),
                    event.velocity,
                ));
            }
        }
    }
}
