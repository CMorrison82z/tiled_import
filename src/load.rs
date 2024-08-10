use std::fs::read_to_string;

use bevy::asset::{io::Reader, AssetLoader, AssetPath, AsyncReadExt, AsyncWriteExt};
use bevy::asset::{Asset, Handle};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::math::Vec2;
use bevy::reflect::TypePath;
use bevy::sprite::TextureAtlasLayout;
use bevy::transform::components::{GlobalTransform, Transform};
use bevy::utils::hashbrown::HashMap;

use crate::data_types::*;
use crate::parse::parse;

#[derive(TypePath, Asset)]
pub struct TiledMapAsset {
    pub map: TiledMap,

    // TODO:
    // pub colliders: todo!(),
    pub tilemap_textures: HashMap<usize, Handle<bevy::prelude::Image>>,
    pub tilemap_atlases: HashMap<usize, Handle<TextureAtlasLayout>>,
}

// Stores a list of tiled layers.
#[derive(Component, Default)]
pub struct TiledLayersStorage {
    pub storage: HashMap<u32, Entity>,
}

// TODO:
// I'm not sure that I want to have the crate commit to this instance implementation...
// For example, GPU rendering would be more efficient (like `bevy_ecs_tilemap`)
#[derive(Default, Bundle)]
pub struct TiledMapBundle {
    pub tiled_map: Handle<TiledMapAsset>,
    pub storage: TiledLayersStorage,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

/// Allows us to do `AssetServer.load("MY_MAP.tmx")`
pub struct TiledLoader;

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

            let TiledMap {
                layers,
                grid_size,
                tile_size,
                tile_sets,
            } = &tm;

            let mut tilemap_textures = HashMap::default();
            let mut tilemap_atlases = HashMap::default();

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
                    let crate::data_types::Image {
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

                    tilemap_textures.insert(ind, texture_handle);
                    tilemap_atlases.insert(ind, texture_atlas);
                });
            });

            Ok(TiledMapAsset {
                map: tm,
                tilemap_textures,
                tilemap_atlases,
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["tmx"];
        EXTENSIONS
    }
}
