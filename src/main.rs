use std::path::Path;

use bevy::utils::Duration;

use bevy::{asset::ChangeWatcher, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use helpers::pokemon_loader::*;

mod helpers;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    South,
    West,
    East,
    North,
}

#[derive(Resource, Clone, Copy, PartialEq)]
pub struct MovementSpeed(f32);

impl Default for MovementSpeed {
    fn default() -> Self {
        Self(128.)
    }
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

#[derive(Component)]
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

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn(Camera2dBundle::default());

    let map_handle: Handle<helpers::tiled::TiledMap> =
        asset_server.load(Path::new("tilemaps").join("tuxemon-town.tmx"));

    let tile_size = TilemapTileSize { x: 64.0, y: 64.0 };
    commands.spawn(helpers::tiled::TiledMapBundle {
        tiled_map: map_handle,
        transform: Transform::from_scale(Vec3::new(2., 2., 1.)),
        ..Default::default()
    });

    let asset_path = Asset::OverworldSprite(3, 0, Shinyness::Shiny).get_path();
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
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
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
    )>,
) {
    for (mut sprite, mut transform, mut animation_indices) in query.iter_mut() {
        let mut direction = Vec3::ZERO;
        let mut facing = animation_indices.direction;

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
        transform.translation += time.delta_seconds() * direction * movement_speed.0;
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
        .add_plugins(TilemapPlugin)
        .add_plugins(helpers::tiled::TiledMapPlugin)
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (animate_sprite, move_sprite, helpers::camera::movement),
        )
        .run();
}
