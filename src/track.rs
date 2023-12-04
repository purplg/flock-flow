use bevy::prelude::*;
use bevy_spatial::{AutomaticUpdate, SpatialStructure};

pub struct TrackPlugin;

impl Plugin for TrackPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            AutomaticUpdate::<Tracked>::new().with_spatial_ds(SpatialStructure::KDTree2),
        );
    }
}

#[derive(Component)]
pub struct Tracked;
