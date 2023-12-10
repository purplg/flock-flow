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

#[derive(Resource)]
pub struct Sounds {
    pub boost: Vec<Handle<AudioSource>>,
    pub gameover: Vec<Handle<AudioSource>>,
    pub collect: Vec<Handle<AudioSource>>,
    pub player_engine: Handle<AudioSource>,
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

    commands.insert_resource(Sounds {
        boost: vec![
            asset_server.load("boost_000.ogg"),
            asset_server.load("boost_001.ogg"),
            asset_server.load("boost_002.ogg"),
            asset_server.load("boost_003.ogg"),
            asset_server.load("boost_004.ogg"),
        ],
        gameover: vec![
            asset_server.load("gameover_000.ogg"),
            asset_server.load("gameover_001.ogg"),
            asset_server.load("gameover_002.ogg"),
            asset_server.load("gameover_003.ogg"),
            asset_server.load("gameover_004.ogg"),
        ],
        collect: vec![
            asset_server.load("collect_000.ogg"),
            asset_server.load("collect_001.ogg"),
            asset_server.load("collect_002.ogg"),
            asset_server.load("collect_003.ogg"),
            asset_server.load("collect_004.ogg"),
        ],
        player_engine: asset_server.load("player_engine.ogg"),
    });
}

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load);
    }
}
