use bevy::{prelude::*, math::Vec3Swizzles};
use bevy_ggrs::*;
use bevy_matchbox::matchbox_socket::PeerId;

use crate::input::*;
use crate::components::*;
use crate::resources::*;

pub struct GgrsConfig;

impl ggrs::Config for GgrsConfig {
    type Input = u16;
    type State = u8;
    type Address = PeerId;
}

const MOVE_SPEED: f32 = 0.13;
const PLAYER_RADIUS: f32 = 0.5;
const BULLET_RADIUS: f32 = 0.025;
pub const MAP_SIZE: u32 = 41;
pub const FIRE_RATE: f32 = 0.1;
pub const DELTA_TIME: f32 = 0.016;

//Rollbackable functions, needs to be determistic, all functions will run downwards in that order
pub fn respawn_players(mut players: Query<(&mut Transform, &mut Health, &mut PlayerTimer, &mut Handle<Image>, &Player)>, images: Res<ImageAssets>) {

    for (mut transform, mut health, mut player_timer, mut sprite, player) in &mut players {

        if health.0 <= 0 {

            player_timer.0 -= DELTA_TIME;
            if player_timer.0 <= 0.0 {
                player_timer.0 = 1.0;
                health.0 = 100;
                transform.translation.x = 0.0;
                transform.translation.y = 0.0;
                if player.handle == 0 {
                    *sprite = images.player1.clone();
                }
                else {
                    *sprite = images.player2.clone();
                }
            }
        }
    }
}

pub fn move_player(inputs: Res<PlayerInputs<GgrsConfig>>, mut players: Query<(&mut Transform, &mut MoveDir, &Health, &Player)>) {

    for (mut transform, mut move_dir, health, player) in &mut players {

        if health.0 <= 0 {
            continue;
        }

        let mut direction = Vec2::ZERO;
        
        let (input, _) = inputs[player.handle];
        
        if input & INPUT_UP != 0 {
            direction.y += 1.0;
        }
        if input & INPUT_DOWN != 0 {
            direction.y -= 1.0;
        }
        if input & INPUT_RIGHT != 0 {
            direction.x += 1.0;
        }
        if input & INPUT_LEFT != 0 {
            direction.x -= 1.0;
        }

        if direction == Vec2::ZERO {
            continue;
        }

        direction = direction.normalize_or_zero();

        move_dir.0 = direction;

        let move_delta = direction * MOVE_SPEED;
        let limit = Vec2::splat(MAP_SIZE as f32  / 2.0 - 0.5);
        let old_pos = transform.translation.xy();
        let new_pos = (old_pos + move_delta).clamp(-limit, limit);
        transform.translation.x = new_pos.x;
        transform.translation.y = new_pos.y;
    }
}

pub fn reload_bullet(mut bullets: Query<&mut BulletReady>) {
    for mut can_fire in &mut bullets {
        if !can_fire.0 {
            can_fire.1 -= DELTA_TIME;
            if can_fire.1 <= 0.0 {
                can_fire.0 = true;
            }
        }
    }
}

pub fn fire_bullets(mut commands: Commands, inputs: Res<PlayerInputs<GgrsConfig>>, images: Res<ImageAssets>, mut players: Query<(&Transform, &Player, &mut BulletReady)>) {
    for (transform, player, mut bullet_ready) in &mut players {
        let (input, _) = inputs[player.handle];
        if input & INPUT_FIRE != 0 && bullet_ready.0 {
            let player_pos = transform.translation.xy();
            let network_angle = (input & INPUT_ANGLE) >> 8;
            let decoded_angle = ((network_angle as f32 / u8::MAX as f32) * (2.0 * std::f32::consts::PI)) - std::f32::consts::PI;
            let angle = Quat::from_rotation_z(decoded_angle);
            let shoot_direction = Vec2::new(decoded_angle.cos(), decoded_angle.sin());
            let pos = player_pos + shoot_direction * PLAYER_RADIUS + BULLET_RADIUS;
            commands.spawn((
                Bullet,
                MoveDir(shoot_direction),
                SpriteBundle {
                transform: Transform::from_translation(pos.extend(20.0)).with_rotation(angle),
                texture: images.bullet.clone(),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(0.3, 0.1)),
                    ..default()
                },
                ..default()
            }))
            .add_rollback();
            bullet_ready.0 = false;
            bullet_ready.1 = FIRE_RATE;
        }
    }
}

pub fn move_bullet(mut commands: Commands, mut bullets: Query<(Entity, &mut Transform, &MoveDir), With<Bullet>>) {

    for (bullet_entity, mut transform, dir) in &mut bullets {
        let delta = (dir.0 * 0.35).extend(0.0);
        transform.translation += delta;
        if transform.translation.x < -(MAP_SIZE as f32) / 2.0 || transform.translation.x > MAP_SIZE as f32 / 2.0 || transform.translation.y < -(MAP_SIZE as f32) / 2.0 || transform.translation.y > MAP_SIZE as f32 / 2.0 {
            commands.entity(bullet_entity).despawn_recursive();
        }
    }
}

pub fn kill_players(mut commands: Commands, mut players: Query<(&Transform, &mut Health, &Player, &mut Handle<Image>), (With<Player>, Without<Bullet>)>, bullets: Query<(Entity, &Transform), With<Bullet>>, mut scores: ResMut<Scores>, mut rollback_state: ResMut<RollbackState>, sprites: Res<ImageAssets>) {
    for (player_transform, mut health, player, mut sprite) in &mut players {

        if health.0 <= 0 {
            continue;
        }

        for (bullet_entity, bullet_transform) in &bullets {
            let distance = Vec2::distance(
                player_transform.translation.xy(),
                bullet_transform.translation.xy()
            );
            if distance < PLAYER_RADIUS + BULLET_RADIUS {
                commands.entity(bullet_entity).despawn_recursive();
                
                if player.handle == 0 {
                    scores.1 += 1;
                }
                else {
                    scores.0 += 1;
                }
                health.0 -= 200;
                *sprite = sprites.remains.clone();
                *rollback_state = RollbackState::Respawn;
                info!("Player Died: {scores:?}")
            }
        }
    }
}

pub fn update_sprites(inputs: Res<PlayerInputs<GgrsConfig>>, mut players: Query<(&mut Transform, &Health, &Player)>) {

    for (mut transform, health, player) in &mut players {

        if health.0 <= 0 {
            continue;
        }

        let (input, _) = inputs[player.handle];

        let network_angle = (input & INPUT_ANGLE) >> 8;
        let decoded_angle = ((network_angle as f32 / u8::MAX as f32) * (2.0 * std::f32::consts::PI)) - std::f32::consts::PI;
        let shoot_direction = Vec2::new(decoded_angle.cos(), decoded_angle.sin());

        if shoot_direction.x > 0.0 {
            transform.rotation = Quat::default();
        }
        else if shoot_direction.x < 0.0 {
            transform.rotation = Quat::from_rotation_y(std::f32::consts::PI);
        }
    }
}

//End of rollback functions