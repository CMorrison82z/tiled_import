use std::{collections::HashMap, path::PathBuf};

pub type ID = u32;

pub type PairU32 = (u32, u32);

pub type Properties = HashMap<String, TiledPropertyType>;

pub enum TiledPropertyType {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    // TODO:
    // Hex with alpha channel : #AARRGGBB)
    // Color(Vec<),
    File(PathBuf),
    Object(ID),
    // Class(???)
}

pub struct Color {
    pub alpha: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

pub enum Shape {
    Ellipse(PairU32), // Pair represents the size
    Point(PairU32), // Pair represents the position
    Polygon(Vec<PairU32>), // Pair represents the position
    Polyline(Vec<PairU32>), // Pair represents the position
}

// pub struct Text {
//     fontfamily: String,
//     pixel_size: u32,
//     wrap: bool,
//     color: 
// }

pub struct Tile {
    // pub global_id: ID,
    pub local_id: ID,
    // pub tile_type: String,
    pub sub_rect_position: PairU32,
    pub sub_rect_size: PairU32,
    pub properties: Option<Properties>
}

pub struct Object {
    pub id: ID,
    pub tile_type: String,
    pub position: PairU32,
    pub size: PairU32,
    pub rotation: f32,
    pub tile_global_id: ID,
    pub visible: bool
}

pub struct Layer {
    pub id: ID,
    pub name: String,
    // pub class: String,
    // Cannot be modified in Tiled
    // _pos: PairU32
    // Always same as Map size
    // _size
    pub visible: bool,
    pub opacity: f32,
    pub parallax: PairU32,
    pub repeatx: bool,
    pub repeaty: bool
}

pub enum TiledLayer {
    Tile(Layer, Vec<Tile>),
    Object(Layer, Vec<Object>),
    Image(Layer, Image),
    Group(Layer)
}

pub struct Image {
    pub source: PathBuf,
    // pub size: PairU32,
    pub format: String,
    // color: Color
}

pub struct TileSet {
    pub tile_size: PairU32,
    pub first_gid: ID,
    pub name: String,
    pub spacing: u8,
    pub margin: u8,
    // NOTE:
    // Removed for now because it's better to rely on `first_gid`
    // tile_count: u32,
    pub images_bytes: Vec<Image>,
    pub tiles: Vec<Tile>
}

pub enum LayerHierarchy {
    Layer(TiledLayer),
    Children(TiledLayer, Vec<LayerHierarchy>)
}

pub struct TiledMap {
    pub layers: LayerHierarchy,
    // Measured in tiles
    pub grid_size: PairU32,
    pub tile_size: PairU32,
    pub tile_sets: Vec<TileSet>,
}
