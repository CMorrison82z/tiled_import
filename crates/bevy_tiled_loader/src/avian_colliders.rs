use bevy::math::{UVec2, Vec2};
use bevy::prelude::*;
use bevy::transform::components::{Transform};

use avian2d::prelude::*;
use bincode::ErrorKind;

use crate::types::{SceneSerializedComponents, Serialized, TiledMapAsset, TiledMapContainer};
use tiled_parse::data_types::*;

// TODO:
// Likely can be made more general and convenient
pub fn deserialize_collider(b: &[u8]) -> Result<Collider, Box<ErrorKind>> {
    bincode::deserialize::<Collider>(b)
}

pub fn add_colliders(e: &mut EntityWorldMut, os: &Vec<Object>) {
    e.with_children(|cb| {
        os.iter()
            .filter(|o| {
                if let Some(TiledPropertyType::Bool(v)) = o.properties.get("collider") {
                    *v
                } else {
                    false
                }
            })
            .for_each(
                |Object {
                     position: (x, y),
                     size,
                     rotation,
                     tile_global_id,
                     visible,
                     otype,
                     properties,
                     ..
                 }| {
                    if let ObjectType::Point = otype {
                        return;
                    }

                    let (
                        Vec2 {
                            x: offset_x,
                            y: offset_y,
                        },
                        collider,
                    ) = construct_geometry(&otype, size.map(|(x, y)| Vec2 { x, y }), None);

                    cb.spawn((
                        TransformBundle::from_transform(
                            Transform::from_xyz(*x + offset_x, -(*y + offset_y), 0.).with_rotation(
                                Quat::from_axis_angle(Vec3::Z, rotation.to_radians()),
                            ),
                        ),
                        Serialized {
                            data: bincode::serialize(&collider)
                                .expect("Expected to serialize collider"),
                            thingy: SceneSerializedComponents::SerCollider,
                        },
                    ));
                },
            )
    });
}

fn construct_geometry(
    shape: &ObjectType,
    size: Option<Vec2>,
    scale_factor: Option<Vec2>,
) -> (Vec2, Collider) {
    let scale_factor = scale_factor.unwrap_or(Vec2::ONE);

    match shape {
        ObjectType::Rectangle => {
            let Some(size) = size else { unreachable!() };
            (
                size / 2. * scale_factor,
                Collider::rectangle(scale_factor.x * size.x,  scale_factor.y * size.y),
            )
        }
        ObjectType::Ellipse => {
            let Some(size) = size else { unreachable!() };

            (
                // TODO:
                // Determine correct offset.
                size / 2. * scale_factor,
                Collider::ellipse(scale_factor.x * size.x / 2., scale_factor.y * size.y / 2.)
            )
        }
        ObjectType::Polygon(points) => (
            Vec2::ZERO,
            Collider::convex_hull(
                points
                    .iter()
                    .map(|(x, y)| Vec2::new(scale_factor.x * *x, scale_factor.y * -*y))
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        ),
        ObjectType::Polyline(points) => (
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
