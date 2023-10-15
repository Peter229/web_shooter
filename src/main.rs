use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_ggrs::{ggrs::PlayerType, *};
use bevy_matchbox::prelude::*;
use bevy_matchbox::matchbox_socket::PeerId;
use input::*;
use components::*;

mod input;
mod components;

const MOVE_SPEED: f32 = 0.13;

#[derive(Resource)]
struct LocalPlayerHandle(usize);

struct GgrsConfig;

impl ggrs::Config for GgrsConfig {
    type Input = u8;
    type State = u8;
    type Address = PeerId;
}

fn main() {
    App::new().add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            fit_canvas_to_parent: true,
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    }))
    .add_ggrs_plugin(GgrsPlugin::<GgrsConfig>::new()
        .with_input_system(input)
        .register_rollback_component::<Transform>(),)
    .insert_resource(ClearColor(Color::rgb(0.53, 0.53, 0.53)))
    .add_systems(Startup, (setup, spawn_players, start_matchbox_socket))
    .add_systems(Update,  (wait_for_players, camera_follow))
    .add_systems(GgrsSchedule, move_player)
    .run();
}

fn setup(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::FixedVertical(10.0);
    commands.spawn(camera_bundle);
}

fn spawn_players(mut commands: Commands) {
    commands.spawn((
        Player { handle: 0 },
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(-2.0, 0.0, 0.0)),
            sprite: Sprite {
                color: Color::rgb(0.0, 0.47, 1.0),
                custom_size: Some(Vec2::new(1.0, 1.0)),
                ..default()
            },
            ..default()
        },
    ))
    .add_rollback();

    commands.spawn((
        Player { handle: 1 },
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)),
            sprite: Sprite {
                color: Color::rgb(0.0, 0.4, 0.0),
                custom_size: Some(Vec2::new(1.0, 1.0)),
                ..default()
            },
            ..default()
        },
    ))
    .add_rollback();
}

fn start_matchbox_socket(mut commands: Commands) {
    let room_url = "ws://127.0.0.1:3536/web_shooter?next=2";
    info!("Connecting to matchbox server: {room_url}");
    commands.insert_resource(MatchboxSocket::new_ggrs(room_url));
}

fn move_player(inputs: Res<PlayerInputs<GgrsConfig>>, mut players: Query<(&mut Transform, &Player)>) {
    
    for (mut transform, player) in &mut players {

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

        let move_delta = (direction * MOVE_SPEED).extend(0.0);
        transform.translation += move_delta;
    }
}

fn wait_for_players(mut commands: Commands, mut socket: ResMut<MatchboxSocket<SingleChannel>>) {
    if socket.get_channel(0).is_err() {
        return;
    }

    socket.update_peers();
    let players = socket.players();
    
    let num_players = 2;
    if players.len() < num_players {
        return;
    }

    info!("All peers have joined, starting game!");

    let mut session_builder = ggrs::SessionBuilder::<GgrsConfig>::new()
        .with_num_players(num_players)
        .with_input_delay(1);

    for (i, player) in players.into_iter().enumerate() {
        if player == PlayerType::Local {
            commands.insert_resource(LocalPlayerHandle(i));
        }

        session_builder = session_builder.add_player(player, i)
            .expect("Failed to add player");
    }

    let channel = socket.take_channel(0).unwrap();

    let ggrs_session = session_builder.start_p2p_session(channel)
        .expect("Failed to start session");

    commands.insert_resource(bevy_ggrs::Session::P2P(ggrs_session));
}

fn camera_follow(
    player_handle: Option<Res<LocalPlayerHandle>>,
    players: Query<(&Player, &Transform)>,
    mut cameras: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    let player_handle = match player_handle {
        Some(handle) => handle.0,
        None => return,
    };

    for (player, player_transform) in &players {
        if player.handle != player_handle {
            continue;
        }

        let pos = player_transform.translation;

        for mut transform in &mut cameras {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        }
    }
}