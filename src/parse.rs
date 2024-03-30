use std::{
    borrow::Borrow, collections::HashMap, fmt::Debug, hash::Hash, path::PathBuf, str::FromStr,
};

use nom_xml::{types::{Tag, Xml}, *};

use crate::data_types::*;

pub fn parse<'a>(i: &'a str) -> Result<TiledMap, ()> {
    let tmx_root = Xml::from_input_str(i).unwrap();
    let Xml::Element(map_tag, Some(elements)) = &tmx_root else {
        panic!()
    };

    Ok(TiledMap {
        grid_size: (
            get_parse::<u32>(&map_tag.attributes, "width").unwrap(),
            get_parse::<u32>(&map_tag.attributes, "height").unwrap(),
        ),
        tile_size: (
            get_parse::<u32>(&map_tag.attributes, "tilewidth").unwrap(),
            get_parse::<u32>(&map_tag.attributes, "tileheight").unwrap(),
        ),
        tile_sets: get_tile_sets(&elements),
        layers: parse_layers(&tmx_root).unwrap(),
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

    Some(TileSet {
        tile_size: (
            get_parse::<u32>(&t.attributes, "tilewidth").unwrap(),
            get_parse::<u32>(&t.attributes, "tileheight").unwrap(),
        ),
        first_gid: get_parse::<u32>(&t.attributes, "firstgid").unwrap(),
        name: t.attributes.get("name").unwrap().clone(),
        margin: get_parse::<u8>(&t.attributes, "margin").unwrap(),
        spacing: get_parse::<u8>(&t.attributes, "margin").unwrap(),
        images_bytes: e
            .iter()
            .filter(|x| x.tag_has_name("image"))
            .map(|xml_element| match xml_element {
                Xml::Element(img_tag, _) => Image {
                    source: img_tag.attributes.get("source").unwrap().into(),
                    // size: (get_parse::<u32>(img_tag.attributes, "width").unwrap(), get_parse::<u32>(img_tag.attributes, "height").unwrap()),
                    format: img_tag.attributes.get("format").unwrap().clone(),
                },
                _ => unreachable!(), // This will panic if Xml::Element is not matched
            })
            .collect(),
        tiles: e
            .iter()
            .filter(|x| x.tag_has_name("tile"))
            .map(|xml_element| match xml_element {
                Xml::Element(tile_tag, tile_elems) => Tile {
                    local_id: get_parse(&tile_tag.attributes, "id").unwrap(),
                    sub_rect_size: (
                        get_parse::<u32>(&t.attributes, "width").unwrap(),
                        get_parse::<u32>(&t.attributes, "height").unwrap(),
                    ),
                    sub_rect_position: (
                        get_parse::<u32>(&t.attributes, "x").unwrap(),
                        get_parse::<u32>(&t.attributes, "y").unwrap(),
                    ),
                    properties: tile_elems
                        .as_ref()
                        .map(|v| parse_tmx_properties(&v))
                        .flatten(),
                },
                _ => unreachable!(),
            })
            .collect(),
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

fn parse_layers(x: &Xml) -> Option<LayerHierarchy> {
    match x {
        Xml::Element(t, Some(c)) => match t.value.as_str() {
            "group" => Some(
                LayerHierarchy::Children(
                    TiledLayer::Group(parse_layer(t)),
                    c.iter().filter_map(parse_layers).collect(),
                )),
            // } else {
            //     LayerHierarchy::Layer(TiledLayer::Group(parse_layer(t)))
            // }),
            "objectgroup" => Some(LayerHierarchy::Layer(TiledLayer::Object(parse_layer(t), c.iter().filter_map(|x| {})))),
            "layer" => Some(LayerHierarchy::Layer(TiledLayer::Tile(parse_layer(t), todo!()))),
            "imagelayer" => Some(LayerHierarchy::Layer(TiledLayer::Image(parse_layer(t), todo!()))),
            _ => None
        },
        _ => None,
    }
}

fn parse_layer(t: &Tag) -> Layer {
    Layer {
        id: get_parse(&t.attributes, "id").unwrap(),
        name: t.attributes.get("name").unwrap().clone(),
        visible: get_parse(&t.attributes, "visible").unwrap(),
        opacity: get_parse(&t.attributes, "opacity").unwrap(),
        parallax: (
            get_parse(&t.attributes, "parallaxx").unwrap(),
            get_parse(&t.attributes, "parallaxy").unwrap(),
        ),
        repeatx: get_parse(&t.attributes, "repeatx").unwrap(),
        repeaty: get_parse(&t.attributes, "repeaty").unwrap(),
    }
}
