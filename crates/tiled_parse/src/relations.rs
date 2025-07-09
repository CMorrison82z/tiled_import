
use try_match::match_ok;

use crate::types::*;

pub fn get_tileset_for_gid(tilesets: &[TileSet], Gid(gid): Gid) -> Option<&TileSet> {
    tilesets
        .iter()
        .filter(|ts| ts.first_gid <= gid)
        .max_by_key(|ts| ts.first_gid)
}

pub fn get_tile_id(TileSet { first_gid, .. }: &TileSet, Gid(gid): Gid) -> u32 {
    gid - first_gid
}

pub fn tile_set_rows_and_columns(TileSet { image: Image {
        source,
        dimensions: (width, height),
    }, tile_size: (tile_width, tile_height), margin, spacing, .. }: &TileSet) -> (u32, u32) {
    (
        (width - (*margin as u32)) / (tile_width + (*spacing as u32)),
        (height - (*margin as u32)) / (tile_height + (*spacing as u32)),
    )
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

    /// For getting the inherited properties of an `ObjectType::Tile` from the Tile it points to.
    /// If the object is NOT an `ObjectType::Tile`, simply iterates its own properties.
    pub fn iter_object_tile_properties<'a>(&'a self, object: &'a Object) -> impl Iterator<Item = (&'a String, &'a TiledPropertyType)> {
        object
            .properties
            .iter()
            .chain({
                match_ok!(object.otype, ObjectType::Tile(t_gid))
                    .and_then(|t_gid| self.get_tile_properties(t_gid))
                    .into_iter()
                    .flat_map(|hm| hm.iter())
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
