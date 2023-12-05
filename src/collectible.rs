use bevy::prelude::*;
use rand::Rng;

use crate::{rng::RngSource, track::Tracked};

#[derive(Copy, Clone, Debug, Event)]
pub enum Event {
    Spawn,
    Collect,
}

#[derive(Component)]
pub struct Collectible {
    pub value: u32,
}

fn setup(mut writer: EventWriter<Event>) {
    writer.send_batch([Event::Spawn].repeat(1));
}

fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: ResMut<RngSource>,
    mut reader: EventReader<Event>,
) {
    for _ in reader.read().filter(|event| matches!(event, Event::Spawn)) {
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
}

pub struct CollectiblePlugin;

impl Plugin for CollectiblePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Event>();
        app.add_systems(Startup, setup);
        app.add_systems(Update, spawn.run_if(on_event::<Event>()));
    }
}
