use std::{borrow::Borrow, collections::HashMap, fmt::Debug, hash::Hash, path::PathBuf, str::FromStr};

use nom_xml::{types::Xml, *};

use crate::data_types::*;

pub fn parse<'a>(
    i: &'a str,
) -> Result<TiledMap, ()> {
    let Xml::Element(map_tag, elements) = Xml::from_input_str(i).unwrap();

    Ok(TiledMap {
        grid_size: (map_tag.attributes.get("width").unwrap(), map_tag.attributes.get("height").unwrap()),
        tile_size: (map_tag.attributes.get("tilewidth").unwrap(), map_tag.attributes.get("tileheight").unwrap())
        tile_sets: get_tile_sets(elements),
    })
}

fn get_tile_sets(elements: Vec<Xml>) -> Vec<TileSet> {
    elements.iter().filter_map(|x| {

    });
}

fn tile_set_element(x: Xml) -> Option<TileSet> {
    let Xml::Element(t, Some(e)) = x else {return None};

    if t.value != "tileset" {
        return None
    }

    Some(TileSet {
        tile_size: (get_parse::<u32>(t.attributes, "tilewidth").unwrap(), get_parse::<u32>(t.attributes, "tileheight").unwrap()),
        first_gid: get_parse::<u32>(t.attributes, "firstgid").unwrap(),
        name: t.attributes.get("name").unwrap().clone(),
        margin: get_parse::<u8>(t.attributes, "margin").unwrap(),
        spacing: get_parse::<u8>(t.attributes, "margin").unwrap(),
        images_bytes: e.iter().filter(|x| x.tag_has_name("image")).map(|Xml::Element(img_tag, _)| Image {
            source: img_tag.attributes.get("source").unwrap().into(),
            // size: (get_parse::<u32>(img_tag.attributes, "width").unwrap(), get_parse::<u32>(img_tag.attributes, "height").unwrap()),
            format: img_tag.attributes.get("format").unwrap().clone()
        }).collect(),
        tiles: e.iter().filter(|x| x.tag_has_name("tile")).map(|Xml::Element(tile_tag, tile_elems)| Tile {
            local_id: get_parse(tile_tag.attributes, "id").unwrap(),
            sub_rect_size: (get_parse::<u32>(t.attributes, "width").unwrap(), get_parse::<u32>(t.attributes, "height").unwrap()),
            sub_rect_position: (get_parse::<u32>(t.attributes, "x").unwrap(), get_parse::<u32>(t.attributes, "y").unwrap()),
            properties: tile_elems.map(|v| parse_tmx_properties(&v)).flatten()
        }).collect()
    })
}

fn parse_tmx_properties(x: &Vec<Xml>) -> Option<Properties> {
    x.iter().find(|n_x| n_x.tag_has_name("properties")).map(|Xml::Element(_, Some(props))| props.iter().map(parse_tmx_property).collect::<Properties>())
}

fn parse_tmx_property(x: &Xml) -> (String, TiledPropertyType) {
    let Xml::Element(t, _) = x;

    let v = t.attributes.get("value").unwrap().clone();

    (t.attributes.get("name").unwrap().clone(), match t.attributes.get("type").unwrap().as_str() {
        "string" => TiledPropertyType::String(v),
        "int" => TiledPropertyType::Int(v.parse().unwrap()),
        "float" => TiledPropertyType::Float(v.parse().unwrap()),
        "bool" => TiledPropertyType::Bool(v.parse().unwrap()),
        "file" => TiledPropertyType::File(v.parse().unwrap()),
        "object" => TiledPropertyType::Object(v.parse().unwrap()),
        _ => unreachable!()
    })
}

// TODO:
// Return a flattened Result
fn get_parse<T>(hm: HashMap<String, String>, field: &str) -> Option<T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug
{
    hm.get(field).map(|v| v.parse::<T>().ok()).flatten()
}

fn xml_element_filter(x: Xml, name: Option<&str>) -> Option<Xml> {
    match (x, name) {
        (Xml::Element(t, _), Some(n)) if t.value == n => Some(x),
        (Xml::Element(_, _), None) => Some(x),
        _ => None,
    }
}
