use bevy::{prelude::*, window::PrimaryWindow};

use crate::camera::PlayerCamera;

pub struct InputPlugin;

#[derive(Debug, Event)]
pub enum InputEvent {
    Turn(f32),
    Schwack(Vec2),
    NextWave,
    Boost,
}

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InputEvent>();
        app.add_systems(Update, mouse_button);
        app.add_systems(Update, keyboard);
    }
}

fn mouse_button(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<PlayerCamera>>,
    mut input_event: EventWriter<InputEvent>,
    buttons: Res<Input<MouseButton>>,
) {
    let Ok((camera, camera_trans)) = camera.get_single() else {
        return;
    };

    let Some(click_position) = windows
        .get_single()
        .ok()
        .and_then(bevy::prelude::Window::cursor_position)
        .and_then(|cursor_position| camera.viewport_to_world_2d(camera_trans, cursor_position))
    else {
        return;
    };

    for button in buttons.get_just_pressed() {
        if MouseButton::Left == *button {
            input_event.send(InputEvent::Schwack(click_position));
        }
    }
}

fn keyboard(keys: Res<Input<KeyCode>>, mut event_writer: EventWriter<InputEvent>) {
    if keys.just_pressed(KeyCode::Space) {
        event_writer.send(InputEvent::NextWave);
    }
    // if keys.pressed(KeyCode::W) {
    //     event_writer.send(InputEvent::Accelerate(1.0));
    // } else if keys.pressed(KeyCode::S) {
    //     event_writer.send(InputEvent::Accelerate(-1.0));
    // }

    if keys.pressed(KeyCode::D) {
        event_writer.send(InputEvent::Turn(-1.0));
    } else if keys.pressed(KeyCode::A) {
        event_writer.send(InputEvent::Turn(1.0));
    }

    if keys.just_pressed(KeyCode::ShiftLeft) {
        event_writer.send(InputEvent::Boost);
    }
}
