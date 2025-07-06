use bevy::ecs::spawn::{SpawnRelatedBundle, SpawnableList, SpawnIter};
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::transform::components::Transform;

use bevy_platform::collections::HashMap;
use try_match::match_ok;
use bevy_rapier2d::prelude::*;
use bincode::ErrorKind;

use crate::types::{SceneSerializedComponents, SerializedComponents};
use tiled_parse::types::*;

// TODO:
// Likely can be made more general and convenient
pub fn deserialize_collider(b: &[u8]) -> Result<Collider, Box<ErrorKind>> {
    bincode::deserialize::<Collider>(b)
}

pub fn object_collider(
    Object {
        position: (x, y),
        size,
        rotation,
        otype,
        ..
    }: &Object,
) -> Option<impl Bundle> {
    let gt = match_ok!(otype, ObjectType::Geometry(gt))?;

    // TODO:
    // Maybe point colliders are useful ?
    if matches!(gt, GeometryType::Point) {
        return None;
    }

    let (
        Vec2 {
            x: offset_x,
            y: offset_y,
        },
        collider,
    ) = construct_geometry(gt, size.map(|(x, y)| Vec2 { x, y }), None);

    Some((
        Transform::from_xyz(*x + offset_x, -(*y + offset_y), 0.)
            .with_rotation(Quat::from_axis_angle(Vec3::Z, rotation.to_radians())),
        SerializedComponents(HashMap::from([(
            SceneSerializedComponents::SerCollider,
            bincode::serialize(&collider).unwrap(),
        )])),
    ))
}

pub fn object_colliders(os: &Vec<Object>) -> SpawnRelatedBundle<ChildOf, impl SpawnableList<ChildOf>> {
    Children::spawn(SpawnIter(
        os.iter()
            .filter_map(|o|
                if let Some(TiledPropertyType::Bool(true)) = o.properties.get("collider") {
                    object_collider(o)
                } else {
                    None
                }
            )
            .collect::<Vec<_>>()
            .into_iter()
    ))
}

fn construct_geometry(
    shape: &GeometryType,
    size: Option<Vec2>,
    scale_factor: Option<Vect>,
) -> (Vec2, Collider) {
    let scale_factor = scale_factor.unwrap_or(Vect::ONE);

    match shape {
        GeometryType::Rectangle => {
            let Some(size) = size else { unreachable!() };
            (
                size / 2. * scale_factor,
                Collider::cuboid(scale_factor.x * size.x / 2., scale_factor.y * size.y / 2.),
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
                Collider::ball(size.x / 2.),
            )
        }
        GeometryType::Polygon(points) => (
            Vec2::ZERO,
            Collider::convex_hull(
                &points
                    .iter()
                    .map(|(x, y)| Vect::new(scale_factor.x * *x, scale_factor.y * -*y))
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
