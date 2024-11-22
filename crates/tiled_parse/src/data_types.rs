use std::{collections::HashMap, path::PathBuf, str::FromStr};

use ndarray::Array2;
use tree::Tree;

pub type ID = u32;

pub type PairU32 = (u32, u32);
pub type PairF32 = (f32, f32);

pub type Properties = HashMap<String, TiledPropertyType>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Gid(pub u32);

impl FromStr for Gid {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u32>().map(|v| Gid(v))
    }
}

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

pub const BASE_LAYER: &str = "map";
pub const GROUP_LAYER: &str = "group";
pub const OBJECTGROUP_LAYER: &str = "objectgroup";
pub const TILE_LAYER: &str = "layer";
pub const IMAGE_LAYER: &str = "imagelayer";

/// Properties in Tiled Objects.
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

/// Geometries for Tiled Objects.
#[derive(Debug, Clone)]
pub enum ObjectType {
    Rectangle,
    /// The existing x, y, width and height attributes are used to determine the size.
    Ellipse,
    /// The existing x, y, width and height attributes are used to determine the size of the ellipse.
    Point,
    /// The existing x and y attributes are used to determine the position of the point.
    Polygon(Vec<PairF32>),
    /// The origin for these coordinates is the location of the parent
    Polyline(Vec<PairF32>),
}

// FIXME:
// Dead code.
#[derive(Debug)]
struct Color {
    pub alpha: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

// pub struct Text {
//     fontfamily: String,
//     pixel_size: u32,
//     wrap: bool,
//     color:
// }

// pub struct Tile {
//     // pub global_id: ID,
//     pub local_id: ID,
//     // pub tile_type: String,
//     pub sub_rect_position: PairU32,
//     pub sub_rect_size: PairU32,
//     pub properties: Option<Properties>
// }

/// A tile placed on a Tile Layer.
#[derive(Clone, Copy, Debug)]
pub struct LayerTile {
    pub tile: Gid,
    pub flip_h: bool,
    pub flip_v: bool,
    pub flip_d: bool,
}

/// Tiled Object.
#[derive(Clone, Debug)]
pub struct Object {
    pub id: ID,
    // pub tile_type: String,
    pub position: PairF32,
    pub size: Option<PairF32>,
    pub rotation: f32,
    // If the object is attached to a Tile, this field will exist.
    pub tile_global_id: Option<Gid>,
    pub visible: bool,
    pub otype: ObjectType,
    pub properties: Properties,
}

/// Layer content parameterized by an `enum` containing the data of each respective layer type.
#[derive(Clone, Debug)]
pub enum LayerType {
    TileLayer(Array2<Option<LayerTile>>),
    ObjectLayer(Vec<Object>),
    ImageLayer(ImageStuff),
    /// A group later has no content, just serves as a structural guide.
    Group,
}

/// A layer in a Tiled map.
#[derive(Clone, Debug)]
pub struct TiledLayer {
    pub id: ID,
    pub name: String,
    // pub class: String,
    // Cannot be modified in Tiled
    // _pos: PairU32
    // Always same as Map size
    // _size
    pub content: LayerType,
    pub visible: bool,
    pub opacity: f32,
    pub parallax: (f32, f32),
    pub properties: Properties,
}

// TODO:
// Better name :)
#[derive(Clone, Debug)]
pub struct ImageStuff {
    pub repeatx: bool,
    pub repeaty: bool,
    pub image: Image,
}

#[derive(Clone, Debug)]
pub struct Image {
    pub source: PathBuf,
    pub dimensions: PairU32,
    pub format: String,
    // pub color: Color
}

/// Auxillary information about a tile
#[derive(Debug)]
pub struct TileAuxInfo {
    // Can contain at most one: <properties>, <image> (since 0.9), <objectgroup>, <animation>
    // pub color: Color,
    // pub animation: ObjectGroup,
    pub properties: Properties,
    /// NOTE:
    /// This departure from Tiled's file specification. I don't like the idea of encoding the objects as an entire layer.
    pub objects: Vec<Object>,
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
    //
    /// Only supports a single Texture Atlas. Undecided on how to represent multiple images
    pub image: Image,
    /// This u32 is the LOCAL id of the tile (relative to this tileset)
    pub tile_stuff: HashMap<u32, TileAuxInfo>,
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
