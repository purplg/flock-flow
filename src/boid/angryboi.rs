use bevy::prelude::*;

use crate::{assets::Images, health::damages};

use crate::player::Player;

use super::{BoidBundle, Home};

#[derive(Component, Default)]
struct AngryBoi;

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn.run_if(on_event::<super::SpawnEvent>()));
        app.add_systems(Update, super::home::<Player>);
        app.add_systems(Update, damages::<AngryBoi, Player, 16>);
    }
}

fn spawn(mut commands: Commands, images: Res<Images>, mut events: EventReader<super::SpawnEvent>) {
    for event in events.read() {
        if let super::BoidKind::AngryBoi = event.kind {
            for _ in 0..event.count {
                let mut entity = commands.spawn_empty();
                entity.insert(Name::new("AngryBoi"));
                entity.insert(AngryBoi);
                let home: Home<Player> = Home::new(10.0);
                entity.insert(home);
                entity.insert(SpriteBundle {
                    texture: images.angryboi.clone(),
                    ..default()
                });
                entity.insert(BoidBundle::new(event.position.extend(2.0), event.velocity));
            }
        }
    }
}
