use bevy::prelude::*;
use tiled_parse::{relations::{get_tile_id, get_tileset_for_gid}, types::{Gid, TiledLayer}};
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
pub fn get_layer_from_id(a: &TiledMapAsset, t_id: TiledId) -> Option<&TiledLayer> {
    let id = match_ok!(t_id, TiledId::Layer(x))?;

    a.map.get_layer_from_id(id)
}

pub fn get_tile_sprite(a: &TiledMapAsset, t_gid: Gid) -> Sprite {
    let tile_sets = &a.map.tile_sets;

    let tile_tileset = get_tileset_for_gid(&tile_sets, t_gid)
        .expect("Tile should belong to tileset");

    let tileset_index = tile_sets
        .iter()
        .position(|ts| ts.first_gid == tile_tileset.first_gid)
        .expect("Yes");

    let local_tile_id = get_tile_id(&tile_tileset, t_gid);

    Sprite {
        image: a.tilemap_textures.get(tileset_index).unwrap().clone(),
        texture_atlas: Some(TextureAtlas {
            layout: a.tilemap_atlases.get(tileset_index).unwrap().clone(),
            index: local_tile_id as usize,
        }),
        anchor: bevy::sprite::Anchor::TopLeft,
        ..Default::default()
    }
}
impl TiledId {
    pub fn as_tile_gid(&self) -> Option<Gid> {
        match_ok!(self, TiledId::Tile(x)).map(|&id| Gid(id))
    }
}
