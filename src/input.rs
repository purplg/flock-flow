use bevy::{prelude::*, window::PrimaryWindow};

pub struct InputPlugin;

#[derive(Debug, Event)]
pub enum InputEvent {
    Schwack(Vec2),
}

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InputEvent>();
        app.add_systems(Update, mouse_button);
    }
}

fn mouse_button(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut input_event: EventWriter<InputEvent>,
    buttons: Res<Input<MouseButton>>,
) {
    let Ok((camera, camera_trans)) = camera.get_single() else {
        return;
    };

    let Some(click_position) = windows
        .get_single()
        .ok()
        .and_then(|window| window.cursor_position())
        .and_then(|cursor_position| camera.viewport_to_world_2d(camera_trans, cursor_position))
    else {
        return;
    };

    for button in buttons.get_pressed() {
        if MouseButton::Left == *button {
            input_event.send(InputEvent::Schwack(click_position));
        }
    }
}
