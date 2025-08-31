/// For Bevy users, this is the main library you will interface with.
///
/// Usage :
/// ```
/// app.add_plugins(
///     TiledScenePlugin
/// )
///
/// // ...
///
/// fn spawn_tiled_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
///     let tiled_map_handle: Handle<TiledMapAsset> = asset_server.load("sample-map.tmx");
///
///     commands.spawn(
///         BufferedMapScene(tiled_map_handle)
///     );
/// }
/// ```
///
/// Note that when using PixelPerfect rendering methods, ensure that your Tiled objects are whole
/// numbers, otherwise things will appear to jitter.

#[cfg(all(feature = "rapier2d_colliders", feature = "avian2d_colliders"))]
compile_error!(
    "features `crate/rapier2d_colliders` and `crate/avian2d_colliders` are mutually exclusive"
);

#[cfg(feature = "avian2d_colliders")]
pub mod avian_colliders;
pub mod load;
pub mod plugin;
#[cfg(feature = "rapier2d_colliders")]
pub mod rapier_colliders;
pub mod relations;
pub mod types;
