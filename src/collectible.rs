use bevy::prelude::*;
use rand::Rng;

use crate::{points::PointEvent, rng::RngSource, track::Tracked, GameEvent};

#[derive(Component)]
pub struct Collectible {
    pub value: u32,
}

pub struct CollectiblePlugin;

impl Plugin for CollectiblePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_systems(Update, collect.run_if(on_event::<GameEvent>()));
    }
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, mut rng: ResMut<RngSource>) {
    let rng = &mut **rng;
    let mut entity = commands.spawn_empty();
    entity.insert(Name::new("Collectible"));
    entity.insert(SpriteBundle {
        texture: asset_server.load("node.png"),
        ..default()
    });
    entity.insert(Tracked);
    entity.insert(Collectible { value: 1 });
    entity.insert(TransformBundle {
        local: Transform::from_xyz(
            rng.gen::<f32>() * 1000. - 500.,
            rng.gen::<f32>() * 600. - 300.,
            0.0,
        ),
        ..default()
    });
}

fn collect(
    mut commands: Commands,
    collectibles: Query<(Entity, &Collectible)>,
    mut reader: EventReader<GameEvent>,
    mut point_event: EventWriter<PointEvent>,
) {
    for (entity, collectible) in collectibles.iter() {
        for event in reader.read() {
            if let GameEvent::Collect(event_entity) = event {
                if entity == *event_entity {
                    commands.entity(entity).despawn();
                    point_event.send(PointEvent::Add(collectible.value));
                    break;
                }
            }
        }
    }
}
