use tiled_parse_tree::TreeZipper;

use crate::data_types::*;

pub fn get_tileset_for_gid(tilesets: &[TileSet], Gid(gid): Gid) -> Option<&TileSet> {
    tilesets
        .iter()
        .filter(|ts| ts.first_gid <= gid)
        .max_by_key(|ts| ts.first_gid)
}

pub fn get_tile_id(TileSet { first_gid, .. }: &TileSet, Gid(gid): Gid) -> u32 {
    gid - first_gid
}

impl TiledMap {
    pub fn get_layer_from_id(&self, id: ID) -> Option<&TiledLayer> {
        self.layers.iter_breadth().find(|l| l.id == id)
    }

    pub fn get_tile_aux_info(&self, gid: Gid) -> Option<&TileAuxInfo> {
        let ts = get_tileset_for_gid(self.tile_sets.as_slice(), gid)?;
        ts.tile_stuff.get(&get_tile_id(ts, gid))
    }

    pub fn get_tile_properties(&self, gid: Gid) -> Option<&Properties> {
        get_tileset_for_gid(self.tile_sets.as_slice(), gid).and_then(|ts| {
            ts.tile_stuff
                .get(&get_tile_id(ts, gid))
                .map(|tai| &tai.properties)
        })
    }

    pub fn get_tile_objects(&self, gid: Gid) -> Option<&Vec<Object>> {
        get_tileset_for_gid(self.tile_sets.as_slice(), gid).and_then(|ts| {
            ts.tile_stuff
                .get(&get_tile_id(ts, gid))
                .map(|tai| &tai.objects)
        })
    }

    /// Gets an object in the scene. Note that objects in Tile Maps can have overlapping IDs...
    pub fn get_scene_object(&self, id: ID) -> Option<&Object> {
        self.layers.iter_breadth().find_map(|l| if let LayerType::ObjectLayer(os) = &l.content {
            os.iter().find(|o| o.id == id)
        } else {None})
    }
}
