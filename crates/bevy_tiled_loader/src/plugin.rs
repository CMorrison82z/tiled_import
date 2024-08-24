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
