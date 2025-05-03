// FIXME:
// Remove various intermediate `collect` with Iterators.

use std::{collections::HashMap, fmt::Debug, future::Future, pin::Pin, str::FromStr};

use futures::future::join_all;
use ndarray::Array2;
use try_match::match_ok;
use xml_nom_parse::types::{Tag, Xml};

use crate::{
    types::*,
    util::{parse_spaced_f32_pairs, parse_tiles_csv},
};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

// trait FileLoader<'a>: Fn(&'a str) -> BoxFuture<'a, &'a str> {}
// impl<'a, T: Fn(&'a str) -> BoxFuture<'a, &'a str>> FileLoader<'a> for T {}

pub async fn parse<'a, 'b, F>(i: &'a str, l: &mut F) -> Result<TiledMap, ()>
where
    F: FnMut(String) -> BoxFuture<'b, Result<String, ()>>,
{
    let tmx_root = Xml::from_input_str(i).unwrap();
    let Xml::Element(map_tag, Some(elements)) = &tmx_root else {
        eprintln!("Room should be an element.");
        return Err(());
    };

    // TODO:
    // Make this less cursed. Be careful of borrow checker.
    let mut futuresss: Vec<Pin<Box<dyn Future<Output = Result<String, ()>> + Send>>> = vec![];
    let mut ids: Vec<ID> = vec![];
    let mut tile_sets = vec![];

    for x in get_tile_sets(&elements).into_iter() {
        match x {
            TiledMapTileSet::Embedded(y) => tile_sets.push(y),
            TiledMapTileSet::External { first_gid, source } => {
                futuresss.push(l(source));
                ids.push(first_gid);
            }
        }
    }

    let tile_sets2 = join_all(futuresss)
        .await
        .into_iter()
        .zip(ids.into_iter())
        .map(|(r, first_gid)| {
            parse_tile_set(first_gid, &Xml::from_input_str(&r.unwrap()).unwrap()).unwrap()
        });

    tile_sets.extend(tile_sets2.into_iter());

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

/// Specifically parses tiled maps with its tilesets and such embedded in the file itself.
pub fn parse_embedded<'a>(i: &'a str) -> Result<TiledMap, ()> {
    let tmx_root = Xml::from_input_str(i).unwrap();
    let Xml::Element(map_tag, Some(elements)) = &tmx_root else {
        panic!("oh shit")
    };

    let tile_sets = get_tile_sets(&elements)
        .iter()
        .filter_map(|x| match_ok!(x, TiledMapTileSet::Embedded(y)).cloned())
        .collect();

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

fn get_tile_sets(elements: &Vec<Xml>) -> Vec<TiledMapTileSet> {
    elements
        .iter()
        .filter_map(|x| tile_set_element(&x))
        .collect()
}

fn tile_set_element(x: &Xml) -> Option<TiledMapTileSet> {
    let Xml::Element(t, _) = x else {
        return None;
    };

    if t.value != "tileset" {
        return None;
    }

    let first_gid = get_parse::<u32>(&t.attributes, "firstgid").unwrap();

    if let Some(src) = get_parse::<String>(&t.attributes, "source") {
        Some(TiledMapTileSet::External {
            first_gid,
            source: src,
        })
    } else {
        parse_tile_set(first_gid, x).map(|x| TiledMapTileSet::Embedded(x))
    }
}

fn parse_tile_set(first_gid: ID, x: &Xml) -> Option<TileSet> {
    let Xml::Element(t, Some(e)) = x else {
        return None;
    };

    let tile_size = (
        get_parse::<u32>(&t.attributes, "tilewidth").unwrap(),
        get_parse::<u32>(&t.attributes, "tileheight").unwrap(),
    );

    let margin = get_parse::<u8>(&t.attributes, "margin").unwrap_or(0);
    let spacing = get_parse::<u8>(&t.attributes, "spacing").unwrap_or(0);

    Some(TileSet {
        tile_size,
        first_gid,
        name: t.attributes.get("name").unwrap().clone(),
        margin,
        spacing,
        image: e
            .iter()
            .find(|x| x.tag_has_name("image"))
            .map(|xml_element| match xml_element {
                Xml::Element(img_tag, _) => Image {
                    source: img_tag.attributes.get("source").unwrap().into(),
                    dimensions: {
                        (
                            (get_parse::<u32>(&img_tag.attributes, "width").unwrap()
                                - (margin as u32))
                                / (tile_size.0 + (spacing as u32)),
                            (get_parse::<u32>(&img_tag.attributes, "height").unwrap()
                                - (margin as u32))
                                / (tile_size.1 + (spacing as u32)),
                        )
                    },
                    format: img_tag
                        .attributes
                        .get("format")
                        .unwrap_or(&"png".into())
                        .clone(),
                },
                _ => unreachable!(), // This will panic if Xml::Element is not matched
            })
            .expect("Tile set should contain an image."),
        tile_stuff: e
            .iter()
            .filter_map(|x| {
                if !x.tag_has_name("tile") {
                    return None;
                };
                let Xml::Element(tile_tag, Some(tile_elems)) = x else {
                    return None;
                };
                let id = get_parse::<u32>(&tile_tag.attributes, "id")?;
                let properties = parse_tmx_properties(x).unwrap_or_default();
                let objects = tile_elems
                    .iter()
                    .find(|t_e| t_e.tag_has_name("objectgroup"))
                    .and_then(|ogroup_xml| match ogroup_xml {
                        // NOTE:
                        // It's necessary to wrap in a new Option like this because then `objects`
                        // is a reference.
                        Xml::Element(_, Some(objects)) => Some(objects),
                        _ => None,
                    })
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(object_parse)
                    .collect();

                Some((
                    id,
                    TileAuxInfo {
                        properties,
                        objects,
                    },
                ))
            })
            .collect(),
    })
}

fn parse_tmx_properties(x: &Xml) -> Option<Properties> {
    let Xml::Element(_, Some(v)) = x else {
        return None;
    };

    v.iter()
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
            // This is the base layer (top of the layer hierarchy)
            BASE_LAYER => Some(LayerHierarchy::Node(
                TiledLayer {
                    id: 0,
                    name: "base".into(),
                    visible: true,
                    opacity: 1.,
                    parallax: (0., 0.),
                    content: LayerType::Group,
                    properties: parse_tmx_properties(x).unwrap_or_default(),
                },
                c.iter().filter_map(|n_x| parse_layers(v, n_x)).collect(),
            )),
            GROUP_LAYER => Some(LayerHierarchy::Node(
                parse_layer(
                    t,
                    LayerType::Group,
                    parse_tmx_properties(x).unwrap_or_default(),
                ),
                c.iter().filter_map(|n_x| parse_layers(v, n_x)).collect(),
            )),
            // } else {
            //     LayerHierarchy::Layer(TiledLayer::Group(parse_layer(t)))
            // }),
            OBJECTGROUP_LAYER => Some(LayerHierarchy::Leaf(parse_layer(
                t,
                LayerType::ObjectLayer(c.iter().filter_map(object_parse).collect()),
                parse_tmx_properties(x).unwrap_or_default(),
            ))),
            // This is the `tilelayer`
            TILE_LAYER => Some(LayerHierarchy::Leaf(parse_layer(
                t,
                LayerType::TileLayer(grid_parse(
                    v,
                    c.iter()
                        .find(|x| {
                            if let Xml::Element(t, _) = x {
                                t.value == "data"
                            } else {
                                false
                            }
                        })
                        .unwrap(),
                )),
                parse_tmx_properties(x).unwrap_or_default(),
            ))),
            IMAGE_LAYER => Some(LayerHierarchy::Leaf(parse_layer(
                t,
                LayerType::ImageLayer(todo!()),
                parse_tmx_properties(x).unwrap_or_default(),
            ))),
            _ => None,
        },
        _ => None,
    }
}

fn grid_parse(v: &Vec<TileSet>, x: &Xml) -> Array2<Option<LayerTile>> {
    let Xml::Element(t, Some(c)) = x else {
        panic!()
    };

    let Some(Xml::Text(s)) = c.iter().find(|n_x| !n_x.is_element()) else {
        panic!("Only csv is supported")
    };

    // TODO:
    // Parse text into vec<gid>

    parse_tiles_csv(s.as_str())
        .unwrap()
        .map(|gid| parse_tile_from_gid(v, gid))
}

// NOTE:
// Maybe use later to support xml elements, but probably not...
fn parse_tile(tilesets: &Vec<TileSet>, x: &Xml) -> Option<LayerTile> {
    let Xml::Element(t, _) = x else { return None };

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
        Some(LayerTile {
            tile: gid,
            flip_h,
            flip_v,
            flip_d,
        })
    }
}

fn object_parse(x: &Xml) -> Option<Object> {
    let Xml::Element(t, c) = x else { return None };

    if t.value != "object" {
        return None;
    };

    Some(Object {
        id: get_parse(&t.attributes, "id").unwrap(),
        // tile_type: get_parse(&t.attributes, "id").unwrap(),
        position: (
            get_parse::<f32>(&t.attributes, "x").unwrap(),
            get_parse::<f32>(&t.attributes, "y").unwrap(),
        ),
        size: get_parse::<f32>(&t.attributes, "width").and_then(|width| {
            get_parse::<f32>(&t.attributes, "height").map(|height| (width, height))
        }),
        rotation: get_parse(&t.attributes, "rotation").unwrap_or_default(),
        visible: (get_parse::<u8>(&t.attributes, "visible").unwrap_or(1) == 1),
        otype: {
            // If the object has a `gid`, then it is a `Tile` object (https://doc.mapeditor.org/en/stable/manual/objects/#insert-tile)
            if let Some(gid) = get_parse(&t.attributes, "gid") {
                ObjectType::Tile(gid)
            } else {
                // FIXME: This could be nicer.
                match c {
                    Some(v) => v
                        .iter()
                        .find_map(|xml_c| {
                            if let Xml::Element(Tag { value, attributes }, _) = xml_c {
                                match value.as_str() {
                                    "ellipse" => Some(ObjectType::Geometry(GeometryType::Ellipse)),
                                    "point" => Some(ObjectType::Geometry(GeometryType::Point)),
                                    "polygon" => Some(ObjectType::Geometry(GeometryType::Polygon(
                                        parse_spaced_f32_pairs(
                                            attributes
                                                .get("points")
                                                .expect("Polygon should have `points` attribute."),
                                        )
                                        .unwrap(),
                                    ))),
                                    "polyline" => {
                                        Some(ObjectType::Geometry(GeometryType::Polyline(
                                            parse_spaced_f32_pairs(
                                                attributes.get("points").expect(
                                                    "Polyline should have `points` attribute.",
                                                ),
                                            )
                                            .unwrap(),
                                        )))
                                    }
                                    _ => None,
                                }
                            } else {
                                None
                            }
                        })
                        .unwrap_or(ObjectType::Geometry(GeometryType::Rectangle)),
                    None => ObjectType::Geometry(GeometryType::Rectangle),
                    // TODO:
                }
            }
        }, // If there is no object type in the xml, it's a Rectangle
        properties: parse_tmx_properties(&x).unwrap_or_default(),
    })
}

// TODO:
// Include `properties`
fn parse_layer(t: &Tag, content: LayerType, properties: Properties) -> TiledLayer {
    TiledLayer {
        id: get_parse(&t.attributes, "id").unwrap(),
        name: t.attributes.get("name").unwrap().clone(),
        visible: (get_parse::<u8>(&t.attributes, "visible").unwrap_or(1) == 1),
        opacity: get_parse(&t.attributes, "opacity").unwrap_or(1.),
        parallax: (
            get_parse(&t.attributes, "parallaxx").unwrap_or(1.),
            get_parse(&t.attributes, "parallaxy").unwrap_or(1.),
        ),
        content, // TODO:
        // This is probably actually a `1` or `0`, like "visible"
        // repeatx: get_parse(&t.attributes, "repeatx").unwrap_or(false),
        // repeaty: get_parse(&t.attributes, "repeaty").unwrap_or(false),
        properties,
    }
}
