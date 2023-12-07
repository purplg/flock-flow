use bevy::prelude::*;

#[derive(Resource)]
pub struct Images {
    pub player: Handle<Image>,
    pub collectible: Handle<Image>,
    pub boi: Handle<Image>,
    pub calmboi: Handle<Image>,
    pub angryboi: Handle<Image>,
}

fn load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Images {
        player: asset_server.load("player.png"),
        collectible: asset_server.load("collectible.png"),
        boi: asset_server.load("boi.png"),
        calmboi: asset_server.load("calmboi.png"),
        angryboi: asset_server.load("angryboi.png"),
    });
}

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load);
    }
}
