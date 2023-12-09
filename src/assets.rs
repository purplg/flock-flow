use bevy::prelude::*;

#[derive(Resource)]
pub struct Images {
    pub background: Handle<Image>,
    pub player: Handle<Image>,
    pub collectible: Handle<Image>,
    pub boi: Handle<Image>,
    pub calmboi: Handle<Image>,
    pub angryboi: Handle<Image>,
    pub smoke: Handle<Image>,
}

fn load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Images {
        background: asset_server.load("background.png"),
        player: asset_server.load("player.png"),
        collectible: asset_server.load("collectible.png"),
        boi: asset_server.load("boi.png"),
        calmboi: asset_server.load("calmboi.png"),
        angryboi: asset_server.load("angryboi.png"),
        smoke: asset_server.load("smoke.png"),
    });
}

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load);
    }
}
