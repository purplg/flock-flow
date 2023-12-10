use bevy::prelude::*;

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            update.run_if(not(in_state(crate::GameState::Paused))),
        );
        #[cfg(feature = "inspector")]
        app.register_type::<Velocity>();
    }
}

#[derive(Component, Default, Deref, DerefMut)]
#[cfg_attr(feature = "inspector", derive(Reflect))]
pub struct Velocity(pub Vec2);

fn update(mut boids: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, vel) in &mut boids {
        transform.translation += vel.extend(0.0) * time.delta_seconds();
    }
}
