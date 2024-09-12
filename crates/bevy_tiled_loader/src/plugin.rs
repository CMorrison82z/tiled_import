use crate::{load::TiledLoader, relations::deserialize_rapier_collider, types::*};
use bevy::prelude::*;

pub fn tiled_scene_plugin(app: &mut App) {
    app.register_type::<TiledMapContainer>()
        .register_type::<Serialized>()
        .register_type_data::<TextureAtlas, ReflectComponent>()
        .register_type_data::<TiledMapContainer, ReflectComponent>()
        .register_type_data::<Serialized, ReflectComponent>()
        .init_asset::<TiledMapAsset>()
        .init_asset_loader::<TiledLoader>()
        .add_systems(Update, load_buffered_map)
        .observe(
            |trigger: Trigger<OnAdd, Serialized>, query: Query<&Serialized>, mut c: Commands| {
                let Ok(Serialized { data, thingy }) = query.get(trigger.entity()) else {
                    return;
                };

                let mut ec = c.entity(trigger.entity());
                ec.remove::<Serialized>();

                ec.insert(match thingy {
                    SceneSerializedComponents::RCollider => {
                        deserialize_rapier_collider(&data).unwrap()
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
        .filter_map(|(e, BufferedMapScene(sc, m))| {
            if s.get_load_state(m) == Some(bevy::asset::LoadState::Loaded) {
                a.get(m).map(|tma| (e, sc, tma))
            } else {
                None
            }
        })
        .for_each(|(e, sc, tma)| {
            let mut e_c = c.entity(e);
            e_c.remove::<BufferedMapScene>();

            // TODO:
            // Don't clone the scene.
            e_c.insert(sc.clone());
            e_c.insert(tma.scene.clone());
        })
}
