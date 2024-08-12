use std::fs::read_to_string;

use bevy::asset::{io::Reader, AssetLoader, AssetPath, AsyncReadExt, AsyncWriteExt};
use bevy::asset::{Asset, Handle, LoadContext};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::world::World;
use bevy::hierarchy::BuildWorldChildren;
use bevy::math::Vec2;
use bevy::prelude::SpatialBundle;
use bevy::reflect::TypePath;
use bevy::scene::Scene;
use bevy::sprite::TextureAtlasLayout;
use bevy::transform::components::{GlobalTransform, Transform};
use bevy::utils::hashbrown::HashMap;

use crate::types::TiledMapAsset;
use tiled_parse::data_types::*;
use tiled_parse::parse::*;

/// Allows us to do `AssetServer.load("MY_MAP.tmx")`
pub struct TiledLoader;

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
    let scene_handle = {
        let mut scene_load_context = load_context.begin_labeled_asset();
        let mut world = World::default();

        // TODO:
        // Maybe hide behind a feature flag
        // Load colliders.
        // Colliders are not assets, so they'll be added as components within the scene.
        let world_root_id = world
            .spawn(SpatialBundle::INHERITED_IDENTITY)
            .with_children(|parent| {
                // TODO:
                // Load entities and stuff I think
            })
            .id();

        let loaded_scene = scene_load_context.finish(Scene::new(world), None);
        // TODO:
        // Figure out what to use as a label.
        load_context.add_loaded_labeled_asset(scene_label(&scene), loaded_scene)
    };

    Ok(TiledMapAsset {
        map: tm,
        tilemap_textures,
        tilemap_atlases,
    })
}
