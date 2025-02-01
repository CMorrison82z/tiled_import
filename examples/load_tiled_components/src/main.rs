pub fn main() {
    App::new()
        .add_plugins((
            bevy_tiled_loader::plugin::TiledScenePlugin::default(),
            avian2d::PhysicsPlugins::default().with_length_unit(16.0),
            avian2d::PhysicsDebugPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, properties_to_components)
        .run();
}

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(
        TiledMapScene(asset_server.load("demo.tmx")),
    );
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
                            match_ok!(object.otype, ObjectType::Tile(t_guid))
                                .and_then(|t_guid| tma.map.get_tile_properties(t_guid))
                                .iter()
                                .flat_map(|hm| hm.iter()),
                        )
                    }
                    .for_each(|(k, v)| match k.as_str() {
                        "spike" => {
                            entity_commands.insert((
                                TouchHazard {
                                    amount: 10.,
                                    rate: 0.5,
                                },
                                ShapeCaster::new(
                                    avian2d::collision::Collider::rectangle(16., 16.),
                                    // Tiles are anchored at top left corner, so
                                    // ShapeCaster needs to be shifted
                                    8. * Vec2::new(1., -1.),
                                    0.,
                                    Dir2::X,
                                )
                                .with_max_distance(0.0)
                                .with_query_filter(
                                    avian2d::prelude::SpatialQueryFilter::from_mask(
                                        GameLayer::Player,
                                    ),
                                ),
                            ));
                        }
                        _ => println!("Un-implemented object property : {}", k),
                    }),
                    TiledId::Layer(_) => (), // TODO:
                    TiledId::Tile(_) => (),  // TODO:
                }
            })
    })
}

