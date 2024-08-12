use bevy::asset::{Asset, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::reflect::TypePath;
use bevy::scene::Scene;
use bevy::sprite::TextureAtlasLayout;
use bevy::transform::components::{GlobalTransform, Transform};
use bevy::utils::hashbrown::HashMap;

use tiled_parse::data_types::TiledMap;

#[derive(TypePath, Asset)]
pub struct TiledMapAsset {
    pub map: TiledMap,

    // TODO:
    // pub colliders: todo!(),
    pub tilemap_textures: Vec<Handle<bevy::prelude::Image>>,
    pub tilemap_atlases: Vec<Handle<TextureAtlasLayout>>,
    pub scene: Handle<Scene>,
}

// Stores a list of tiled layers.
#[derive(Component, Default)]
pub struct TiledLayersStorage {
    pub storage: HashMap<u32, Entity>,
}

// TODO:
// I'm not sure that I want to have the crate commit to this instance implementation...
// For example, GPU rendering would be more efficient (like `bevy_ecs_tilemap`)
#[derive(Default, Bundle)]
pub struct TiledMapBundle {
    pub tiled_map: Handle<TiledMapAsset>,
    pub storage: TiledLayersStorage,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}
