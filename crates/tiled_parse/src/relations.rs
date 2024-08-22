use crate::data_types::{Gid, TileSet};

pub fn get_tileset_for_gid(tilesets: &[TileSet], Gid(gid): Gid) -> Option<&TileSet> {
    tilesets
        .iter()
        .filter(|ts| ts.first_gid <= gid)
        .max_by_key(|ts| ts.first_gid)
}

pub fn get_tile_id(TileSet { first_gid, .. }: &TileSet, Gid(gid): Gid) -> u32 {
    gid - first_gid
}
