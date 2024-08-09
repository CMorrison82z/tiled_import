use std::{collections::HashMap, path::PathBuf};

use ndarray::Array2;
use tree::Tree;

pub type ID = u32;

pub type PairU32 = (u32, u32);
pub type PairF32 = (f32, f32);

pub type Properties = HashMap<String, TiledPropertyType>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Gid(pub u32);

impl Gid {
    /// The GID representing an empty tile in the map.
    #[allow(dead_code)]
    pub const EMPTY: Gid = Gid(0);
}

pub const FLIPPED_HORIZONTALLY_FLAG: u32 = 0x80000000;
pub const FLIPPED_VERTICALLY_FLAG: u32 = 0x40000000;
pub const FLIPPED_DIAGONALLY_FLAG: u32 = 0x20000000;
pub const ALL_FLIP_FLAGS: u32 =
    FLIPPED_HORIZONTALLY_FLAG | FLIPPED_VERTICALLY_FLAG | FLIPPED_DIAGONALLY_FLAG;

#[derive(Clone, Debug)]
pub enum TiledPropertyType {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    // TODO:
    // Hex with alpha channel : #AARRGGBB)
    // Color(Vec<),
    File(PathBuf),
    // Object properties can reference any object on the same map and are stored as an integer (the ID of the referenced object,
    // or 0 when no object is referenced). When used on objects in the Tile Collision Editor, they can only refer to other objects on the same tile.
    Object(ID),
    // Class(???)
}

#[derive(Debug, Clone)]
pub enum ObjectType {
    Ellipse, // The existing x, y, width and height attributes are used to determine the size of the ellipse.
    Point,   // The existing x and y attributes are used to determine the position of the point.
    Polygon(Vec<PairF32>), // The origin for these coordinates is the location of the parent
    Polyline(Vec<PairF32>),
}

#[derive(Debug)]
pub struct Color {
    pub alpha: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug)]
pub enum Shape {
    Ellipse(PairU32),       // Pair represents the size
    Point(PairU32),         // Pair represents the position
    Polygon(Vec<PairU32>),  // Pair represents the position
    Polyline(Vec<PairU32>), // Pair represents the position
}

// pub struct Text {
//     fontfamily: String,
//     pixel_size: u32,
//     wrap: bool,
//     color:
// }

#[derive(Clone, Copy, Debug)]
pub struct Tile(pub ID);
// pub struct Tile {
//     // pub global_id: ID,
//     pub local_id: ID,
//     // pub tile_type: String,
//     pub sub_rect_position: PairU32,
//     pub sub_rect_size: PairU32,
//     pub properties: Option<Properties>
// }

#[derive(Clone, Copy, Debug)]
pub struct LayerTile {
    pub tile: Tile,
    pub flip_h: bool,
    pub flip_v: bool,
    pub flip_d: bool,
}

#[derive(Clone, Debug)]
pub struct Object {
    pub id: ID,
    // pub tile_type: String,
    pub position: PairU32,
    pub size: PairU32,
    pub rotation: f32,
    pub tile_global_id: ID,
    pub visible: bool,
    otype: ObjectType,
    properties: Properties,
}

#[derive(Clone, Debug)]
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
    pub parallax: (f32, f32),
    pub repeatx: bool,
    pub repeaty: bool,
}

#[derive(Clone, Debug)]
pub enum TiledLayer {
    Tile(Layer, Array2<Option<LayerTile>>),
    Object(Layer, Vec<Object>),
    Image(Layer, Image),
    Group(Layer),
}

#[derive(Clone, Debug)]
pub struct Image {
    pub source: PathBuf,
    pub dimensions: PairU32,
    pub format: String,
    // color: Color
}

#[derive(Debug)]
pub struct TileAuxInfo {
    // color: Color
    // Can contain at most one: <properties>, <image> (since 0.9), <objectgroup>, <animation>
    // animation: ObjectGroup,
    properties: Properties,
    objectgroup: Vec<Object>,
}

#[derive(Debug)]
pub struct TileSet {
    pub tile_size: PairU32,
    pub first_gid: ID,
    pub name: String,
    pub spacing: u8,
    pub margin: u8,
    // NOTE:
    // Removed for now because it's better to rely on `first_gid`
    // tile_count: u32,
    pub images: Vec<Image>,
    // I think this still needs to exist because some IDs may disappear when updating the
    // underlying tileset.
    // TODO:
    // Do we want it to be HashSet? Vecs are more space efficient.
    pub tiles: Vec<Tile>,
    pub tile_stuff: HashMap<Tile, TileAuxInfo>,
}

pub type LayerHierarchy = Tree<TiledLayer>;

#[derive(Debug)]
pub struct TiledMap {
    pub layers: LayerHierarchy,
    // Measured in tiles
    pub grid_size: PairU32,
    pub tile_size: PairU32,
    pub tile_sets: Vec<TileSet>,
}
