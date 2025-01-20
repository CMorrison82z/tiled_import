use std::marker::PhantomData;

use bevy::asset::{Asset, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::reflect;
use bevy::prelude::Visibility;
use bevy::reflect::{Reflect, TypePath};
use bevy::scene::{Scene, SceneBundle};
use bevy::sprite::TextureAtlasLayout;
use bevy::transform::components::{GlobalTransform, Transform};
use bevy::utils::hashbrown::HashMap;

use bincode::ErrorKind;
use serde::{Deserialize, Serialize};
use tiled_parse::data_types::TiledMap;

/// The scene bundle's `scene` should be just `default`
/// A useful struct for preparing a MapScene that has not necessarily been loaded yet.
#[derive(Component)]
#[require(Transform, Visibility)]
pub struct BufferedMapScene(pub Handle<TiledMapAsset>);

/// Marker type for a Tiled Map
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
    SerCollider,
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
