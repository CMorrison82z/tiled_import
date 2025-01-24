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

/// When the dependencies have loaded, will replace `BufferedMapScene` with a
/// `SceneRoot(TiledMapAsset.scene)`
#[derive(Component)]
#[require(Transform, Visibility)]
pub struct BufferedMapScene(pub Handle<TiledMapAsset>);

/// Marker type for a Tiled Map
#[derive(Component, Reflect)]
pub struct TiledMapContainer;

#[derive(Component, Reflect)]
pub enum TiledId {
    Layer(tiled_parse::data_types::ID),
    /// Note that this is actually the `Gid` of a tile.
    Tile(tiled_parse::data_types::ID),
    Object(tiled_parse::data_types::ID),
}

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
//
/// `tiled_scene_plugin` will deserialize components added to Entities within a TiledMapContainer
/// Scene.
///
/// Components like `Collider` are not `Reflect` (perhaps use ColliderConstructor instead ?), so they need to be Serialized. In the case of
/// `RigidBody`, they have a lot of required Components, so deserializing them once they've been
/// added to the main App where its required Components are in the type registry is more
/// convenient. This is likely subject to change.
#[derive(Component, Reflect)]
pub struct SerializedComponents(pub HashMap<SceneSerializedComponents, Vec<u8>>);

#[derive(Reflect, PartialEq, Eq, Hash, Clone, Copy)]
pub enum SceneSerializedComponents {
    SerCollider,
    SerRigidBody,
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
