use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_ggrs::{ggrs::PlayerType, *};
use bevy_matchbox::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_egui::{egui::{self, Align2, Color32, FontId, RichText}, EguiContexts, EguiPlugin};
use input::*;
use components::*;
use rollback_functions::*;
use resources::*;

mod input;
mod components;
mod rollback_functions;
mod resources;

//Use matchbox_server in cmd window then keep open while working. Just keeps server up
//Recompile with cargo run --release --target wasm32-unknown-unknown
//Testing url http://127.0.0.1:1334

const MAP_SIZE: u32 = 41;
const GRID_WIDTH: f32 = 0.05;

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum GameState{
    #[default]
    AssetLoading,
    Matchmaking,
    InGame,
}

fn main() {
    App::new().add_state::<GameState>()
    .add_loading_state(LoadingState::new(GameState::AssetLoading).continue_to_state(GameState::Matchmaking))
    .add_collection_to_loading_state::<_, ImageAssets>(GameState::AssetLoading)
    .add_plugins((DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            fit_canvas_to_parent: true,
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
        }),
        EguiPlugin,
    ))
    .add_ggrs_plugin(GgrsPlugin::<GgrsConfig>::new()
        .with_input_system(input)
        .register_rollback_component::<Transform>()
        .register_rollback_component::<BulletReady>()
        .register_rollback_component::<MoveDir>()
        .register_rollback_resource::<Scores>()
        .register_rollback_resource::<RollbackState>(),
    )
    .init_resource::<Scores>()
    .init_resource::<RollbackState>()
    .insert_resource(ClearColor(Color::rgb(0.53, 0.53, 0.53)))
    .add_systems(OnEnter(GameState::Matchmaking), (setup, start_matchbox_socket))
    .add_systems(OnEnter(GameState::InGame), spawn_players)
    .add_systems(Update,  (wait_for_players.run_if(in_state(GameState::Matchmaking)), 
        (camera_follow, update_score_ui).run_if(in_state(GameState::InGame)),),)
    .add_systems(GgrsSchedule, (move_player, reload_bullet, fire_bullets, move_bullet, kill_players).chain())
    .run();
}

//Using .chain() forces one to run after another, using .after() allows for parallel exucation as long as the same components are not effected in each function
//Web does not allow for parallel execution so .chain() is fine

//All functions below runs once when we start matching making
fn setup(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::FixedVertical(10.0);
    commands.spawn(camera_bundle);

    for i in 0..=MAP_SIZE {
        //Horizontal line
        commands.spawn(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(
                0.0, 
                i as f32 - MAP_SIZE as f32 / 2.0,
                0.0,
            )),
            sprite: Sprite {
                color: Color::rgb(0.27, 0.27, 0.27),
                custom_size: Some(Vec2::new(MAP_SIZE as f32, GRID_WIDTH)),
                ..default()
            },
            ..default()
        });

        //Vertical line
        commands.spawn(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(
                i as f32 - MAP_SIZE as f32 / 2.0,
                0.0, 
                0.0,
            )),
            sprite: Sprite {
                color: Color::rgb(0.27, 0.27, 0.27),
                custom_size: Some(Vec2::new(GRID_WIDTH, MAP_SIZE as f32)),
                ..default()
            },
            ..default()
        });
    }
}

fn start_matchbox_socket(mut commands: Commands) {
    let room_url = "ws://127.0.0.1:3536/web_shooter?next=2";
    info!("Connecting to matchbox server: {room_url}");
    commands.insert_resource(MatchboxSocket::new_ggrs(room_url));
}

//All functions below runs once when we start the game
fn spawn_players(mut commands: Commands) {
    commands.spawn((
        Player { handle: 0 },
        BulletReady(true),
        MoveDir(-Vec2::X),
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(-2.0, 0.0, 10.0)),
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
        BulletReady(true),
        MoveDir(Vec2::X),
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(2.0, 0.0, 10.0)),
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

//All functions below run ever frame while we are match making
fn wait_for_players(mut commands: Commands, mut socket: ResMut<MatchboxSocket<SingleChannel>>, mut next_state: ResMut<NextState<GameState>>) {
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

    next_state.set(GameState::InGame);
}

//All functions below run every frame when we are playing
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

fn update_score_ui(mut context: EguiContexts, scores: Res<Scores>, rollback_state: ResMut<RollbackState>) {
    let Scores(p1_score, p2_score) = *scores;

    let current_state = *rollback_state;

    egui::Area::new("Score").anchor(Align2::CENTER_TOP, (0.0, 25.0))
        .show(context.ctx_mut(), |ui| {
            ui.label(RichText::new(format!("{p1_score} - {p2_score} {current_state:?}"))
                .color(Color32::BLACK)
                .font(FontId::proportional(72.0)),
            );
        });
}