use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource)]
pub struct Scores(pub u32,pub u32);

#[derive(Resource, Reflect, Default, Clone, Copy, Eq, PartialEq, Debug)]
#[reflect(Resource)]
pub enum RollbackState {
    #[default]
    Playing,
    Respawn,
}

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "bullet.png")]
    pub bullet: Handle<Image>,
    #[asset(path = "player1.png")]
    pub player1: Handle<Image>,
    #[asset(path = "player2.png")]
    pub player2: Handle<Image>,
    #[asset(path = "remains.png")]
    pub remains: Handle<Image>,
}

#[derive(Resource)]
pub struct LocalPlayerHandle(pub usize);
