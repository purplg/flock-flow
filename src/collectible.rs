use bevy::prelude::*;
use rand::Rng;

use crate::{rng::RngSource, track::Tracked};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Event>();
        app.add_systems(Startup, setup);
        app.add_systems(PostUpdate, events);
        app.add_systems(Update, cooldown);
    }
}

#[derive(Copy, Clone, Debug, Event)]
pub enum Event {
    Spawn,
    Collect(Entity),
}

#[derive(Component)]
pub struct Collectible {
    pub value: u32,
}

fn setup(mut writer: EventWriter<Event>) {
    writer.send_batch([Event::Spawn].repeat(1));
}

fn events(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: ResMut<RngSource>,
    mut reader: EventReader<Event>,
    mut collectibles: Query<&mut Transform, With<Collectible>>,
) {
    for event in reader.read() {
        match event {
            Event::Spawn => {
                let pos = Vec2 {
                    x: rng.gen::<f32>() * 1000. - 500.,
                    y: rng.gen::<f32>() * 600. - 300.,
                };
                let mut entity = commands.spawn_empty();
                entity.insert(Name::new("Collectible"));
                entity.insert(SpriteBundle {
                    texture: asset_server.load("collectible.png"),
                    ..default()
                });
                entity.insert(Tracked);
                entity.insert(Collectible { value: 1 });
                entity.insert(TransformBundle {
                    local: Transform::from_translation(pos.extend(0.0)),
                    ..default()
                });
            }
            Event::Collect(entity) => {
                if let Ok(mut trans) = collectibles.get_mut(*entity) {
                    commands.entity(*entity).insert(Cooldown(10.0));
                    let pos = Vec2 {
                        x: rng.gen::<f32>() * 1000. - 500.,
                        y: rng.gen::<f32>() * 600. - 300.,
                    };
                    trans.translation = pos.extend(0.0);
                }
            }
        }
    }
}

#[derive(Component)]
pub struct Cooldown(f32);

fn cooldown(mut commands: Commands, mut cooldown: Query<(Entity, &mut Cooldown)>, time: Res<Time>) {
    for (entity, mut cooldown) in &mut cooldown {
        cooldown.0 -= time.delta_seconds();
        if cooldown.0 < 0.0 {
            commands.entity(entity).remove::<Cooldown>();
        }
    }
}
