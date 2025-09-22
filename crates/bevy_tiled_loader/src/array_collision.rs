/// WARN:
/// DO NOT DELETE ! USEFUL FUNCTIONS `get_centered_rings` AND OTHERS BELOW !

use std::f32::consts::PI;

#[cfg(feature = "avian2d_colliders")]
use avian2d::prelude::Collider;
use bevy::math::{bounding::Aabb2d, IVec2, UVec2, Vec2};
use ndarray::Array2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TileShape {
    Square,
    /// A `Right - Triangle`
    FullTriangle(TriangleOrientation),
    HalfTriangle(TriangleOrientation),
}

/// Refering to the corner of the right angle of the Triangle.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TriangleOrientation {
    /// Right angle is located at the top-left corner of the tile.
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub type ArrayCollider = Array2<Option<TileShape>>;

pub struct MapCollision {
    pub collider: ArrayCollider,
    pub meter: f32
}

impl MapCollision {
    pub fn is_colliding(&self, point: Vec2) -> bool {
        let UVec2 { x, y } = point.as_uvec2();

        self.collider.get((x as usize, y as usize)).flatten().is_some()
    }
    /// NOTE:
    /// The Aabb extents get cast to u32.
    /// If this gets used, you'll like want to use it in `parry2d::query::cast_shapes`.
    /// TODO:
    /// This is a physics crate specific function... unless I use parry2d types maybe ?
    pub fn get_polygon_in(&self, Aabb2d { min, max }: Aabb2d) -> Collider {
        let (min, max) = (min.as_uvec2(), max.as_uvec2());
        todo!()
        // TODO:
        // Try to generate some kind of polygon that gits within the bounds provided by the
        // caller...
    }
    // TODO:
    // pub fn shortest_vector_to_edge(&self, point: Vec2) -> Vec2 {
    //     let UVec2 {x, y} = point.as_uvec2();
    //
    //     (0..).iter().find_map(|r| {
    //         // Find first tile that is empty (this indicates the edge is near.)
    //         let empty_point = get_ring(r).find(|p| {
    //             self.collider.get((x, y)).flatten().is_none()
    //         })?;
    //     })
    // }
}

#[cfg(feature = "avian2d_colliders")]
impl TileShape {
    pub fn into_collider(&self, meter: f32) -> Collider {
        let half_meter = meter / 2.;

        match self {
            TileShape::Square => Collider::rectangle(meter, meter),
            TileShape::FullTriangle(x) => Collider::triangle(
                Vec2::from_angle(x.into_angle()).rotate(Vec2 { x: half_meter, y: half_meter }),
                Vec2::from_angle(x.into_angle()).rotate(Vec2 { x: - half_meter, y: half_meter }),
                Vec2::from_angle(x.into_angle()).rotate(Vec2 { x: - half_meter, y: - half_meter })
            ),
            TileShape::HalfTriangle(x) => Collider::triangle(
                Vec2::from_angle(x.into_angle()).rotate(Vec2 { x: half_meter, y: half_meter / 2. }),
                Vec2::from_angle(x.into_angle()).rotate(Vec2 { x: - half_meter, y: half_meter / 2. }),
                Vec2::from_angle(x.into_angle()).rotate(Vec2 { x: - half_meter, y: - half_meter / 2. })
            )
        }
    }
}

impl TriangleOrientation {
    pub fn into_angle(&self) -> f32 {
        match self {
            TriangleOrientation::TopLeft => 0.,
            TriangleOrientation::TopRight => PI / 2.,
            TriangleOrientation::BottomLeft => PI,
            TriangleOrientation::BottomRight =>  3. * PI / 2.,
        }
    }
}

/// Returns rings with radius 1, 3, 5, ...
pub fn get_centered_rings() -> impl Iterator<Item = IVec2> {
    (1..).step_by(2).flat_map(get_centered_ring)
}

pub fn get_centered_ring(diameter: u32) -> impl Iterator<Item = IVec2> {
    let radius = (diameter / 2) as i32;

    [
        (IVec2 { x: 1, y: 0 }, IVec2 { x: 0, y: -1 }),
        (IVec2 { x: 0, y: 1 }, IVec2 { x: 1, y: 0 }),
        (IVec2 { x: 1, y: 0 }, IVec2 { x: 0, y: 1 }),
        (IVec2 { x: 0, y: 1 }, IVec2 { x: -1, y: 0 }),
    ]
    .iter()
    .flat_map(move |(a, b)| (-radius..=radius).map(move |x| x * a + radius * b))
}

/// Rings corner is located at the origin, i.e. NOT centered.
pub fn get_ring(diameter: u32) -> impl Iterator<Item = UVec2> {
    // TODO:
    // I'm sure there's an interesting relationship between some value the `UVec2`s
    // defined in this initial slice. (Generate these pairs of vectors from some single number.
    // Imaginary numbers / rotors ?)
    [
        (UVec2 { x: 1, y: 0 }, UVec2 { x: 0, y: 0 }),
        (UVec2 { x: 0, y: 1 }, UVec2 { x: 1, y: 0 }),
        (UVec2 { x: 1, y: 0 }, UVec2 { x: 0, y: 1 }),
        (UVec2 { x: 0, y: 1 }, UVec2 { x: 0, y: 0 }),
    ]
    .iter()
    .flat_map(move |(a, b)| (0..diameter).map(move |x| x * a + diameter * b))
}
