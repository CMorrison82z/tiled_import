use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::transform::components::Transform;

use avian2d::prelude::*;
use bevy_platform::collections::HashMap;
use bincode::ErrorKind;
#[cfg(feature = "tilemap_collider")]
use i_overlay::{i_float::float::point::FloatPoint, i_shape::base::data::Shapes};
use try_match::match_ok;

use crate::types::{SceneSerializedComponents, SerializedComponents};
use tiled_parse::types::*;

// TODO:
// Likely can be made more general and convenient
pub(crate) fn deserialize_collider(b: &[u8]) -> Result<Collider, Box<ErrorKind>> {
    bincode::deserialize::<Collider>(b)
}

// TODO: Use `as` pattern on Object
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
    ) = construct_geometry(&gt, size.map(|(x, y)| Vec2 { x, y }), None);

    Some((
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
    ))
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

#[cfg(feature = "tilemap_collider")]
pub fn shapes_to_collider(s: Shapes<FloatPoint<f32>>) -> impl Bundle {
    use i_triangle::float::triangulation::Triangulation;
    use i_triangle::float::triangulatable::Triangulatable;

    let delaunay_triangulation: Triangulation<FloatPoint<f32>, u32> =
        s.triangulate().into_delaunay().to_triangulation();

    let (points, edges) = dbg!((
        delaunay_triangulation.points.into_iter().map(|FloatPoint { x, y }| Vec2 {x, y}).collect(),
        delaunay_triangulation.indices.chunks(3).flat_map(|s| match s {
            &[a, b, c] => vec![[a, b], [b, c], [a, c]],
            _ => unreachable!()
        }).collect(),
    ));

    (
        SerializedComponents(HashMap::from([
            (
                SceneSerializedComponents::SerCollider,
                bincode::serialize(&Collider::convex_decomposition(points, edges)).unwrap(),
            ),
            // TODO:
            // Parse a tiled property to allow other RigidBody types
            (
                SceneSerializedComponents::SerRigidBody,
                bincode::serialize(&RigidBody::Static).unwrap(),
            ),
        ])),
        Transform::default()
    )
// pub fn shapes_to_colliders(ss: Vec<Contour<Vec2>>) -> Vec<Collider> {
// ss.into_iter().map(|s| {
//     Collider::convex_hull(s).unwrap()
// }).collect()
}
