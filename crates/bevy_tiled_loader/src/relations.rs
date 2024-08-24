use bevy::prelude::Component;
use bevy_rapier2d::prelude::Collider;
use bincode::ErrorKind;

use crate::types::{SceneSerializedComponents, Serialized};

// TODO:
// Likely can be made more general and convenient
pub fn deserialize_rapier_collider(b: &[u8]) -> Result<Collider, Box<ErrorKind>> {
    bincode::deserialize::<Collider>(b)
}

// TODO:
// Just an idea for future, nice implementation...
// pub enum DeRes {
//     RCollider(Collider),
// }
//
// pub fn deserialize_component(
//     Serialized { data, thingy }: Serialized,
// ) -> Result<DeRes, Box<ErrorKind>> {
//     match thingy {
//         SceneSerializedComponents::RCollider => {
//             bincode::deserialize::<Collider>(&data).map(|d| DeRes::RCollider(d))
//         }
//     }
// }
