use bevy::prelude::*;
use bevy_ggrs::ggrs::PlayerHandle;

pub const INPUT_UP: u8 = 1 << 0;
pub const INPUT_RIGHT: u8 = 1 << 2;
pub const INPUT_DOWN: u8 = 1 << 1;
pub const INPUT_LEFT: u8 = 1 << 3;
pub const INPUT_FIRE: u8 = 1 << 4;

pub fn input(_: In<PlayerHandle>, keys: Res<Input<KeyCode>>) -> u8 {

    let mut input: u8 = 0;

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
    if keys.any_pressed([KeyCode::Return, KeyCode::K]) {
        input |= INPUT_FIRE;
    }

    return input;
}