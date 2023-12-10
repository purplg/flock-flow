use bevy::prelude::*;

pub struct InputPlugin;

#[derive(Debug, Event)]
pub enum InputEvent {
    Brake,
    Turn(f32),
    Boost,
}

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InputEvent>();
        app.add_systems(Update, keyboard);
    }
}

fn keyboard(keys: Res<Input<KeyCode>>, mut event_writer: EventWriter<InputEvent>) {
    if keys.pressed(KeyCode::S) {
        event_writer.send(InputEvent::Brake);
    }

    if keys.pressed(KeyCode::D) {
        event_writer.send(InputEvent::Turn(-1.0));
    } else if keys.pressed(KeyCode::A) {
        event_writer.send(InputEvent::Turn(1.0));
    }

    if keys.just_pressed(KeyCode::ShiftLeft) {
        event_writer.send(InputEvent::Boost);
    }
}
