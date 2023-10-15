use bevy::{prelude::*, render::camera::ScalingMode};

const MOVE_SPEED: f32 = 0.13;

#[derive(Component)]
struct Player;

fn main() {
    App::new().add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            fit_canvas_to_parent: true,
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    }))
    .insert_resource(ClearColor(Color::rgb(0.53, 0.53, 0.53)))
    .add_systems(Startup, (setup, spawn_player))
    .add_systems(Update, move_player)
    .run();
}

fn setup(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::FixedVertical(10.0);
    commands.spawn(camera_bundle);
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.0, 0.47, 1.0),
                custom_size: Some(Vec2::new(1.0, 1.0)),
                ..default()
            },
            ..default()
        },
    ));
}

fn move_player(keys: Res<Input<KeyCode>>, mut players: Query<&mut Transform, With<Player>>) {
    let mut direction = Vec2::ZERO;
    if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
        direction.y += 1.0;
    }
    if keys.any_pressed([KeyCode::Up, KeyCode::S]) {
        direction.y -= 1.0;
    }
    if keys.any_pressed([KeyCode::Up, KeyCode::D]) {
        direction.x += 1.0;
    }
    if keys.any_pressed([KeyCode::Up, KeyCode::A]) {
        direction.x -= 1.0;
    }

    let move_delta = (direction * MOVE_SPEED).extend(0.0);
    for mut transform in &mut players {
        transform.translation += move_delta;
    }
}