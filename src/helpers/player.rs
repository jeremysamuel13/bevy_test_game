use avian2d::prelude::*;
use bevy::prelude::*;

use crate::Shinyness;

use super::{
    camera::{Direction, *},
    pokemon_loader::{Asset, PokedexNumber},
};

#[derive(Bundle)]
pub struct EntityPhysics {
    rigid_body: RigidBody,
    collider: Collider,
    gravity_scale: GravityScale,
}

#[derive(Bundle)]
pub struct AnimatedSprite {
    sprite_bundle: SpriteBundle,
    texture_atlas: TextureAtlas,
    animation_indices: AnimationIndices,
    sprite_animation_timer: AnimationTimer,
}

pub fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut TextureAtlas)>,
) {
    for (indices, mut timer, mut atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            atlas.index = if atlas.index == indices.last {
                indices.first
            } else {
                atlas.index + 1
            };
        }
    }
}

#[derive(Bundle)]
pub struct OverworldPokemonBundle {
    animated_sprite: AnimatedSprite,
    physics: EntityPhysics,
}

impl OverworldPokemonBundle {
    pub fn new(
        pokedex_id: PokedexNumber,
        asset_server: &Res<AssetServer>,
        mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    ) -> Self {
        let texture_handle =
            Asset::OverworldSprite(pokedex_id, 0, Shinyness::Shiny).load(&asset_server);
        let texture_atlas_layout =
            TextureAtlasLayout::from_grid(UVec2::new(64, 64), 4, 4, None, None);

        let texture_atlas_layout_handle = texture_atlases.add(texture_atlas_layout);
        let animation_indices = AnimationIndices::from_direction(Direction::South);

        return Self {
            animated_sprite: AnimatedSprite {
                sprite_bundle: SpriteBundle {
                    texture: texture_handle,
                    transform: Transform::from_scale(Vec3::new(0.5, 0.5, 1.))
                        .with_translation(Vec3::new(0., 0., 10.)),
                    ..Default::default()
                },
                texture_atlas: TextureAtlas {
                    layout: texture_atlas_layout_handle,
                    index: animation_indices.first,
                },
                animation_indices,
                sprite_animation_timer: AnimationTimer::new(MovementMode::Idle),
            },
            physics: EntityPhysics {
                rigid_body: RigidBody::Dynamic,
                collider: Collider::circle(10.0),
                gravity_scale: GravityScale(10.0),
            },
        };
    }
}
