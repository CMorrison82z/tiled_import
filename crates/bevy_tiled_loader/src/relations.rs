use bevy::prelude::*;
use tiled_parse::types::{Gid, TiledLayer};
use try_match::match_ok;

use crate::types::{TiledId, TiledMapAsset};

/// Given a `TiledMapAsset`, returns `Entity`'s with a SceneRoot containing `TiledMapAsset.scene`.
pub fn get_entities_with_tiled_map(
    a: &TiledMapAsset,
    q: &Query<(Entity, &SceneRoot)>,
) -> Vec<Entity> {
    q.iter()
        .filter_map(|(e, SceneRoot(h))| if a.scene == *h { Some(e) } else { None })
        .collect()
}

/// Given a `SceneRoot`, returns the `TiledMapAsset` with the associated `TiledMapAsset.scene`.
pub fn get_tiled_map_with_scene<'a>(
    a: &'a Res<Assets<TiledMapAsset>>,
    SceneRoot(h): &SceneRoot,
) -> Option<&'a TiledMapAsset> {
    a.iter()
        .find_map(|(_, tm)| if tm.scene == *h { Some(tm) } else { None })
}

/// Given a `TiledLayer` from a `TiledMapAsset`.
pub fn get_layer_from_id(a: &TiledMapAsset, tid: TiledId) -> Option<&TiledLayer> {
    let id = match_ok!(tid, TiledId::Layer(x))?;

    a.map.get_layer_from_id(id)
}

impl TiledId {
    pub fn as_tile_gid(&self) -> Option<Gid> {
        match_ok!(self, TiledId::Tile(x)).map(|&id| Gid(id))
    }
}
