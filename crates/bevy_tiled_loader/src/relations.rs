use bevy::prelude::*;
use tiled_parse::{relations::{get_tile_id, get_tileset_for_gid}, types::{AnimationFrame, Gid, TiledLayer}};
use try_match::match_ok;

use crate::types::*;
use tiled_parse::types::*;

/// Given a `TiledMapAsset`, returns `Entity`'s with a SceneRoot containing `TiledMapAsset.scene`.
pub fn get_entities_with_tiled_map(
    a: &TiledMapAsset,
    q: &Query<(Entity, &SceneRoot)>,
) -> Vec<Entity> {
    q.iter()
        .filter_map(|(e, SceneRoot(h))| if a.scene == *h { Some(e) } else { None })
        .collect()
}

pub fn is_collider(o: &Object) -> bool {
    matches!(o.properties.get("collider"), Some(TiledPropertyType::Bool(true)))
}

/// Given a `TiledLayer` from a `TiledMapAsset`.
pub fn get_layer_from_id(a: &TiledMapAsset, t_id: TiledId) -> Option<&TiledLayer> {
    let id = match_ok!(t_id, TiledId::Layer(x))?;

    a.map.get_layer_from_id(id)
}

// TODO:
// Change to a `Result<Sprite, TiledGetError>` to provide more context about what was missing.
pub fn get_tile_sprite(a: &TiledMapAsset, t_gid: Gid) -> Option<Sprite> {
    let tile_sets = &a.map.tile_sets;

    let tile_tileset = get_tileset_for_gid(&tile_sets, t_gid)?;

    let tileset_index = tile_sets
        .iter()
        .position(|ts| ts.first_gid == tile_tileset.first_gid)?;

    let local_tile_id = get_tile_id(&tile_tileset, t_gid);

    Some(Sprite {
        image: a.tilemap_textures.get(tileset_index)?.clone(),
        texture_atlas: Some(TextureAtlas {
            layout: a.tilemap_atlases.get(tileset_index)?.clone(),
            index: local_tile_id as usize,
        }),
        anchor: bevy::sprite::Anchor::TopLeft,
        ..Default::default()
    })
}

// FIXME:
// This function will be deprecated when `TiledAnimation` becomes an Asset.
//
/// Returns the Sprite as well, because it would be a lot of duplicate work to get the Sprite
/// separately
pub fn get_animation(a: &TiledMapAsset, t_gid: Gid, start_time_secs: f32) -> Option<(TiledAnimation, Sprite)> {
    let tile_sets = &a.map.tile_sets;

    let tile_tileset = get_tileset_for_gid(&tile_sets, t_gid)?;

    let tileset_index = tile_sets
        .iter()
        .position(|ts| ts.first_gid == tile_tileset.first_gid)?;

    let local_tile_id = get_tile_id(&tile_tileset, t_gid);

    let animation = tile_tileset.tile_stuff.get(&local_tile_id)?.animation.as_ref()?;

    Some((
        TiledAnimation {
            animation: animation.into_iter().map(|&af| af.into()).collect(),
            start_time: start_time_secs,
            player: TiledAnimationPlayer::Once
        },
        Sprite {
            image: a.tilemap_textures.get(tileset_index).unwrap().clone(),
            texture_atlas: Some(TextureAtlas {
                layout: a.tilemap_atlases.get(tileset_index).unwrap().clone(),
                index: animation[0].tile_id as usize,
            }),
            anchor: bevy::sprite::Anchor::TopLeft,
            ..Default::default()
        }
    ))
}

impl TiledId {
    pub fn as_tile_gid(&self) -> Option<Gid> {
        match_ok!(self, TiledId::Tile(x)).map(|&id| Gid(id))
    }
}

// TODO:
// All functions convert to secs (from milli-secs). Wasteful computation.
impl TiledAnimation {
    /// Is `None` if the time has elapsed beyond the duration of the final frame.
    /// If it is `Cycled` will always be `Some`
    pub fn get_current_tile(&self, now_secs: f32) -> Option<tiled_parse::types::ID> {
        match self.player {
            TiledAnimationPlayer::Once => self.animation.iter().enumerate().scan(self.start_time, |acc_t, (i, &AnimationFrameReflect {tile_id, duration})| {
                if *acc_t <= now_secs {
                    *acc_t += duration / 1000.;

                    // If it is the last frame and the time has elapsed beyond the duration of the last
                    // frame, then we've gone past the length of the animation.
                    if i == (self.animation.len() - 1) && now_secs > *acc_t {
                        Some(None)
                    } else {
                        Some(Some(tile_id))
                    }
                } else {
                    None
                }
            }).last().flatten(),
            TiledAnimationPlayer::Cycled => {
                let remainder = now_secs % self.get_net_duration();

                // NOTE:
                // Looks similar to `get_current_tile`, but has less checks and doesn't need to enumerate
                // the iterator.
                self.animation.iter().scan(self.start_time, |acc_t, &AnimationFrameReflect {tile_id, duration}| {
                    if *acc_t <= (self.start_time + remainder) {
                        *acc_t += duration / 1000.;

                        Some(tile_id)
                    } else {
                        None
                    }
                }).last()
            }
        }
    }
    pub fn get_net_duration(&self) -> f32 {
        self.animation.iter().map(|af| af.duration / 1000.).sum()
    }
    pub fn get_end_time(&self, now_secs: f32) -> f32 {
        match self.player {
            TiledAnimationPlayer::Once => self.start_time + self.get_net_duration(),
            TiledAnimationPlayer::Cycled =>
                self.start_time + ((now_secs - self.start_time) / self.get_net_duration()).ceil() * self.get_net_duration()
        }
    }
}
