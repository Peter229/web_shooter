use bevy::prelude::*;
use bevy_ggrs::ggrs::PlayerHandle;

pub const INPUT_UP: u8 = 1 << 0;
pub const INPUT_RIGHT: u8 = 1 << 2;
pub const INPUT_DOWN: u8 = 1 << 1;
pub const INPUT_LEFT: u8 = 1 << 3;

pub fn input(_: In<PlayerHandle>, keys: Res<Input<KeyCode>>) -> u8 {

    let mut input: u8 = 0;

    if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
        input |= INPUT_UP;
    }
    if keys.any_pressed([KeyCode::Up, KeyCode::S]) {
        input |= INPUT_DOWN;
    }
    if keys.any_pressed([KeyCode::Up, KeyCode::D]) {
        input |= INPUT_RIGHT;
    }
    if keys.any_pressed([KeyCode::Up, KeyCode::A]) {
        input |= INPUT_LEFT;
    }

    return input;
}