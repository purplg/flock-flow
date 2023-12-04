use bevy::prelude::*;

use crate::GameEvent;

pub struct PointsPlugin;

impl Plugin for PointsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Points(0));
        app.add_event::<PointEvent>();
        app.add_systems(Update, collect.run_if(on_event::<PointEvent>()));
    }
}

#[derive(Debug, Event)]
pub enum PointEvent {
    Add(u32),
    Remove(u32),
}

#[derive(Resource)]
struct Points(pub u32);

fn collect(mut read: EventReader<PointEvent>, mut points: ResMut<Points>) {
    for event in read.read() {
        match event {
            PointEvent::Add(amount) => points.0 += amount,
            PointEvent::Remove(amount) => points.0 -= amount,
        }
        println!("points: {:?}", points.0);
    }
}
