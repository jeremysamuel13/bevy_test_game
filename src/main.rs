use std::path::Path;

use bevy::input::mouse::MouseWheel;
use bevy::math::VectorSpace;
use bevy::prelude::*;

use bevy_ecs_ldtk::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use helpers::camera::{AnimationIndices, AnimationTimer, Direction, MovementMode, MovementSpeed};
use helpers::player::{animate_sprite, OverworldPokemonBundle};
use helpers::pokemon_loader::*;

use rand::distributions::Uniform;
use rand::prelude::Distribution;
use rand::thread_rng;

mod helpers;

#[derive(Component)]
struct MainPlayer;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load(Path::new("tilemaps").join("pkmn.ldtk")),
        ..Default::default()
    });

    let mut trng = thread_rng();
    let dist = Uniform::new(0, 899);

    commands
        .spawn(OverworldPokemonBundle::new(
            dist.sample(&mut trng),
            &asset_server,
            texture_atlases,
        ))
        .insert(MainPlayer);
}

fn zoom_camera(
    mut scroll_evr: EventReader<MouseWheel>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    const ZOOM_SENSITIVITY: f32 = 0.1;
    const MIN_ZOOM: f32 = 0.01;
    const MAX_ZOOM: f32 = 5.0;

    for ev in scroll_evr.read() {
        for mut transform in query.iter_mut() {
            // Adjust the camera scale based on the scroll wheel
            let zoom_change = ZOOM_SENSITIVITY * ev.y;
            transform.scale.x = (transform.scale.x - zoom_change).clamp(MIN_ZOOM, MAX_ZOOM);
            transform.scale.y = (transform.scale.y - zoom_change).clamp(MIN_ZOOM, MAX_ZOOM);
        }
    }
}

fn move_sprite(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    movement_speed: Res<MovementSpeed>,
    mut query: Query<
        (
            &mut TextureAtlas,
            &mut Transform,
            &mut AnimationIndices,
            &mut AnimationTimer,
        ),
        With<MainPlayer>,
    >,
) {
    for (mut sprite, mut transform, mut animation_indices, mut timer) in query.iter_mut() {
        let mut direction = Vec3::ZERO;
        let mut facing = animation_indices.direction;

        if keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
            facing = Direction::West;
            direction += facing.vec3();
        }
        if keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
            facing = Direction::East;
            direction += facing.vec3();
        }
        if keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
            facing = Direction::North;
            direction += facing.vec3();
        }
        if keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
            facing = Direction::South;
            direction += facing.vec3();
        }

        // facing
        if animation_indices.change_direction(facing) {
            sprite.index = animation_indices.first;
        }

        // movement
        let is_moving = direction != Vec3::ZERO;

        let movement_mode = match (
            is_moving,
            keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
        ) {
            (true, true) => MovementMode::Sprinting,
            (true, false) => MovementMode::Normal,
            _ => MovementMode::Idle,
        };

        timer.set_movement_mode(movement_mode);

        if movement_mode == MovementMode::Idle {
            return;
        }

        let z = transform.translation.z;

        let new_translation = transform.translation + time.delta_seconds()
        * direction.normalize_or_zero()
        * movement_speed.0
        * movement_mode.movement_multiplier();
        
        // Important! We need to restore the Z values when moving the camera around.
        // Bevy has a specific camera setup and this can mess with how our layers are shown.
        transform.translation.z = z;
    }
}

fn main() {
    App::new()
        .init_resource::<MovementSpeed>()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Tiled Map Editor Example"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(LdtkPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                animate_sprite,
                move_sprite,
                helpers::camera::movement,
                zoom_camera,
            ),
        )
        .insert_resource(LevelSelection::index(0))
        .run();
}
