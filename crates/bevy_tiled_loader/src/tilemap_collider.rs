use std::f32::consts::PI;

use bevy::math::Vec2;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::single::SingleFloatOverlay;
use i_overlay::float::slice::FloatSlice;
use i_overlay::i_float::float::point::FloatPoint;
use i_overlay::i_shape::base::data::{Contour, Shape, Shapes};
use itertools::{Either, Itertools};
use tiled_parse::types::*;

#[derive(Clone, PartialEq, Debug)]
pub enum TileShape {
    Square,
    /// A `Right - Triangle`
    Triangle(TriangleOrientation),
    /// NOTE: The points in the contour should be local.
    Polygon(Contour<Vec2>),
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

/// NOTE: `TileShape` is expected to be in local coordinates.
/// The generated shapes all consist of a single contour.
// pub fn gen_shapes(meter: f32, ts: Array2<Option<TileShape>>) -> Vec<Contour<Vec2>> {
pub fn gen_shapes<T>(
    meter: f32,
    ts: T,
) -> Shapes<FloatPoint<f32>>
//) -> Vec<Contour<Vec2>>
where
    T: Iterator<Item = ((usize, usize), Option<TileShape>)>
{
    let (xs, ys): (Shapes<FloatPoint<f32>>, Shapes<FloatPoint<f32>>) =
        ts.partition_map(|(ix, s)| {
            if let Some(s) = s {
                Either::Left(vec![tile_shape_contour(meter, ix, &s)])
            } else {
                Either::Right(vec![tile_shape_contour(meter, ix, &TileShape::Square)])
            }
        });

    xs.overlay(&ys, OverlayRule::Difference, FillRule::Positive)
        // .into_iter()
        // .flat_map(|s| {
        //     slice_holes(s)
        //         .into_iter()
        //         .map(|mut s2| {
        //             std::mem::take(&mut s2[0])
        //                 .into_iter()
        //                 .map(|v| float_point_to_vec2(v))
        //                 .collect::<Vec<_>>()
        //         })
        //         .collect::<Vec<_>>()
        // })
        // .collect()
    // r.iter().for_each(|c| {
    //     println!("cont : {}\n", c.iter().map(|Vec2 {x, y}| format!("{x},{y} ")).collect::<String>());
    // });
    //
    // r
}

pub fn float_point_to_vec2(FloatPoint { x, y }: FloatPoint<f32>) -> Vec2 {
    Vec2 { x, y }
}

pub fn tile_shape_contour(
    meter: f32,
    tile_pos: (usize, usize),
    s: &TileShape,
) -> Contour<FloatPoint<f32>> {
    // NOTE: Cannot factor out `contextualize` because match arms have incompatible types...
    match s {
        TileShape::Square => contextualize(meter, tile_pos, box_contour()).collect(),
        TileShape::Triangle(o) => {
            contextualize(meter, tile_pos, right_triangle_contour(*o)).collect()
        }
        TileShape::Polygon(vs) => contextualize(meter, tile_pos, vs.iter().map(|x| *x)).collect(),
    }
}

pub fn contextualize<I: IntoIterator<Item = Vec2>>(
    meter: f32,
    (ix_x, ix_y): (usize, usize),
    vs: I,
) -> impl Iterator<Item = FloatPoint<f32>> {
    let r = Vec2 {
        x: ix_x as f32,
        y: ix_y as f32,
    };

    vs.into_iter().map(move |v| {
        let Vec2 { x, y } = meter * (r + v);
        FloatPoint { x, y }
    })
}

pub fn box_contour() -> impl Iterator<Item = Vec2> {
    // NOTE: This is CCW with y-up, which is correct.
    [
        Vec2 { x: 0., y: 0. },
        Vec2 { x: 1., y: 0. },
        Vec2 { x: 1., y: 1. },
        Vec2 { x: 0., y: 1. },
    ]
    .into_iter()
}

/// Ellipse divisions down to a pixel.
/// Circum = 2 PI r
/// arc-length : `s = r theta`
/// density = N / l & [density] = point / pixel
/// Therefore `arc_vertices = density s = density (r theta)`, where `s` is the arc-length, defined above.
/// A unit circle is defined to have a density of 1.
/// density = 1 / r.
/// Returns an ellipse with the major axis length 1
/// NOTE:
/// The ellipse is offset such that the center of a unit circle would be located at `(1., 1.)`
/// as this is typical expectation in tile-based projects to have shape origins at top-left.
/// WARN:
/// The ellipse is NOT scaled by `meter`. Meter is just used to determine the granularity.
/// FIXME:
/// Vertices are not evenly distributed along the arc of the ellipse...
pub fn ellipse_contour(meter: f32, r_x: f32, r_y: f32) -> impl Iterator<Item = Vec2> {
    // For a circle, it would be `meter * r.sqrt() * 2 * PI`. We remove the factor of 2,
    // and add the other axis' radius (imagine the case where `r_x == r_y`, then
    // `r.sqrt() + r.sqrt() = 2 * r.sqrt()`; the factor of 2 is recovered.
    let arc_vertices = (meter * (r_x.sqrt() + r_y.sqrt()) * PI).round() as usize;

    (0..arc_vertices)
        .into_iter()
        .map(move |i| {
            let p = i as f32 / (arc_vertices as f32);
            Vec2 { x: r_x, y: r_y } * Vec2::from_angle(p * 2. * PI) + Vec2::ONE
        })
}

#[test]
fn check_ellipse_contour() {
    let circle : Vec<_> = ellipse_contour(16., 1., 1.).collect();

    // Check that all of the unit circle's vertices lie within the expected range.
    assert!(circle.iter().all(|x| 0. <= x.length() && x.length() <= 2. ));
}

pub fn right_triangle_contour(orientaton: TriangleOrientation) -> impl Iterator<Item = Vec2> {
    [
        Vec2::from_angle(orientaton.into_angle()).rotate(Vec2 { x: - 0.5, y: - 0.5 }),
        Vec2::from_angle(orientaton.into_angle()).rotate(Vec2 { x: 0.5, y: - 0.5 }),
        Vec2::from_angle(orientaton.into_angle()).rotate(Vec2 { x: - 0.5, y: 0.5 }),
    ]
    .into_iter().map(|v| v + Vec2::splat(0.5))
}

impl TriangleOrientation {
    pub fn into_angle(&self) -> f32 {
        match self {
            TriangleOrientation::TopLeft => 0.,
            TriangleOrientation::TopRight => PI / 2.,
            TriangleOrientation::BottomRight => PI,
            TriangleOrientation::BottomLeft => 3. * PI / 2.
        }
    }
    pub fn try_from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "topleft" => Some(TriangleOrientation::TopLeft),
            "topright" => Some(TriangleOrientation::TopRight),
            "bottomleft" => Some(TriangleOrientation::BottomLeft),
            "bottomright" => Some(TriangleOrientation::BottomRight),
            _ => None
        }
    }
    pub fn flip_x(self) -> Self {
        match self {
            TriangleOrientation::TopLeft => Self::TopRight,
            TriangleOrientation::TopRight => Self::TopLeft,
            TriangleOrientation::BottomLeft => Self::BottomRight,
            TriangleOrientation::BottomRight => Self::BottomLeft
        }
    }
    pub fn flip_y(self) -> Self {
        match self {
            TriangleOrientation::TopLeft => Self::BottomLeft,
            TriangleOrientation::TopRight => Self::BottomRight,
            TriangleOrientation::BottomLeft => Self::TopLeft,
            TriangleOrientation::BottomRight => Self::TopRight
        }
    }
}

impl TileShape {
    pub fn flip_x(self) -> Self {
        match self {
            TileShape::Triangle(o) => TileShape::Triangle(o.flip_x()),
            TileShape::Polygon(vs) => TileShape::Polygon(vs.into_iter().rev().map(|Vec2 {x, y}| {
                Vec2 {x: 1. - x, y}
            }).collect()),
            x => x
        }
    }
    pub fn flip_y(self) -> Self {
        match self {
            TileShape::Triangle(o) => TileShape::Triangle(o.flip_y()),
            TileShape::Polygon(vs) => TileShape::Polygon(vs.into_iter().rev().map(|Vec2 {x, y}| {
                Vec2 {x, y: 1. - y}
            }).collect()),
            x => x
        }
    }
}

// WARN:
// Assumes that all contours after the first are holes...
pub fn slice_holes(s: Shape<FloatPoint<f32>>) -> Shapes<FloatPoint<f32>> {
    let (min, max) = contour_aabb(&s[0]);

    let slice_lines: Vec<_> = s
        .iter()
        .skip(1)
        .map(|h_c| {
            vec![
                FloatPoint {
                    x: min.x,
                    y: h_c[0].y,
                },
                FloatPoint {
                    x: max.x,
                    y: h_c[0].y,
                },
            ]
        })
        .collect();

    slice_lines
        .iter()
        .fold(vec![s], |acc, line|
            // I don't know why flat_mapping works. I would have expected `i_overlay` to handle
            // multi-shape cuts properly if it's supported.
            acc.iter().flat_map(|s|
                s.slice_by(
                    line,
                    FillRule::Positive
                )
            ).collect())
        // If there are still holes after slicing, recursively call `slice_holes` on that shape.
        .into_iter()
        .flat_map(|s| if s.len() > 1 { slice_holes(s) } else { vec![s] })
        .collect()
}

pub fn contour_width(c: &Contour<FloatPoint<f32>>) -> f32 {
    let (min, max) = contour_aabb(c);
    max.x - min.x
}

pub fn contour_aabb(c: &Contour<FloatPoint<f32>>) -> (FloatPoint<f32>, FloatPoint<f32>) {
    (
        c.iter().fold(
            FloatPoint { x: 0., y: 0. },
            |FloatPoint { x: min_x, y: min_y }, &FloatPoint { x, y }| FloatPoint {
                x: min_x.min(x),
                y: min_y.min(y),
            },
        ),
        c.iter().fold(
            FloatPoint { x: 0., y: 0. },
            |FloatPoint { x: max_x, y: max_y }, &FloatPoint { x, y }| FloatPoint {
                x: max_x.max(x),
                y: max_y.max(y),
            },
        ),
    )
}

pub fn tiled_object_to_tile_shape(meter: f32, o: &Object) -> Option<TileShape> {
    let ObjectType::Geometry(geo_type) = &o.otype else {return None};
    let size = o.size;
    let (pos_x, pos_y) = o.position;

    Some(TileShape::Polygon((match geo_type {
        GeometryType::Rectangle => {
            let (width, height) = size?;
            box_contour().map(|v| v * Vec2 { x: width, y: height } / meter).collect()
        },
        GeometryType::Ellipse => ellipse_contour(meter, size?.0 / 2., size?.1 / 2.).collect(),
        GeometryType::Point => vec![Vec2::ZERO],
        // NOTE:
        // Divided by `meter`, because `contextualize` re-introduces the `meter`...
        GeometryType::Polygon(v) => v.iter().map(|&(x, y)| Vec2 {x, y} / meter).collect(),
        // FIXME:
        // A polyline is not necessarily a closed loop. However, the `Contour`s assume it is
        // closed.
        GeometryType::Polyline(v) => v.iter().map(|&(x, y)| Vec2 {x, y} / meter).collect(),
    }).into_iter().map(|v| v + Vec2 {x: pos_x, y: pos_y}).collect()))
}

#[test]
fn doit() {
    use tiled_parse::parse::*;
    use tiled_parse::types::*;

    let data = std::fs::read("assets/test.tmx").unwrap();
    let x = parse_embedded(std::str::from_utf8(&data).unwrap()).unwrap();

    x.layers.iter_breadth().for_each(|y| {
        let LayerType::TileLayer(z) = &y.content else {
            return;
        };
        // let w = z.map(|o| o.map(|_| TileShape::Triangle(TriangleOrientation::TopLeft)));

        gen_shapes(32., z.indexed_iter().map(|(ix, o)| (ix, o.map(|_| TileShape::Square))));
    });

    // TODO:
    // Define this a debug function or something. Basically this just serializes a contour into a
    // Tiled Polygon that can be used as :
    // ```
    // <object id="20" x="0" y="0">
    //  <polygon points="256,288 256,320 160,320 160,288"/>
    // </object>
    // ```
    // r.iter().for_each(|s| if s.len() > 1 {
    //     println!("cont : {}", s.iter().map(|c| c.iter().map(|FloatPoint {x, y}| format!("{x},{y} ")).collect::<String>() + "\n").collect::<String>());
    // });
    panic!();
}
