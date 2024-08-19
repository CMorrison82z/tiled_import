use std::fs::read_to_string;

use bevy::asset::{io::Reader, AssetLoader, AssetPath, AsyncReadExt, AsyncWriteExt};
use bevy::asset::{Asset, Handle, LoadContext};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::world::World;
use bevy::hierarchy::BuildWorldChildren;
use bevy::math::Vec2;
use bevy::prelude::{SpatialBundle, TransformBundle};
use bevy::reflect::TypePath;
use bevy::scene::Scene;
use bevy::sprite::{Anchor, Sprite, SpriteBundle, TextureAtlas, TextureAtlasLayout};
use bevy::transform::components::{GlobalTransform, Transform};
use bevy::utils::hashbrown::HashMap;

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
    let TiledMap {
        layers,
        grid_size,
        tile_size,
        tile_sets,
    } = &tm;

    let mut tilemap_textures = Vec::with_capacity(tile_sets.len());
    let mut tilemap_atlases = Vec::with_capacity(tile_sets.len());

    tile_sets.iter().enumerate().for_each(|(ind, ts)| {
        let TileSet {
            tile_size,
            first_gid,
            name,
            spacing,
            margin,
            images,
            tile_stuff,
        } = ts;

        images.iter().for_each(|i| {
            let tiled_parse::data_types::Image {
                source,
                format,
                dimensions: (columns, rows),
            } = i;

            let tmx_dir = load_context
                .path()
                .parent()
                .expect("The asset load context was empty.");
            let tile_path = tmx_dir.join(&source);
            let asset_path = AssetPath::from(tile_path);

            let texture_handle: Handle<bevy::prelude::Image> =
                load_context.load(asset_path.clone());

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
                        Vec2::new(tile_size.0 as f32, tile_size.1 as f32),
                        *columns as usize,
                        *rows as usize,
                        // TODO:
                        // I'm not sure this translates correctly
                        Some((*spacing as f32) * Vec2::ONE),
                        Some((*margin as f32) * Vec2::ONE),
                    ),
                );

            tilemap_textures.push(texture_handle);
            tilemap_atlases.push(texture_atlas);
        });
    });

    // Load scene
    let scene = {
        let mut scene_load_context = load_context.begin_labeled_asset();
        let mut world = World::default();

        // TODO:
        // Maybe hide behind a feature flag
        // Load colliders.
        // Colliders are not assets, so they'll be added as components within the scene.
        let world_root_id = world.spawn(SpatialBundle::INHERITED_IDENTITY).id();

        let (ghalfsize_x, ghalfsize_y) = (tm.grid_size.0 as f32 / 2., tm.grid_size.1 as f32 / 2.);

        let mut tile_ents = Vec::new();

        tm.layers.iter().for_each(|x: &TiledLayer| match x {
            // TODO:
            // Handle other layer types
            TiledLayer::Tile(Layer { name, content, .. }) => {
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

                            // NOTE:
                            // There is an assumption that it's being loaded for a 2d camera here.

                            // TODO:
                            // Figure out why I need the additional "0.25"
                            tile_ents.push(
                                world
                                    .spawn((
                                        SpriteBundle {
                                            sprite: Sprite {
                                                flip_x: flip_h,
                                                flip_y: flip_v,
                                                anchor: Anchor::TopLeft,
                                                ..Default::default()
                                            },
                                            transform: Transform::from_xyz(x, y, 0.),
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
                                    ))
                                    .id(),
                            );
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
        e_c.push_children(&tile_ents);
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
