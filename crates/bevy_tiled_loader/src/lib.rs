#[cfg(all(feature = "rapier2d_colliders", feature = "avian2d_colliders"))]
compile_error!("features `crate/rapier2d_colliders` and `crate/avian2d_colliders` are mutually exclusive");

pub mod load;
pub mod plugin;
pub mod relations;
pub mod types;
#[cfg(feature = "rapier2d_colliders")]
pub mod rapier_colliders;
#[cfg(feature = "avian2d_colliders")]
pub mod avian_colliders;
