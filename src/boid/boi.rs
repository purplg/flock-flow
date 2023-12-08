use std::f32::consts::PI;

use super::{BoidBundle, Velocity};

use bevy::prelude::*;
use bevy_spatial::{kdtree::KDTree2, SpatialAccess};
use rand::Rng;

use crate::{
    assets::Images, input::InputEvent, rng::RngSource, shockwave, track::Tracked, GameEvent,
};

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, input);
        app.add_systems(Update, spawn);
    }
}

#[derive(Component)]
struct Boi;

fn input(
    mut input_events: EventReader<InputEvent>,
    mut game_events: EventWriter<GameEvent>,
    mut shock_events: EventWriter<shockwave::Event>,
) {
    for event in input_events.read() {
        match event {
            InputEvent::NextWave => {
                game_events.send(GameEvent::NextWave);
            }
            InputEvent::Boost | InputEvent::Turn(_) => {}
            InputEvent::Schwack(schwack_pos) => shock_events.send(shockwave::Event::Spawn {
                position: *schwack_pos,
                radius: 100.,
                duration: Duration::from_secs(1),
            }),
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
        if let super::SpawnEvent::Boi = event {
            let mut entity = commands.spawn_empty();
            entity.insert(Name::new("Boi"));
            entity.insert(Boi);
            entity.insert(SpriteBundle {
                texture: images.boi.clone(),
                ..default()
            });
            let angle = rng.gen::<f32>() * PI * 2.;
            let pos = Vec2 {
                x: angle.cos(),
                y: angle.sin(),
            } * 1000.;
            entity.insert(BoidBundle::new(
                pos,
                Vec2 {
                    x: rng.gen::<f32>() * 200. - 100.,
                    y: rng.gen::<f32>() * 200. - 100.,
                } * 20.,
            ));
        }
    }
}
