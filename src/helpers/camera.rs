use bevy::utils::Duration;

use bevy::{input::ButtonInput, math::Vec3, prelude::*, render::camera::Camera};

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

    pub const fn vec3(&self) -> Vec3 {
        match self {
            Direction::South => Vec3::NEG_Y,
            Direction::West => Vec3::NEG_X,
            Direction::East => Vec3::X,
            Direction::North => Vec3::Y,
        }
    }
}

#[derive(Reflect, Resource, Clone, Copy, PartialEq)]
pub struct MovementSpeed(pub f32);

impl Default for MovementSpeed {
    fn default() -> Self {
        Self(128.)
    }
}

#[derive(Reflect, Component)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
    pub direction: Direction,
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

    pub const fn from_direction(direction: Direction) -> Self {
        Self {
            first: direction.first_index(),
            last: direction.last_index(),
            direction,
        }
    }
}

#[derive(Reflect, Clone, Copy, PartialEq, Eq)]
pub enum MovementMode {
    Sprinting,
    Normal,
    Idle,
}

impl MovementMode {
    pub const fn timer_duration_f32(&self) -> f32 {
        match self {
            MovementMode::Sprinting => 0.1,
            MovementMode::Normal => 0.2,
            MovementMode::Idle => 0.8,
        }
    }

    pub fn timer_duration(&self) -> Duration {
        Duration::from_secs_f32(self.timer_duration_f32())
    }

    pub const fn movement_multiplier(&self) -> f32 {
        match self {
            MovementMode::Sprinting => 1.5,
            MovementMode::Normal => 1.,
            MovementMode::Idle => 0.,
        }
    }
}

#[derive(Reflect, Component, Deref, DerefMut)]
pub struct AnimationTimer {
    #[deref]
    pub timer: Timer,
    movement_mode: MovementMode,
}

impl AnimationTimer {
    pub fn new(movement_mode: MovementMode) -> Self {
        return Self {
            movement_mode: movement_mode,
            timer: Timer::from_seconds(
                movement_mode.timer_duration_f32(),
                bevy::time::TimerMode::Repeating,
            ),
        };
    }

    pub fn set_movement_mode(&mut self, movement_mode: MovementMode) {
        self.movement_mode = movement_mode;
        self.set_duration(movement_mode.timer_duration());
    }
}

// A simple camera system for moving and zooming the camera.
#[allow(dead_code)]
pub fn movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    movement_speed: Res<MovementSpeed>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    for mut transform in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        let movement_mode = if keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
        {
            MovementMode::Sprinting
        } else {
            MovementMode::Normal
        };

        if keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
            direction += Direction::West.vec3();
        }
        if keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
            direction += Direction::East.vec3();
        }
        if keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
            direction += Direction::North.vec3();
        }
        if keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
            direction += Direction::South.vec3();
        }

        direction = direction.normalize_or_zero();

        let z = transform.translation.z;
        transform.translation += time.delta_seconds()
            * direction
            * movement_speed.0
            * movement_mode.movement_multiplier();
        // Important! We need to restore the Z values when moving the camera around.
        // Bevy has a specific camera setup and this can mess with how our layers are shown.
        transform.translation.z = z;
    }
}
