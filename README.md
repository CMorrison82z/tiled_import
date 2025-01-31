Tiled utilities and an AssetLoader for the [Bevy game engine](https://bevyengine.org/).

> [!CAUTION]
> There are still quite a few bugs, missing / un-intuitive functionality, missing docs, etc.
> Usage, feedback, and contribution will help to improve support.

---

## Features

- Parse Group, Tile, and Object layers, Tile and object properties.
- Loads the scene into Bevy.
- Generate Tiled Colliders as [bevy_rapier2d](https://docs.rs/bevy_rapier2d/latest/bevy_rapier2d/geometry/struct.Collider.html) or [avian2d](https://docs.rs/avian2d/0.2.0/avian2d/collision/collider/struct.Collider.html) Colliders.

## Documentation

- [Bevy Plugin documentation](DOCS_URL)
- [Parser documentation](DOCS_URL)

## Bevy Usage Example

Add `bevy_tiled_loader` to `Cargo.toml`:
```toml
# For 2D applications:
[dependencies]
bevy_tiled_loader = {version = "*", features = "avian2d_colliders"} # You should explicitly set the version.
```

Add the plugin and load a `*.tmx`:
```rust
app.add_plugins(
  TiledScenePlugin
)

// ...

fn spawn_tiled_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
  let tiled_map_handle: Handle<TiledMapAsset> = asset_server.load("my_first_tiled_map.tmx");

  commands.spawn(
    BufferedMapScene(tiled_map_handle)
  );
}

```

## Supported Bevy Versions

| Bevy    | My Tiled Loader |
| ------- | ----- |
| 0.15    | 0.1   |

## Future Features

- Settings to control how the `*.tmx` is parsed and how / what gets generated.

## Contributing

If you encounter any problems, feel free to open issues or create pull requests.

## Acknowledgements

Bevy is awesome.
