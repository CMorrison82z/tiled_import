use bevy::prelude::*;
use tiled_parse::data_types::TiledLayer;

use crate::types::{TiledLayerId, TiledMapAsset};

/// Given a `TiledMapAsset`, returns `Entity`'s with a SceneRoot containing `TiledMapAsset.scene`.
pub fn get_entities_with_tiled_map(a: &TiledMapAsset, q: &Query<(Entity, &SceneRoot)>) -> Vec<Entity> {
    q.iter().filter_map(|(e, SceneRoot(h))|
        if a.scene == *h {Some(e)} else {None}
    ).collect()
}

/// Given a `SceneRoot`, returns the `TiledMapAsset` with the associated `TiledMapAsset.scene`.
pub fn get_tiled_map_with_scene<'a>(a: &'a Res<Assets<TiledMapAsset>>, SceneRoot(h): &SceneRoot) -> Option<&'a TiledMapAsset> {
    a.iter().find_map(|(_, tm)| if tm.scene == *h {Some(tm)} else {None})
}

/// Given a `TiledLayer` from a `TiledMapAsset`.
pub fn get_layer_from_id(a: &TiledMapAsset, TiledLayerId(id): TiledLayerId) -> Option<&TiledLayer> {
    tiled_parse::relations::get_layer_from_id(&a.map.layers, id)
}
