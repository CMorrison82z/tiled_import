use crate::{load::TiledLoader, types::*};
use bevy::prelude::*;

pub struct TiledScenePlugin {
    // TODO:
    // Various options should be made available.
}

impl Plugin for TiledScenePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TiledMapContainer>()
        .register_type::<SerializedComponents>()
        .register_type_data::<TiledMapContainer, ReflectComponent>()
        .register_type_data::<SerializedComponents, ReflectComponent>()
        .init_asset::<TiledMapAsset>()
        .init_asset_loader::<TiledLoader>()
        .add_systems(Update, load_buffered_map)
        .add_observer(
            |trigger: Trigger<OnAdd, SerializedComponents>, query: Query<&SerializedComponents>, mut c: Commands| {
                let Ok(SerializedComponents (data_map)) = query.get(trigger.entity()) else {
                    return;
                };

                let mut ec = c.entity(trigger.entity());
                ec.remove::<SerializedComponents>();

                data_map.iter().for_each(|(sc, d_bytes)| {
                    match *sc {
                        #[cfg(feature = "rapier2d_colliders")]
                        SceneSerializedComponents::SerCollider  => {ec.insert(crate::rapier_colliders::deserialize_collider(&d_bytes).unwrap());},
                        #[cfg(feature = "avian2d_colliders")]
                        SceneSerializedComponents::SerCollider  => {ec.insert(crate::avian_colliders::deserialize_collider(&d_bytes).unwrap());},
                        #[cfg(feature = "avian2d_colliders")]
                        SceneSerializedComponents::SerRigidBody => {ec.insert(bincode::deserialize::<avian2d::prelude::RigidBody>(&d_bytes).unwrap());},
                        _ => unimplemented!(),
                    }
                });
            },
        );

    #[cfg(feature = "avian2d_colliders")]
    app.register_type::<avian2d::prelude::RigidBody>();
    }
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
