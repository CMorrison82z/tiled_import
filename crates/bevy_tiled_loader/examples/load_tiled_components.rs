use bevy::prelude::*;
use bevy_tiled_loader::types::*;
use bevy_tiled_loader::relations::*;
use tiled_parse::types::*;

pub fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            bevy_tiled_loader::plugin::TiledScenePlugin::default(),
            avian2d::PhysicsPlugins::default(),
            avian2d::prelude::PhysicsDebugPlugin::default(),
        ))
        .insert_gizmo_config(
            avian2d::prelude::PhysicsGizmos {
                aabb_color: None,
                collider_color: Some(Color::WHITE),
                ..Default::default()
            },
            GizmoConfig::default(),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, properties_to_components)
        .run();
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(
        TiledMapScene(asset_server.load("sample-map.tmx")),
    );

    commands.spawn((
        Camera2d::default(),
        Transform::from_xyz(175., -100., 0.).with_scale(0.3 * Vec3::ONE)
    ));
}

fn properties_to_components(
    mut commands: Commands,
    tiled_map_assets: Res<Assets<TiledMapAsset>>,
    query_scenes: Query<(Entity, &SceneRoot), Added<SceneRoot>>,
    children: Query<&Children>,
    query_tiled_id: Query<&TiledId>,
) {
    query_scenes.iter().for_each(|(entity, scene_root)| {
        let Some(tma) = get_tiled_map_with_scene(&tiled_map_assets, scene_root) else {
            return;
        };

        children
            .iter_descendants(entity)
            .filter_map(|child_e| query_tiled_id.get(child_e).ok().map(|tile_id| (child_e, tile_id)))
            .for_each(|(child_e, tile_id)| {
                let mut entity_commands = commands.entity(child_e);
                match tile_id {
                    TiledId::Object(o_id) => {
                        // For Tile objects `ObjectType::Tile`, you may also want to include the properties placed on
                        // the tile itself with the object's properties, done like so :
                        let object = tma.map.get_scene_object(*o_id).unwrap();
                        object.properties.iter().chain(
                            try_match::match_ok!(object.otype, ObjectType::Tile(t_guid))
                                .and_then(|t_guid| tma.map.get_tile_properties(t_guid))
                                .iter()
                                .flat_map(|hm| hm.iter()),
                        )
                    }
                    .for_each(|(k, v)| match k.as_str() {
                        "coin" => {
                            entity_commands.insert((
                                Name::new("GameCoin")
                            ));
                        },
                        "door" => println!("The door"),
                        "is_locked" => println!("The door is locked"),
                        _ => println!("Un-implemented object property : {}", k),
                    }),
                    TiledId::Layer(_) => (), // TODO:
                    TiledId::Tile(_) => (),  // TODO:
                }
            })
    })
}



