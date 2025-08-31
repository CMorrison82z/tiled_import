//! NOTE:
//! Tiles with objects (for example a collider) is different from an object `ObjectType::Tile`.
//! - Tiles are in a TileLayer. The associated objects are children of the Tile Entity
//! - `ObjectType::Tile`s are just a single Entity, containing a Sprite for the tile.
use bevy::asset::{io::Reader, AssetLoader, AssetPath, AsyncReadExt};
use bevy::asset::{Handle, LoadContext};
use bevy::image::{TextureAtlas, TextureAtlasLayout};
use bevy::math::{UVec2, Vec2};
use bevy::prelude::*;
use bevy::scene::Scene;
use bevy::sprite::{Anchor, Sprite};
use bevy::transform::components::Transform;

#[cfg(feature = "rapier2d_colliders")]
use bevy_rapier2d::prelude::*;
use tiled_parse::relations::{get_tile_id, get_tileset_for_gid, tile_set_rows_and_columns};

use crate::types::*;
use crate::relations::is_collider;
use tiled_parse::parse::*;
use tiled_parse::types::*;

/// Load `*.tmx` via `AssetServer.load("MY_MAP.tmx")`
///
/// Note that objects within tile's tileset are NOT marked with ANY TiledId.
#[derive(Default)]
pub struct TiledLoader;

pub const MAP_SCENE: &str = "MapScene";

// TODO:
// Improve error
impl AssetLoader for TiledLoader {
    type Asset = TiledMapAsset;
    /// TODO:
    /// Add settings
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data).await?;

        let data_as_utf8 = std::str::from_utf8(&data).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Could not load TMX map: {e}"),
            )
        })?;

        // let path = p
        //     .to_str()
        //     .and_then(|p| resolve_path(&ctx, p))
        //     .ok_or(OpenFileFailed)?;
        // ctx.read_asset_bytes(path)
        //     .await
        //     .map_or(Err(OpenFileFailed), |bytes| {
        //         tobj::load_mtl_buf(&mut bytes.as_slice())
        //     })

        let tm: TiledMap = parse(data_as_utf8, &mut |p| {
            let p2 = resolve_path(&load_context, &p).unwrap().to_string();
            Box::pin(async {
                load_context
                    .begin_labeled_asset()
                    .read_asset_bytes(p2)
                    .await
                    .map(|v| String::from_utf8(v).unwrap())
                    .map_err(drop)
            })
        })
        .await
        .map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("Could not load TMX map"))
        })?;

        load_tmx(load_context, tm)
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["tmx"];
        EXTENSIONS
    }
}

fn resolve_path<'a>(ctx: &LoadContext, path: &'a str) -> Option<AssetPath<'a>> {
    ctx.asset_path().parent()?.resolve(path).ok()
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
            image: tiled_parse::types::Image {
                source,
                ..
            },
            tile_stuff,
        } = ts;

        let tmx_dir = load_context
            .path()
            .parent()
            .expect("The asset load context was empty.");
        let tile_path = tmx_dir.join(&source);
        let asset_path = AssetPath::from(tile_path);

        // TODO:
        // I think needs to use another loading method to register it as a dependency
        let texture_handle: Handle<bevy::prelude::Image> = load_context.load(asset_path.clone());

        let file_name = source
            .file_name()
            .expect("Should have file name")
            .to_str()
            .expect("Valid utf8");

        let (columns, rows) = tile_set_rows_and_columns(ts);

        // TODO:
        // I don't know if I should use "add_labeled_asset", and if the arguments are
        // conventional
        let texture_atlas: Handle<bevy::prelude::TextureAtlasLayout> = load_context
            .add_labeled_asset(
                file_name.into(),
                TextureAtlasLayout::from_grid(
                    UVec2::new(tile_size.0, tile_size.1),
                    columns,
                    rows,
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

        let world_root_id = world
            .spawn((Transform::IDENTITY, Visibility::Inherited))
            .id();

        let mut layer_ents = Vec::new();

        let tile_size_f32 = (tile_size.0 as f32, tile_size.1 as f32);

        layers.iter_depth().enumerate().for_each(|(i, TiledLayer {
                id, name, content, visible, ..
            })| {
            if !visible {return};

            match content {
                // TODO:
                // Handle other layer types
                LayerType::TileLayer(tile_layer) => {
                    // TODO:
                    // Review if just setting the z-coordeinate to the iter-index is good.
                    let spatial_bundle = (
                        Transform::from_translation(Vec2::ZERO.extend(i as f32)),
                        Visibility::Inherited,
                    );

                    let layer_ent = world
                        .spawn((Name::new(name.clone()), spatial_bundle, TiledId::Layer(*id)))
                        .id();

                    layer_ents.push(layer_ent);

                    tile_layer
                        .indexed_iter()
                        .filter_map(|(p, t)| t.map(|v| (p, v)))
                        .for_each(
                            |(
                                tile_pos,
                                LayerTile {
                                    tile: tile_gid,
                                    flip_h,
                                    flip_v,
                                    flip_d,
                                },
                            )| {
                                if flip_d {
                                    panic!("`flip_d` is not yet implemented");
                                }
                                let (world_pos_x, world_pos_y) = (
                                    tile_size_f32.0 * tile_pos.0 as f32,
                                    -tile_size_f32.1 * tile_pos.1 as f32,
                                );

                                let tile_tileset = get_tileset_for_gid(tile_sets, tile_gid)
                                    .expect("Tile should belong to tileset");

                                let tileset_index = tile_sets
                                    .iter()
                                    .position(|ts| ts.first_gid == tile_tileset.first_gid)
                                    .expect("Yes");

                                let local_tile_id = get_tile_id(tile_tileset, tile_gid);

                                let tile_aux_info_opt = tile_tileset.tile_stuff.get(&local_tile_id);

                                let mut tile_entity = world.spawn((
                                    Sprite {
                                        image: tilemap_textures.get(tileset_index).unwrap().clone(),
                                        texture_atlas: Some(TextureAtlas {
                                            layout: tilemap_atlases
                                                .get(tileset_index)
                                                .unwrap()
                                                .clone(),
                                            index: local_tile_id as usize,
                                        }),
                                        flip_x: flip_h,
                                        flip_y: flip_v,
                                        anchor: Anchor::TopLeft,
                                        ..Default::default()
                                    },
                                    TiledIndex(tile_pos.0.try_into().unwrap(), tile_pos.1.try_into().unwrap()),
                                    Transform::from_xyz(world_pos_x, world_pos_y, 0.),
                                    TiledId::Tile(tile_gid.0),
                                    ChildOf(layer_ent),
                                ));

                                if let Some(tile_aux_info) = tile_aux_info_opt {
                                    tile_entity.with_children(|child_spawner| {
                                        tile_aux_info.objects.iter().for_each(|o| {
                                            let c_e = child_spawner.spawn(TileObject(o.id));

                                            // FIXME: 
                                            // Dynamic RigidBody's move independently from the Sprite.
                                            // Maybe put `RigidBody` (NOT the `Collider` !) in `tile_entity` ?
                                            if is_collider(o) {
                                                #[cfg(feature = "rapier2d_colliders")]
                                                if let Some(obj_bundle) = crate::rapier_colliders::object_collider(o) {
                                                    c_e.insert(obj_bundle);
                                                }

                                                #[cfg(feature = "avian2d_colliders")]
                                                if let Some(obj_bundle) = crate::avian_colliders::object_collider(o) {
                                                    c_e.insert(obj_bundle);
                                                }
                                            }
                                        });
                                    });

                                    if let Some(animation) = &tile_aux_info.animation {
                                        let AnimationFrame { tile_id, .. } = animation[0];

                                        tile_entity.insert((
                                            Sprite {
                                                image: tilemap_textures.get(tileset_index).unwrap().clone(),
                                                texture_atlas: Some(TextureAtlas {
                                                    layout: tilemap_atlases
                                                        .get(tileset_index)
                                                        .unwrap()
                                                        .clone(),
                                                    index: tile_id as usize,
                                                }),
                                                flip_x: flip_h,
                                                flip_y: flip_v,
                                                anchor: Anchor::TopLeft,
                                                ..Default::default()
                                            },
                                            TiledAnimation {
                                                animation: animation.into_iter().map(|&f| f.into()).collect(),
                                                player: TiledAnimationPlayer::Cycled,
                                                start_time: 0.
                                            }
                                        ));
                                    }
                                }
                            },
                        );
                }
                LayerType::ObjectLayer(os) => {
                    let layer_entity = world.spawn((
                        Name::new(name.clone()),
                        Transform::IDENTITY,
                        TiledId::Layer(*id),
                    ));

                    let layer_entity_id = layer_entity.id();

                    layer_ents.push(layer_entity_id);

                    os.iter().for_each(|o| {
                        let Object {
                            id,
                            position,
                            size,
                            rotation,
                            otype,
                            ..
                        } = o;

                        if let &ObjectType::Tile(tile_gid) = otype {
                            let scale_factor = if let Some((width, height)) = size {
                                Vec2::new(width / (tile_size_f32.0), height / (tile_size_f32.1))
                            } else {
                                Vec2::ONE
                            };

                            let world_pos = Vec2::new(
                                position.0,
                                -(position.1 - scale_factor.y * tile_size_f32.1),
                            );

                            let tile_tileset = get_tileset_for_gid(tile_sets, tile_gid)
                                .expect("Tile should belong to tileset");

                            let tileset_index = tile_sets
                                .iter()
                                .position(|ts| ts.first_gid == tile_tileset.first_gid)
                                .expect("Yes");

                            let local_tile_id = get_tile_id(tile_tileset, tile_gid);

                            let tile_aux_info_opt = tile_tileset.tile_stuff.get(&local_tile_id);

                            let mut tile_entity = world.spawn((
                                Sprite {
                                    image: tilemap_textures.get(tileset_index).unwrap().clone(),
                                    texture_atlas: Some(TextureAtlas {
                                        layout: tilemap_atlases.get(tileset_index).unwrap().clone(),
                                        index: local_tile_id as usize,
                                    }),
                                    anchor: Anchor::TopLeft,
                                    ..Default::default()
                                },
                                // FIXME: Use a correct z-index.
                                Transform::from_translation(world_pos.extend(3.))
                                    .with_rotation(Quat::from_axis_angle(
                                        Vec3::Z,
                                        rotation.to_radians(),
                                    ))
                                    .with_scale(scale_factor.extend(1.)),
                                TiledId::Object(*id),
                                ChildOf(layer_entity_id),
                            ));

                            if let Some(tile_aux_info) = tile_aux_info_opt {
                                tile_entity.with_children(|child_spawner| {
                                    tile_aux_info.objects.iter().for_each(|o| {
                                        let c_e = child_spawner.spawn(TileObject(o.id));

                                        // FIXME:
                                        // Dynamic RigidBody's move independently from the Sprite.
                                        // Maybe put `RigidBody` (NOT the `Collider` !) in `tile_entity` ?
                                        if is_collider(o) {
                                            #[cfg(feature = "rapier2d_colliders")]
                                            if let Some(obj_bundle) = crate::rapier_colliders::object_collider(o) {
                                                c_e.insert(obj_bundle);
                                            }

                                            #[cfg(feature = "avian2d_colliders")]
                                            if let Some(obj_bundle) = crate::avian_colliders::object_collider(o) {
                                                c_e.insert(obj_bundle);
                                            }
                                        }
                                    });
                                });
                            }
                        } else {
                            let obj_ent = world.spawn((ChildOf(layer_entity_id), TiledId::Object(*id)));

                            #[cfg(feature = "rapier2d_colliders")]
                            if let Some(collider_bundle) = crate::rapier_colliders::object_collider(o) {
                                obj_ent.insert(collider_bundle);
                            }

                            #[cfg(feature = "avian2d_colliders")]
                            if let Some(collider_bundle) = crate::avian_colliders::object_collider(o) {
                                obj_ent.insert(collider_bundle);
                            }
                        }
                    });
                }
                LayerType::ImageLayer(ImageStuff {
                    repeatx,
                    repeaty,
                    image: tiled_parse::types::Image {
                        source,
                        ..
                    }
                }) => {
                    let tmx_dir = load_context
                        .path()
                        .parent()
                        .expect("The asset load context was empty.");
                    let tile_path = tmx_dir.join(&source);

                    layer_ents.push(world.spawn((
                        Name::new(name.clone()),
                        Transform::IDENTITY,
                        TiledId::Layer(*id),
                        Sprite {
                            image: scene_load_context.load(AssetPath::from(tile_path)),
                            image_mode: SpriteImageMode::Tiled {
                                tile_x: *repeatx,
                                tile_y: *repeaty,
                                stretch_value: 1.
                            },
                            ..default()
                        }
                    )).id());
                }
                LayerType::Group => println!("Group layer {name}"),
            }
        });

        // TODO:
        // I'm not convinced this `per-entity` thing is very good.
        let mut e_c = world.spawn((
            TiledMapContainer,
            // TODO:
            // There may be some situation where it won't just be 0 ?
            Transform::IDENTITY,
            Visibility::Inherited,
            ChildOf(world_root_id),
        ));

        e_c.add_children(&layer_ents);

        let loaded_scene = scene_load_context.finish(Scene::new(world));
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
