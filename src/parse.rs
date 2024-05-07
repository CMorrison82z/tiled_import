use std::{
    borrow::Borrow, collections::HashMap, fmt::Debug, hash::Hash, path::PathBuf, str::FromStr,
};

use ndarray::Array2;
use nom_xml::{
    types::{Tag, Xml},
    *,
};

use crate::{data_types::*, util::parse_tiles_csv};

pub fn parse<'a>(i: &'a str) -> Result<TiledMap, ()> {
    let tmx_root = Xml::from_input_str(i).unwrap();
    let Xml::Element(map_tag, Some(elements)) = &tmx_root else {
        panic!("oh shit")
    };

    let tile_sets = get_tile_sets(&elements);

    Ok(TiledMap {
        grid_size: (
            get_parse::<u32>(&map_tag.attributes, "width").unwrap(),
            get_parse::<u32>(&map_tag.attributes, "height").unwrap(),
        ),
        tile_size: (
            get_parse::<u32>(&map_tag.attributes, "tilewidth").unwrap(),
            get_parse::<u32>(&map_tag.attributes, "tileheight").unwrap(),
        ),
        layers: parse_layers(&tile_sets, &tmx_root).unwrap(),
        tile_sets,
    })
}

fn get_tile_sets(elements: &Vec<Xml>) -> Vec<TileSet> {
    elements
        .iter()
        .filter_map(|x| tile_set_element(&x))
        .collect()
}

fn tile_set_element(x: &Xml) -> Option<TileSet> {
    let Xml::Element(t, Some(e)) = x else {
        return None;
    };

    if t.value != "tileset" {
        return None;
    }

    let first_gid = get_parse::<u32>(&t.attributes, "firstgid").unwrap();

    let tile_size = (
            get_parse::<u32>(&t.attributes, "tilewidth").unwrap(),
            get_parse::<u32>(&t.attributes, "tileheight").unwrap(),
        );

    Some(TileSet {
        tile_size,
        first_gid,
        name: t.attributes.get("name").unwrap().clone(),
        margin: get_parse::<u8>(&t.attributes, "margin").unwrap_or(0),
        spacing: get_parse::<u8>(&t.attributes, "spacing").unwrap_or(0),
        images: e
            .iter()
            .filter(|x| x.tag_has_name("image"))
            .map(|xml_element| match xml_element {
                Xml::Element(img_tag, _) => Image {
                    source: img_tag.attributes.get("source").unwrap().into(),
                    dimensions: (get_parse::<u32>(&img_tag.attributes, "width").unwrap() / tile_size.0, get_parse::<u32>(&img_tag.attributes, "height").unwrap() / tile_size.1),
                    format: img_tag.attributes.get("format").unwrap_or(&"png".into()).clone(),
                },
                _ => unreachable!(), // This will panic if Xml::Element is not matched
            })
            .collect(),
        tiles: (0..(get_parse::<u32>(&t.attributes, "tilecount").unwrap())).map(|i| Tile(first_gid + i)).collect()
        // TODO:
        // Individual time-elements are only if the tilset is based off of multiple images...
        // Otherwise, we need to iterate and give everything and ID ourselves.
        // tiles: e
        //     .iter()
        //     .filter(|x| x.tag_has_name("tile"))
        //     .map(|xml_element| match xml_element {
        //         Xml::Element(tile_tag, tile_elems) => Tile {
        //             local_id: get_parse(&tile_tag.attributes, "id").unwrap(),
        //             sub_rect_size: (
        //                 get_parse::<u32>(&t.attributes, "width").unwrap(),
        //                 get_parse::<u32>(&t.attributes, "height").unwrap(),
        //             ),
        //             sub_rect_position: (
        //                 get_parse::<u32>(&t.attributes, "x").unwrap(),
        //                 get_parse::<u32>(&t.attributes, "y").unwrap(),
        //             ),
        //             properties: tile_elems
        //                 .as_ref()
        //                 .map(|v| parse_tmx_properties(&v))
        //                 .flatten(),
        //         },
        //         _ => unreachable!(),
        //     })
        //     .collect(),
    })
}

fn parse_tmx_properties(x: &Vec<Xml>) -> Option<Properties> {
    x.iter()
        .find(|n_x| n_x.tag_has_name("properties"))
        .map(|xml_element| match xml_element {
            Xml::Element(_, Some(props)) => {
                props.iter().map(parse_tmx_property).collect::<Properties>()
            }
            _ => unreachable!(), // This will panic if Xml::Element is not matched
        })
}

fn parse_tmx_property(x: &Xml) -> (String, TiledPropertyType) {
    let Xml::Element(t, _) = x else { panic!() };

    let v = t.attributes.get("value").unwrap().clone();

    (
        t.attributes.get("name").unwrap().clone(),
        match t.attributes.get("type").unwrap().as_str() {
            "string" => TiledPropertyType::String(v),
            "int" => TiledPropertyType::Int(v.parse().unwrap()),
            "float" => TiledPropertyType::Float(v.parse().unwrap()),
            "bool" => TiledPropertyType::Bool(v.parse().unwrap()),
            "file" => TiledPropertyType::File(v.parse().unwrap()),
            "object" => TiledPropertyType::Object(v.parse().unwrap()),
            _ => unreachable!(),
        },
    )
}

// TODO:
// Return a flattened Result
fn get_parse<T>(hm: &HashMap<String, String>, field: &str) -> Option<T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    hm.get(field).map(|v| v.parse::<T>().ok()).flatten()
}

fn parse_layers(v: &Vec<TileSet>, x: &Xml) -> Option<LayerHierarchy> {
    match x {
        Xml::Element(t, Some(c)) => match t.value.as_str() {
            "group" => Some(LayerHierarchy::Node(
                TiledLayer::Group(parse_layer(t)),
                c.iter().filter_map(|n_x| parse_layers(v, n_x)).collect(),
            )),
            "map" => Some(LayerHierarchy::Node(
                TiledLayer::Group(Layer {
                    id: 0,
                    name: "base".into(),
                    visible: true,
                    opacity: 1.,
                    parallax: (0., 0.),
                    repeatx: false,
                    repeaty: false,
                }),
                c.iter().filter_map(|n_x| parse_layers(v, n_x)).collect(),
            )),
            // } else {
            //     LayerHierarchy::Layer(TiledLayer::Group(parse_layer(t)))
            // }),
            "objectgroup" => Some(LayerHierarchy::Leaf(TiledLayer::Object(
                parse_layer(t),
                c.iter().filter_map(object_parse).collect(),
            ))),
            "layer" => Some(LayerHierarchy::Leaf(TiledLayer::Tile(
                parse_layer(t),
                grid_parse(v, c.iter().find(|x| if let Xml::Element(t, _) = x {t.value == "data"} else {false}).unwrap())
            ))),
            "imagelayer" => Some(LayerHierarchy::Leaf(TiledLayer::Image(
                parse_layer(t),
                todo!(),
            ))),
            _ => None,
        },
        _ => None,
    }
}

fn grid_parse(v: &Vec<TileSet>, x: &Xml) -> Array2<Option<LayerTile>> {
    let Xml::Element(t, Some(c)) = x else {panic!()};

    let Some(Xml::Text(s)) = c.iter().find(|n_x| !n_x.is_element()) else {panic!("Only csv is supported")};

    // TODO:
    // Parse text into vec<gid>

    parse_tiles_csv(s.as_str()).unwrap().map(|gid| parse_tile_from_gid(v, gid))
}

// NOTE:
// Maybe use later to support xml elements, but probably not...
fn parse_tile(tilesets: &Vec<TileSet>, x: &Xml) -> Option<LayerTile> {
    let Xml::Element(t, _) = x else {return None};

    let bits: u32 = t.attributes.get("gid").unwrap().parse().unwrap();

    parse_tile_from_gid(tilesets, &bits)
}

fn parse_tile_from_gid(tilesets: &Vec<TileSet>, bits: &u32) -> Option<LayerTile> {
    let flags = bits & ALL_FLIP_FLAGS;

    let gid = Gid(bits & !ALL_FLIP_FLAGS);
    let flip_d = flags & FLIPPED_DIAGONALLY_FLAG == FLIPPED_DIAGONALLY_FLAG; // Swap x and y axis (anti-diagonally) [flips over y = -x line]
    let flip_h = flags & FLIPPED_HORIZONTALLY_FLAG == FLIPPED_HORIZONTALLY_FLAG; // Flip tile over y axis
    let flip_v = flags & FLIPPED_VERTICALLY_FLAG == FLIPPED_VERTICALLY_FLAG; // Flip tile over x axis

    if gid == Gid::EMPTY {
        None
    } else {
        let tileset = crate::util::get_tileset_for_gid(tilesets, gid)?;
        let id = gid.0 - tileset.first_gid;

        let tile = (tileset.tiles.iter().find(|Tile(t_id)| *t_id == id)?).clone();

        Some(LayerTile {
            tile,
            flip_h,
            flip_v,
            flip_d,
        })
    }
}

fn object_parse(x: &Xml) -> Option<Object> {
    let Xml::Element(t, _) = x else { return None };

    if t.value != "object" {
        return None;
    };

    Some(Object {
        id: get_parse(&t.attributes, "id").unwrap(),
        // tile_type: get_parse(&t.attributes, "id").unwrap(),
        position: (
            get_parse::<u32>(&t.attributes, "x").unwrap(),
            get_parse::<u32>(&t.attributes, "y").unwrap(),
        ),
        size: (
            get_parse::<u32>(&t.attributes, "width").unwrap(),
            get_parse::<u32>(&t.attributes, "height").unwrap(),
        ),
        rotation: get_parse(&t.attributes, "rotation").unwrap(),
        tile_global_id: get_parse(&t.attributes, "gid").unwrap(),
        visible: (get_parse::<u8>(&t.attributes, "visible").unwrap() == 1),
    })
}

// TODO:
// Include `properties`
fn parse_layer(t: &Tag) -> Layer {
    Layer {
        id: get_parse(&t.attributes, "id").unwrap(),
        name: t.attributes.get("name").unwrap().clone(),
        visible: (get_parse::<u8>(&t.attributes, "visible").unwrap_or(1) == 1),
        opacity: get_parse(&t.attributes, "opacity").unwrap_or(1.),
        parallax: (
            get_parse(&t.attributes, "parallaxx").unwrap_or(1.),
            get_parse(&t.attributes, "parallaxy").unwrap_or(1.),
        ),
        // TODO:
        // This is probably actually a `1` or `0`, like "visible"
        repeatx: get_parse(&t.attributes, "repeatx").unwrap_or(false),
        repeaty: get_parse(&t.attributes, "repeaty").unwrap_or(false),
    }
}
