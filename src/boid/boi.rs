use super::BoidBundle;
use crate::{assets::Images, rng::RngSource};
use bevy::prelude::*;
use rand::Rng;

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn.run_if(on_event::<super::SpawnEvent>()));
    }
}

#[derive(Component)]
struct Boi;

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
