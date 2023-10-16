use bevy::{prelude::*, math::Vec3Swizzles};
use bevy_ggrs::ggrs::PlayerHandle;

use crate::{resources::LocalPlayerHandle, components::{Player, Crosshair}};

pub const INPUT_UP: u16 = 1 << 0;
pub const INPUT_RIGHT: u16 = 1 << 1;
pub const INPUT_DOWN: u16 = 1 << 2;
pub const INPUT_LEFT: u16 = 1 << 3;
pub const INPUT_FIRE: u16 = 1 << 4;
pub const INPUT_ANGLE: u16 = 0b1111111100000000;

pub fn input(_: In<PlayerHandle>, keys: Res<Input<KeyCode>>, buttons: Res<Input<MouseButton>>, 
    player_handle: Res<LocalPlayerHandle>,
    players: Query<(&Player, &Transform)>,
    crosshair: Query<&mut Transform, (With<Crosshair>, Without<Player>)>,
) -> u16 {

    let mut input: u16 = 0;

    if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
        input |= INPUT_UP;
    }
    if keys.any_pressed([KeyCode::Down, KeyCode::S]) {
        input |= INPUT_DOWN;
    }
    if keys.any_pressed([KeyCode::Right, KeyCode::D]) {
        input |= INPUT_RIGHT;
    }
    if keys.any_pressed([KeyCode::Left, KeyCode::A]) {
        input |= INPUT_LEFT;
    }
    if keys.any_pressed([KeyCode::Return, KeyCode::K]) || buttons.any_pressed([MouseButton::Left]) {
        input |= INPUT_FIRE;
    }

    for (player, player_transform) in &players {
        if player.handle != player_handle.0 {
            continue;
        }

        let mouse_move_dir = (crosshair.single().translation.xy() - player_transform.translation.xy()).normalize();
        let network_angle = ((mouse_move_dir.y.atan2(mouse_move_dir.x) + std::f32::consts::PI) / (2.0 * std::f32::consts::PI) * u8::MAX as f32) as u8;
        input |= (network_angle as u16) << 8;
    }

    return input;
}