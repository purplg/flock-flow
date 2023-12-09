use bevy::prelude::*;

use crate::boid::BoidSettings;

use super::Player;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum PlayerBoundsState {
    #[default]
    In,
    Out,
}

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_state::<PlayerBoundsState>();
        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            out_of_bounds.run_if(in_state(PlayerBoundsState::Out)),
        );
        app.add_systems(Update, in_bounds.run_if(in_state(PlayerBoundsState::In)));
        app.add_systems(OnExit(PlayerBoundsState::Out), disable);
        app.add_systems(OnEnter(PlayerBoundsState::Out), enable);
        app.add_systems(PostUpdate, gizmo);
    }
}

#[derive(Component)]
pub(super) struct OffscreenMarker;

#[derive(Component)]
struct Disabled;

fn setup(mut commands: Commands) {
    let mut entity = commands.spawn_empty();
    entity.insert(OffscreenMarker);
    entity.insert(Disabled);
    entity.insert(TransformBundle::default());
}

fn out_of_bounds(
    settings: Res<BoidSettings>,
    mut marker: Query<&mut Transform, (With<OffscreenMarker>, Without<Player>, Without<Disabled>)>,
    player: Query<&Transform, With<Player>>,
    mut state: ResMut<NextState<PlayerBoundsState>>,
) {
    let Ok(player) = player.get_single() else {
        return;
    };

    for mut marker in &mut marker {
        marker.translation = player
            .translation
            .xy()
            .clamp(settings.bounds.min, settings.bounds.max)
            .extend(0.0);
    }

    if settings.bounds.contains(player.translation.xy()) {
        state.set(PlayerBoundsState::In);
    }
}

fn in_bounds(
    settings: Res<BoidSettings>,
    player: Query<&Transform, With<Player>>,
    mut state: ResMut<NextState<PlayerBoundsState>>,
) {
    let Ok(player) = player.get_single() else {
        return;
    };

    if !settings.bounds.contains(player.translation.xy()) {
        state.set(PlayerBoundsState::Out);
    }
}

fn enable(
    mut commands: Commands,
    settings: Res<BoidSettings>,
    player: Query<&Transform, With<Player>>,
    marker: Query<Entity, (With<OffscreenMarker>, With<Disabled>)>,
) {
    let Ok(player) = player.get_single() else {
        return;
    };

    if settings.bounds.contains(player.translation.xy()) {
        return;
    }

    for entity in &marker {
        commands.entity(entity).remove::<Disabled>();
    }
}

fn disable(
    mut commands: Commands,
    settings: Res<BoidSettings>,
    player: Query<&Transform, With<Player>>,
    marker: Query<Entity, (With<OffscreenMarker>, Without<Disabled>)>,
) {
    let Ok(player) = player.get_single() else {
        return;
    };

    if !settings.bounds.contains(player.translation.xy()) {
        return;
    }

    for entity in &marker {
        commands.entity(entity).insert(Disabled);
    }
}

fn gizmo(
    mut gizmos: Gizmos,
    player: Query<&Transform, With<Player>>,
    marker: Query<&Transform, (With<OffscreenMarker>, Without<Disabled>)>,
) {
    let Ok(player) = player.get_single() else {
        return;
    };

    for marker in &marker {
        let dist = player.translation.xy() - marker.translation.xy();
        let max = dist.normalize_or_zero() * 20.;
        gizmos.line_2d(
            marker.translation.xy(),
            marker.translation.xy()
                + (if dist.length_squared() < max.length_squared() {
                    dist
                } else {
                    max
                }),
            Color::GREEN,
        );
    }
}
