use bevy::prelude::*;
use bevy_spatial::{kdtree::KDTree2, SpatialAccess};

use crate::track::Tracked;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Event>();
        app.add_systems(Update, invulnerable);
    }
}

#[derive(Debug, Event)]
pub enum Event {
    Die(Entity),
}

#[derive(Component)]
pub struct Health(pub u32);

#[derive(Component)]
pub struct Invulnerable(pub f32);

fn invulnerable(
    mut commands: Commands,
    mut invul: Query<(Entity, &mut Invulnerable)>,
    time: Res<Time>,
) {
    for (entity, mut invul) in &mut invul {
        invul.0 -= time.delta_seconds();
        if invul.0 <= 0.0 {
            commands.entity(entity).remove::<Invulnerable>();
        }
    }
}

pub fn damages<Attacker: Component + Default, Target: Component + Default, const DISTANCE: u32>(
    mut commands: Commands,
    quadtree: Res<KDTree2<Tracked>>,
    attacker: Query<&Transform, With<Attacker>>,
    mut target: Query<&mut Health, (With<Target>, Without<Invulnerable>)>,
    mut events: EventWriter<Event>,
) {
    for trans in attacker.iter() {
        let pos = trans.translation.xy();
        #[allow(clippy::cast_precision_loss)]
        for entity in quadtree
            .within_distance(pos, DISTANCE as f32)
            .into_iter()
            .filter_map(|(_pos, entity)| entity)
        {
            let Ok(mut health) = target.get_mut(entity) else {
                continue;
            };

            health.0 = if health.0 > 1 {
                commands.entity(entity).insert(Invulnerable(1.));
                health.0 - 1
            } else {
                events.send(Event::Die(entity));
                0
            };
        }
    }
}
