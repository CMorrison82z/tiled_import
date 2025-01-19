use bevy::prelude::Component;
use bincode::ErrorKind;

use crate::types::{SceneSerializedComponents, Serialized};


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
