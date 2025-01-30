use bevy::math::{UVec2, Vec2};
use bevy::prelude::*;
use bevy::transform::components::Transform;

use avian2d::prelude::*;
use bevy::utils::HashMap;
use bincode::ErrorKind;

use crate::types::{SceneSerializedComponents, SerializedComponents};
use tiled_parse::data_types::*;

// TODO:
// Likely can be made more general and convenient
pub(crate) fn deserialize_collider(b: &[u8]) -> Result<Collider, Box<ErrorKind>> {
    bincode::deserialize::<Collider>(b)
}

// TODO: Use `as` pattern on Object
pub(crate) fn insert_collider(
    e: &mut EntityWorldMut,
    Object {
        position: (x, y),
        size,
        rotation,
        otype,
        ..
    }: &Object,
) {
    let ObjectType::Geometry(gt) = otype else {
        return;
    };

    // TODO:
    // Maybe point colliders are useful ?
    if let GeometryType::Point = gt {
        return;
    }

    let (
        Vec2 {
            x: offset_x,
            y: offset_y,
        },
        collider,
    ) = construct_geometry(&gt, size.map(|(x, y)| Vec2 { x, y }), None);

    e.insert((
        Transform::from_xyz(*x + offset_x, -(*y + offset_y), 0.)
            .with_rotation(Quat::from_axis_angle(Vec3::Z, rotation.to_radians())),
        SerializedComponents(HashMap::from([
            (
                SceneSerializedComponents::SerCollider,
                bincode::serialize(&collider).unwrap(),
            ),
            // TODO:
            // Parse a tiled property to allow other RigidBody types
            (
                SceneSerializedComponents::SerRigidBody,
                bincode::serialize(&RigidBody::Static).unwrap(),
            ),
        ])),
    ));
}

pub(crate) fn add_child_colliders(e: &mut EntityWorldMut, os: &Vec<Object>) {
    e.with_children(|cb| {
        os.iter()
            .filter(|o| {
                if let Some(TiledPropertyType::Bool(v)) = o.properties.get("collider") {
                    *v
                } else {
                    false
                }
            })
            .for_each(|o| insert_collider(&mut cb.spawn_empty(), o))
    });
}

fn construct_geometry(
    shape: &GeometryType,
    size: Option<Vec2>,
    scale_factor: Option<Vec2>,
) -> (Vec2, Collider) {
    let scale_factor = scale_factor.unwrap_or(Vec2::ONE);

    match shape {
        GeometryType::Rectangle => {
            let Some(size) = size else { unreachable!() };
            (
                size / 2. * scale_factor,
                Collider::rectangle(scale_factor.x * size.x, scale_factor.y * size.y),
            )
        }
        GeometryType::Ellipse => {
            let Some(size) = size else { unreachable!() };

            if size.x != size.y {
                panic!("Ellipses cannot be serialized, so therefore cannot be constructed. To construct a circle, make sure length and width are the same.")
            };

            (
                // TODO:
                // Determine correct offset.
                size / 2. * scale_factor,
                Collider::circle(size.x / 2.),
            )
        }
        GeometryType::Polygon(points) => (
            Vec2::ZERO,
            Collider::convex_hull(
                points
                    .iter()
                    .map(|(x, y)| Vec2::new(scale_factor.x * *x, scale_factor.y * -*y))
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        GeometryType::Polyline(points) => (
            Vec2::ZERO,
            Collider::polyline(
                points
                    .iter()
                    .map(|p| Vec2 {
                        x: scale_factor.x * p.0,
                        y: scale_factor.y * -p.1,
                    })
                    .collect(),
                None,
            ),
        ),
        _ => todo!(),
    }
}
