use bevy::prelude::*;
use rand::{rngs::SmallRng, SeedableRng};

pub struct RngPlugin;

impl Plugin for RngPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RngSource>();
    }
}

#[derive(Deref, DerefMut, Resource)]
pub struct RngSource(SmallRng);

impl Default for RngSource {
    fn default() -> Self {
        Self(SmallRng::from_entropy())
    }
}
