use crate::{load::TiledLoader, types::*};
use bevy::prelude::*;

pub struct TiledScenePlugin {
    pub auto_play_animations: bool,
}

impl Default for TiledScenePlugin {
    fn default() -> Self {
        TiledScenePlugin {
            auto_play_animations: true
        }
    }
}

impl Plugin for TiledScenePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TiledMapContainer>()
            .register_type::<SerializedComponents>()
            .register_type::<TiledId>()
            .register_type::<TileObject>()
            .register_type::<TiledIndex>()
            .register_type::<TiledAnimation>()
            .register_type_data::<TiledId, ReflectComponent>()
            .register_type_data::<TileObject, ReflectComponent>()
            .register_type_data::<TiledIndex, ReflectComponent>()
            .register_type_data::<TiledAnimation, ReflectComponent>()
            .register_type_data::<TiledMapContainer, ReflectComponent>()
            .register_type_data::<SerializedComponents, ReflectComponent>()
            .init_asset::<TiledMapAsset>()
            .init_asset_loader::<TiledLoader>()
            .add_event::<TiledAnimationCompleted>()
            .add_systems(Update, (load_buffered_map, emit_animation_event))
            .add_observer(
                |trigger: Trigger<OnAdd, SerializedComponents>,
                 query: Query<&SerializedComponents>,
                 mut c: Commands| {
                    let Ok(SerializedComponents(data_map)) = query.get(trigger.target()) else {
                        return;
                    };

                    let mut ec = c.entity(trigger.target());
                    ec.remove::<SerializedComponents>();

                    data_map.iter().for_each(|(sc, d_bytes)| match *sc {
                        #[cfg(feature = "rapier2d_colliders")]
                        SceneSerializedComponents::SerCollider => {
                            ec.insert(
                                crate::rapier_colliders::deserialize_collider(&d_bytes).unwrap(),
                            );
                        }
                        #[cfg(feature = "avian2d_colliders")]
                        SceneSerializedComponents::SerCollider => {
                            ec.insert(
                                crate::avian_colliders::deserialize_collider(&d_bytes).unwrap(),
                            );
                        }
                        #[cfg(feature = "avian2d_colliders")]
                        SceneSerializedComponents::SerRigidBody => {
                            ec.insert(
                                bincode::deserialize::<avian2d::prelude::RigidBody>(&d_bytes)
                                    .unwrap(),
                            );
                        }
                        _ => unimplemented!(),
                    });
                },
            );

        if self.auto_play_animations {
            app.add_systems(Update, run_animations);
        }

        #[cfg(feature = "avian2d_colliders")]
        app.register_type::<avian2d::prelude::RigidBody>();
    }
}

fn load_buffered_map(
    mut c: Commands,
    q: Query<(Entity, &TiledMapScene), Without<SceneRoot>>,
    s: Res<AssetServer>,
    a: Res<Assets<TiledMapAsset>>,
) {
    q.iter()
        .filter_map(|(e, TiledMapScene(m))| {
            if matches!(s.get_load_state(m), Some(bevy::asset::LoadState::Loaded)) {
                a.get(m).map(|tma| (e, tma))
            } else {
                None
            }
        })
        .for_each(|(e, tma)| {
            let mut e_c = c.entity(e);

            e_c.insert(SceneRoot(tma.scene.clone()));
        })
}

fn run_animations(
    mut q: Query<(&mut Sprite, &TiledAnimation)>,
    t: Res<Time>
) {
    q.iter_mut().for_each(|(mut s, ta)| {
        if let Some(current_id) = ta.get_current_tile(t.elapsed_secs()) {
            let current_id = current_id as usize;

            if s.texture_atlas.as_ref().unwrap().index != current_id {
                s.texture_atlas.as_mut().unwrap().index = current_id;
            }
        }
    })
}

fn emit_animation_event(
    mut c: Commands,
    q: Query<(Entity, &TiledAnimation)>,
    t: Res<Time>
) {
    c.trigger_targets(
        TiledAnimationCompleted,
        q.iter().filter_map(|(entity, ta)| {
            let end_time = ta.get_end_time(t.elapsed_secs());
            if t.elapsed_secs() - t.delta_secs() < end_time && t.elapsed_secs() >= end_time {
                Some(entity)
            } else {None}
        }).collect::<Vec<Entity>>()
    );
}
