use crate::{load::TiledLoader, types::*};
use bevy::prelude::*;

pub fn tiled_scene_plugin(app: &mut App) {
    app.register_type::<TiledMapContainer>()
        .register_type::<Serialized>()
        .register_type_data::<TiledMapContainer, ReflectComponent>()
        .register_type_data::<Serialized, ReflectComponent>()
        .init_asset::<TiledMapAsset>()
        .init_asset_loader::<TiledLoader>()
        .add_systems(Update, load_buffered_map)
        .add_observer(
            |trigger: Trigger<OnAdd, Serialized>, query: Query<&Serialized>, mut c: Commands| {
                let Ok(Serialized { data, thingy }) = query.get(trigger.entity()) else {
                    return;
                };

                let mut ec = c.entity(trigger.entity());
                ec.remove::<Serialized>();

                ec.insert(match thingy {
                    SceneSerializedComponents::SerCollider => {
                        #[cfg(feature = "rapier2d_colliders")]
                        crate::rapier_colliders::deserialize_collider(&data).unwrap();
                        #[cfg(feature = "avian2d_colliders")]
                        crate::avian_colliders::deserialize_collider(&data).unwrap();
                    }
                });
            },
        );
}

fn load_buffered_map(
    mut c: Commands,
    q: Query<(Entity, &BufferedMapScene)>,
    s: Res<AssetServer>,
    a: Res<Assets<TiledMapAsset>>,
) {
    q.iter()
        .filter_map(|(e, BufferedMapScene(m))| {
            if matches!(s.get_load_state(m), Some(bevy::asset::LoadState::Loaded)) {
                a.get(m).map(|tma| (e, tma))
            } else {
                None
            }
        })
        .for_each(|(e, tma)| {
            let mut e_c = c.entity(e);
            e_c.remove::<BufferedMapScene>();

            e_c.insert(SceneRoot(tma.scene.clone()));
        })
}
