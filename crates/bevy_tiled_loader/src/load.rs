use std::fs::read_to_string;

use bevy::asset::{io::Reader, AssetLoader, AssetPath, AsyncReadExt, AsyncWriteExt};
use bevy::asset::{Asset, Handle, LoadContext};
use bevy::hierarchy::BuildWorldChildren;
use bevy::math::{UVec2, Vec2};
use bevy::prelude::*;
use bevy::prelude::{SpatialBundle, TransformBundle};
use bevy::scene::Scene;
use bevy::sprite::{Anchor, Sprite, SpriteBundle, TextureAtlas, TextureAtlasLayout};
use bevy::transform::components::{GlobalTransform, Transform};
use bevy::utils::hashbrown::HashMap;

#[cfg(feature = "rapier2d_colliders")]
use bevy_rapier2d::prelude::*;
use tiled_parse::relations::{get_tile_id, get_tileset_for_gid};

use crate::types::{TiledMapAsset, TiledMapContainer};
use tiled_parse::data_types::*;
use tiled_parse::parse::*;

/// Allows us to do `AssetServer.load("MY_MAP.tmx")`
pub struct TiledLoader;

pub const MAP_SCENE: &str = "MapScene";

// TODO:
// Improved error
impl AssetLoader for TiledLoader {
    type Asset = TiledMapAsset;
    type Settings = ();
    type Error = std::io::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut data = Vec::new();
            reader.read_to_end(&mut data).await?;

            let data_as_utf8 = std::str::from_utf8(&data).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Could not load TMX map: {e}"),
                )
            })?;

            let tm: TiledMap = parse(data_as_utf8).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, format!("Could not load TMX map"))
            })?;

            load_tmx(load_context, tm)
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["tmx"];
        EXTENSIONS
    }
}

fn load_tmx(load_context: &mut LoadContext, tm: TiledMap) -> Result<TiledMapAsset, std::io::Error> {
    // TODO:
    // Might need some way to get tilemap_texture from a Tile's GID (To get the tile's texture).
    let TiledMap {
        layers,
        grid_size,
        tile_size,
        tile_sets,
    } = &tm;

    // TODO:
    // Review how tile set images are stored into these `Vec`s.
    // Some tilesets have more than one image (check TMX docs to verify what that actually means)
    let mut tilemap_textures = Vec::with_capacity(tile_sets.len());
    let mut tilemap_atlases = Vec::with_capacity(tile_sets.len());

    tile_sets.iter().for_each(|ts| {
        let TileSet {
            tile_size,
            first_gid,
            name,
            spacing,
            margin,
            image,
            tile_stuff,
        } = ts;

        let tiled_parse::data_types::Image {
            source,
            format,
            dimensions: (columns, rows),
        } = image;

        let tmx_dir = load_context
            .path()
            .parent()
            .expect("The asset load context was empty.");
        let tile_path = tmx_dir.join(&source);
        let asset_path = AssetPath::from(tile_path);

        let texture_handle: Handle<bevy::prelude::Image> = load_context.load(asset_path.clone());

        let file_name = source
            .file_name()
            .expect("Should have file name")
            .to_str()
            .expect("Valid utf8");

        // TODO:
        // I don't know if I should use "add_labeled_asset", and if the arguments are
        // conventional
        let texture_atlas: Handle<bevy::prelude::TextureAtlasLayout> = load_context
            .add_labeled_asset(
                file_name.into(),
                TextureAtlasLayout::from_grid(
                    UVec2::new(tile_size.0, tile_size.1),
                    *columns,
                    *rows,
                    // TODO:
                    // I'm not sure this translates correctly
                    Some(*spacing as u32 * UVec2::ONE),
                    Some(*margin as u32 * UVec2::ONE),
                ),
            );

        tilemap_textures.push(texture_handle);
        tilemap_atlases.push(texture_atlas);
    });

    // Load scene
    let scene = {
        let mut scene_load_context = load_context.begin_labeled_asset();
        let mut world = World::default();

        let world_root_id = world.spawn(SpatialBundle::INHERITED_IDENTITY).id();

        let (ghalfsize_x, ghalfsize_y) = (grid_size.0 as f32 / 2., grid_size.1 as f32 / 2.);

        let mut layer_ents = Vec::new();

        layers.iter().for_each(|x: &TiledLayer| match x {
            // TODO:
            // Handle other layer types
            TiledLayer::Tile(Layer { name, content, .. }) => {
                let layer_ent = world
                    .spawn((Name::new(name.clone()), TransformBundle::default()))
                    .id();

                layer_ents.push(layer_ent);

                let mut tile_ents = Vec::new();

                // TODO:
                // Maybe I want a layer container, and Name it ?
                content
                    .indexed_iter()
                    .filter_map(|(p, t)| t.map(|v| (p, v)))
                    .for_each(
                        |(
                            (x, y),
                            LayerTile {
                                tile: Gid(tile_gid),
                                flip_h,
                                flip_v,
                                flip_d,
                            },
                        )| {
                            let (x, y) = (x as f32, y as f32);

                            let tile_tileset = get_tileset_for_gid(tile_sets, Gid(tile_gid))
                                .expect("Tile should belong to tileset");

                            let tile_aux_info_opt = tile_tileset
                                .tile_stuff
                                .get(&get_tile_id(tile_tileset, Gid(tile_gid)));

                            let mut tile_entity = world.spawn((
                                SpriteBundle {
                                    sprite: Sprite {
                                        flip_x: flip_h,
                                        flip_y: flip_v,
                                        anchor: Anchor::TopLeft,
                                        ..Default::default()
                                    },
                                    transform: Transform::from_xyz(
                                        x - ghalfsize_x,
                                        y - ghalfsize_y,
                                        0.,
                                    ),
                                    // TODO:
                                    // Don't just get the `0` item
                                    texture: tilemap_textures.get(0).unwrap().clone_weak(),
                                    ..Default::default()
                                },
                                TextureAtlas {
                                    // TODO:
                                    // Don't just get the `0` item
                                    layout: tilemap_atlases.get(0).unwrap().clone(),
                                    index: tile_gid as usize,
                                },
                            ));

                            if let Some(tile_aux_info) = tile_aux_info_opt {
                                #[cfg(feature = "rapier2d_colliders")]
                                {
                                    add_colliders(&mut tile_entity, &tile_aux_info.objects);
                                }
                            }

                            tile_entity.set_parent(layer_ent);

                            // NOTE:
                            // There is an assumption that it's being loaded for a 2d camera here.
                            tile_ents.push(tile_entity.id());
                        },
                    );
            }
            _ => {
                println!("Layer was not a `Tile` layer. Not currently handled.");
            }
        });

        let mut e_c = world.spawn((
            TiledMapContainer,
            // TODO:
            // There may be some situation where it won't just be 0 ?
            TransformBundle::default(),
        ));
        e_c.push_children(&layer_ents);
        e_c.set_parent(world_root_id);
        let loaded_scene = scene_load_context.finish(Scene::new(world), None);
        // TODO:
        // Figure out what to use as a label.
        load_context.add_loaded_labeled_asset(MAP_SCENE, loaded_scene)
    };

    Ok(TiledMapAsset {
        map: tm,
        scene,
        tilemap_textures,
        tilemap_atlases,
    })
}

fn add_colliders(e: &mut EntityWorldMut, os: &Vec<Object>) {
    e.with_children(|cb| {
        os.iter()
            .filter(|o| {
                if let Some(TiledPropertyType::Bool(v)) = o.properties.get("collider") {
                    *v
                } else {
                    false
                }
            })
            .for_each(|o| {
                if let ObjectType::Point = o.otype {
                    return;
                }

                let (Vec2 { x, y }, collider) =
                    construct_geometry(&o.otype, o.size.map(|(x, y)| Vec2 { x, y }), None);

                cb.spawn((
                    TransformBundle::from_transform(
                        Transform::from_xyz(x, y, 0.)
                            .with_rotation(Quat::from_axis_angle(Vec3::Z, o.rotation.to_radians())),
                    ),
                    collider,
                ));
            })
    });
}

// fn handle_parallax(
//     camera_trans_q: Query<&Transform, With<Camera>>,
//     mut parallax_layer: Query<(&mut Transform, &LayerParallax), Without<Camera>>,
// ) {
//     let Ok(cam_transform) = camera_trans_q.get_single() else {
//         return;
//     };
//
//     parallax_layer
//         .iter_mut()
//         .for_each(|(mut layer_transform, layer_parallax)| {
//             let dist_from_layer_center = cam_transform.translation - layer_parallax.offset;
//
//             layer_transform.translation = layer_parallax.center
//                 - Vec3::new(
//                     dist_from_layer_center.x * (layer_parallax.parallax.x - 1.),
//                     dist_from_layer_center.y * (layer_parallax.parallax.y - 1.),
//                     0.,
//                 );
//         })
// }

fn construct_geometry(
    shape: &ObjectType,
    size: Option<Vec2>,
    scale_factor: Option<Vect>,
) -> (Vec2, Collider) {
    let scale_factor = scale_factor.unwrap_or(Vect::ONE);

    match shape {
        ObjectType::Rectangle => {
            let Some(size) = size else { unreachable!() };
            (
                size / 2. * scale_factor,
                Collider::cuboid(scale_factor.x * size.x / 2., scale_factor.y * size.y / 2.),
            )
        }
        ObjectType::Ellipse => {
            let Some(size) = size else { unreachable!() };
            todo!("Do it");
        }
        ObjectType::Polygon(points) => (
            Vec2::ZERO,
            Collider::convex_hull(
                &points
                    .iter()
                    .map(|(x, y)| Vect::new(scale_factor.x * *x, scale_factor.y * -*y))
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        ObjectType::Polyline(points) => (
            Vec2::ZERO,
            Collider::polyline(
                points
                    .iter()
                    .map(|p| Vec2 {
                        x: scale_factor.x * p.0,
                        y: scale_factor.y * -p.1,
                    })
                    .collect(),
                None,
            ),
        ),
        _ => todo!(),
    }
}

// TODO:
// I guess for an ObjectLayer ? Maybe ?
// fn tile_collision(
//     grid_size: TilemapGridSize,
//     object_layer: ObjectLayerData,
//     custom_size: Option<(f32, f32)>,
// ) -> Option<(Collider, TransformBundle)> {
//     let (width, height) = custom_size.unwrap_or((grid_size.x, grid_size.y));
//
//     let shapes = object_layer
//         .object_data()
//         .iter()
//         .filter_map(|object_data| {
//             let scale_factor = Vect::new(width / grid_size.x, height / grid_size.y);
//
//             let pos = scale_factor * Vect::new(object_data.x, -object_data.y);
//             let rot = object_data.rotation;
//
//             let (shape_pos, collider) = construct_geometry(&object_data.shape, Some(scale_factor));
//
//             Some((
//                 Vect::new(
//                     pos.x - width / 2. + shape_pos.x,
//                     pos.y + height / 2. - shape_pos.y,
//                 ),
//                 rot,
//                 collider,
//             ))
//         })
//         .collect::<Vec<_>>();
//
//     if shapes.len() == 1 {
//         let (pos, rot, collider) = shapes[0].clone();
//
//         Some((
//             collider,
//             TransformBundle::from_transform(Transform {
//                 translation: Vec3::new(pos.x, pos.y, 0.),
//                 rotation: Quat::from_rotation_x(rot),
//                 ..default()
//             }),
//         ))
//     } else if shapes.len() > 1 {
//         Some((Collider::compound(shapes), TransformBundle::default()))
//     } else {
//         None
//     }
// }
