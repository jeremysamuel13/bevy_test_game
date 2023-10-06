use std::path::Path;

use bevy::utils::Duration;

use bevy::{asset::ChangeWatcher, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;


use helpers::pokemon_loader::*;

use rand::distributions::Uniform;
use rand::prelude::Distribution;
use rand::thread_rng;

mod helpers;

#[derive(Reflect, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    South,
    West,
    East,
    North,
}

impl Direction {
    pub const fn first_index(&self) -> usize {
        match self {
            Direction::South => 0,
            Direction::West => 4,
            Direction::East => 8,
            Direction::North => 12,
        }
    }

    pub const fn last_index(&self) -> usize {
        self.first_index() + 3
    }
}

#[derive(Reflect, Resource, Clone, Copy, PartialEq)]
pub struct MovementSpeed(f32);

impl Default for MovementSpeed {
    fn default() -> Self {
        Self(128.)
    }

    
}

#[derive(Reflect, Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
    direction: Direction,
}

impl AnimationIndices {
    pub fn change_direction(&mut self, dir: Direction) -> bool {
        if self.direction == dir {
            return false;
        }

        self.direction = dir;
        self.first = self.direction.first_index();
        self.last = self.direction.last_index();

        true
    }

    pub fn from_direction(direction: Direction) -> Self {
        Self {
            first: direction.first_index(),
            last: direction.last_index(),
            direction,
        }
    }
}

#[derive(Reflect, Clone, Copy, PartialEq, Eq)]
enum MovementMode {
    Sprinting,
    Normal
}

impl MovementMode {
    pub const fn timer_duration_f32(&self) -> f32 {
        match self {
            MovementMode::Sprinting => 0.1,
            MovementMode::Normal => 0.2,
        }
    }

    pub fn timer_duration(&self) -> Duration {
        Duration::from_secs_f32(self.timer_duration_f32())
    }

    pub const fn movement_multiplier(&self) -> f32 {
        match self {
            MovementMode::Sprinting => 1.5,
            MovementMode::Normal => 1.,
        }
    }
}

#[derive(Reflect, Component, Deref, DerefMut)]
struct SpriteAnimationTimer(Timer);

impl SpriteAnimationTimer {
    fn set_movement_mode(&mut self, mode: MovementMode) {
        self.set_duration(mode.timer_duration())
    }
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn(Camera2dBundle::default());

    let map_handle: Handle<helpers::tiled::TiledMap> =
        asset_server.load(Path::new("tilemaps").join("tuxemon-town.tmx"));

    commands.spawn(helpers::tiled::TiledMapBundle {
        tiled_map: map_handle,
        transform: Transform::from_scale(Vec3::new(2., 2., 1.)),
        ..Default::default()
    });

    let mut trng = thread_rng();
    let dist = Uniform::new(0, 899);

    let asset_path = Asset::OverworldSprite(dist.sample(&mut trng), 0, Shinyness::Shiny).get_path();
    let texture_handle = asset_server.load(asset_path);
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(64., 64.), 4, 4, None, None);

    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let animation_indices = AnimationIndices::from_direction(Direction::South);

    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(animation_indices.first),
            transform: Transform::from_translation(Vec3::new(0., 0., 2.)),
            ..Default::default()
        },
        animation_indices,
        SpriteAnimationTimer(Timer::from_seconds(MovementMode::Normal.timer_duration_f32(), bevy::time::TimerMode::Repeating)),
    ));
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut SpriteAnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
}

fn move_sprite(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    movement_speed: Res<MovementSpeed>,
    mut query: Query<(
        &mut TextureAtlasSprite,
        &mut Transform,
        &mut AnimationIndices,
        &mut SpriteAnimationTimer,
    )>,
) {
    for (mut sprite, mut transform, mut animation_indices, mut timer) in query.iter_mut() {
        let mut direction = Vec3::ZERO;
        let mut facing = animation_indices.direction;

        let movement_mode = if keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            MovementMode::Sprinting
        } else {
            MovementMode::Normal
        };

        timer.set_movement_mode(movement_mode);

        if keyboard_input.any_pressed([KeyCode::A, KeyCode::Left]) {
            direction -= Vec3::new(1.0, 0.0, 0.0);
            facing = Direction::West;
        }
        if keyboard_input.any_pressed([KeyCode::D, KeyCode::Right]) {
            direction += Vec3::new(1.0, 0.0, 0.0);
            facing = Direction::East;
        }

        if keyboard_input.any_pressed([KeyCode::W, KeyCode::Up]) {
            direction += Vec3::new(0.0, 1.0, 0.0);
            facing = Direction::North;
        }

        if keyboard_input.any_pressed([KeyCode::S, KeyCode::Down]) {
            direction -= Vec3::new(0.0, 1.0, 0.0);
            facing = Direction::South;
        }

        if animation_indices.change_direction(facing) {
            sprite.index = animation_indices.first;
        }

        direction = direction.normalize_or_zero();

        let z = transform.translation.z;
        transform.translation += time.delta_seconds() * direction * movement_speed.0 * movement_mode.movement_multiplier();
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
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
                    ..default()
                }),
        )
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(TilemapPlugin)
        .add_plugins(helpers::tiled::TiledMapPlugin)
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (animate_sprite, move_sprite, helpers::camera::movement),
        )
        .run();
}
