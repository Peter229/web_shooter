use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub handle: usize
}

#[derive(Component, Reflect, Default)]
pub struct BulletReady(pub bool);

#[derive(Component, Reflect, Default)]
pub struct Bullet;

#[derive(Component, Reflect, Default, Clone, Copy)]
pub struct MoveDir(pub Vec2);

#[derive(Component, Reflect, Default)]
pub struct Health(pub i32);

#[derive(Component, Reflect, Default)]
pub struct PlayerTimer(pub f32);

#[derive(Component, Reflect, Default)]
pub struct Crosshair;