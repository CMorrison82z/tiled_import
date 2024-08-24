use std::marker::PhantomData;

use bevy::asset::{Asset, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::reflect;
use bevy::reflect::{Reflect, TypePath};
use bevy::scene::Scene;
use bevy::sprite::TextureAtlasLayout;
use bevy::transform::components::{GlobalTransform, Transform};
use bevy::utils::hashbrown::HashMap;

use bincode::ErrorKind;
use serde::{Deserialize, Serialize};
use tiled_parse::data_types::TiledMap;

#[derive(Component, Reflect)]
pub struct TiledMapContainer;

#[derive(TypePath, Asset)]
pub struct TiledMapAsset {
    pub map: TiledMap,

    // TODO:
    // pub colliders: todo!(),
    pub tilemap_textures: Vec<Handle<bevy::prelude::Image>>,
    pub tilemap_atlases: Vec<Handle<TextureAtlasLayout>>,
    pub scene: Handle<Scene>,
}

// TODO:
// I'm not sure that I want to have the crate commit to this instance implementation...
// For example, GPU rendering would be more efficient (like `bevy_ecs_tilemap`)
#[derive(Default, Bundle)]
pub struct TiledMapBundle {
    pub tiled_map: Handle<TiledMapAsset>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

// NOTE:
// While this could be made further general to any Serailizer, because of how `serde` Serailizers
// are suggested to be written (the result byte stream stored within the instance, without a clear
// way of extracting it), I cannot easily generalize it...
#[derive(Component, Reflect)]
// pub struct Serialized<T: Serialize + for<'de> Deserialize<'de>> {
pub struct Serialized {
    pub data: Vec<u8>,
    pub thingy: SceneSerializedComponents,
}

#[derive(Reflect)]
pub enum SceneSerializedComponents {
    RCollider,
}

// impl<T> Serialized<T> {
//     pub fn new(data: T) -> Result<Self, Box<ErrorKind>> {
//         bincode::serialize(&data).map(|data| Serialized {
//             data,
//             _marker: PhantomData,
//         })
//     }
//
//     pub fn deserialize(&self) -> Result<T, Box<ErrorKind>> {
//         bincode::deserialize(&self.data)
//     }
// }
