use bevy::asset::{Asset, Handle};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::Event;
use bevy::prelude::Visibility;
use bevy::reflect::{impl_reflect, Reflect, TypePath};
use bevy::scene::Scene;
use bevy::image::TextureAtlasLayout;
use bevy::transform::components::Transform;
use bevy_platform::collections::HashMap;

use tiled_parse::types::*;

/// When the dependencies have loaded, will add `SceneRoot(TiledMapAsset.scene)` with `TiledMapScene`
#[derive(Component)]
#[require(Transform, Visibility)]
pub struct TiledMapScene(pub Handle<TiledMapAsset>);

/// Marker type for a Tiled Map
#[derive(Component, Reflect, Copy, Clone, Debug)]
pub struct TiledMapContainer;

#[derive(Component, Reflect, Copy, Clone, Debug)]
pub enum TiledId {
    Layer(tiled_parse::types::ID),
    /// Note that this is actually the `Gid` of a tile.
    Tile(tiled_parse::types::ID),
    Object(tiled_parse::types::ID),
}

#[derive(Component, Reflect, Copy, Clone, Debug)]
pub struct TileObject(pub tiled_parse::types::ID);

#[derive(Component, Reflect, Copy, Clone, Debug)]
pub struct TiledIndex(pub usize, pub usize);

impl Into<(usize, usize)> for TiledIndex {
    fn into(self) -> (usize, usize) {
        (self.0, self.1)
    }
}

#[derive(Reflect, Copy, Clone, Debug)]
pub enum TiledAnimationPlayer {
    Once,
    Cycled
}

#[derive(Component, Reflect, Clone, Debug)]
pub struct TiledAnimation {
    // TODO:
    // Animation becomes an Asset. This will become a `Handle<TiledAnimation>`
    pub animation: Vec<AnimationFrameReflect>,
    /// In Seconds
    pub start_time: f32,
    pub player: TiledAnimationPlayer
}

// TODO:
// Try to impl reflect on foreign type `tiled_parse::AnimationFrame`
#[derive(Reflect, Copy, Clone, Debug)]
pub struct AnimationFrameReflect {
    pub tile_id: ID,
    pub duration: f32
}

impl From<AnimationFrame> for AnimationFrameReflect {
    fn from(AnimationFrame { tile_id, duration }: AnimationFrame) -> Self {
        Self { tile_id, duration }
    }
}

/// Triggers when a tween completes (regardless of if it cycles)
#[derive(Event, Reflect, Copy, Clone, Debug)]
pub struct TiledAnimationCompleted {
    pub entity: Entity,
}

#[derive(TypePath, Asset)]
pub struct TiledMapAsset {
    pub map: tiled_parse::types::TiledMap,

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
#[derive(Component, Reflect, Debug)]
pub struct SerializedComponents(pub HashMap<SceneSerializedComponents, Vec<u8>>);

#[derive(Reflect, PartialEq, Eq, Hash, Clone, Copy, Debug)]
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
