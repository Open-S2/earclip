#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! The `earclip` module is a tool to convert 2D and 3D polygons into a triangle mesh designed to be
//! fast, efficient, and sphere capable.
//!
//! Basic usage:
//! ```rust
//! use earclip::earclip;
//!
//! let polygon = vec![vec![vec![0.0, 0.0, 0.0], vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0]]];
//! let (vertices, indices) = earclip(&polygon, None, None);
//! assert_eq!(vertices, vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
//! assert_eq!(indices, vec![1, 2, 0]);
//! ```

extern crate alloc;

/// The `earcut` module
pub mod earcut;

use alloc::vec::Vec;
use core::f64;
pub use earcut::{earcut, signed_area};
use libm::fabs;

/// A trait that must be implemented by any type that wants to represent a 2D point
pub trait Point2D {
    /// Get the x-coordinate
    fn x(&self) -> f64;
    /// Get the y-coordinate
    fn y(&self) -> f64;
}

/// A trait that must be implemented by any type that wants to represent a 3D point
pub trait Point3D {
    /// Get the x-coordinate
    fn x(&self) -> f64;
    /// Get the y-coordinate
    fn y(&self) -> f64;
    /// Get the z-coordinate
    fn z(&self) -> f64;
}

/// An earcut polygon generator with tesselation support
pub fn earclip(
    polygon: &[Vec<Vec<f64>>],
    modulo: Option<f64>,
    offset: Option<usize>,
) -> (Vec<f64>, Vec<usize>) {
    let modulo = modulo.unwrap_or(f64::INFINITY);
    let offset = offset.unwrap_or(0);

    // Use earcut to build standard triangle set
    let (mut vertices, hole_indices, dim) = flatten(polygon); // dim => dimensions
    let mut indices = earcut(&vertices, &hole_indices, dim);

    // tesselate if necessary
    if modulo != f64::INFINITY {
        tesselate(&mut vertices, &mut indices, modulo, 2);
    }

    // update offsets
    indices = indices.into_iter().map(|index| index + offset).collect();

    (vertices, indices)
}

/// Tesselates the flattened polygon
pub fn tesselate(vertices: &mut Vec<f64>, indices: &mut Vec<usize>, modulo: f64, dim: usize) {
    for axis in 0..dim {
        let mut i = 0;
        while i < indices.len() {
            let a = indices[i];
            let b = indices[i + 1];
            let c = indices[i + 2];

            if let Some(new_triangle) =
                split_if_necessary(a, b, c, vertices, indices, dim, axis, modulo)
            {
                indices[i] = new_triangle[0];
                indices[i + 1] = new_triangle[1];
                indices[i + 2] = new_triangle[2];
                i -= 3;
            }

            i += 3;
        }
    }
}

/// given vertices, and an axis of said vertices:
/// find a number "x" that is x % modulo == 0 and between v1 and v2
#[allow(clippy::too_many_arguments)]
fn split_if_necessary(
    i1: usize,
    i2: usize,
    i3: usize,
    vertices: &mut Vec<f64>,
    indices: &mut Vec<usize>,
    dim: usize,
    axis: usize,
    modulo: f64,
) -> Option<[usize; 3]> {
    let v1 = vertices[i1 * dim + axis];
    let v2 = vertices[i2 * dim + axis];
    let v3 = vertices[i3 * dim + axis];
    // 1 is corner
    if v1 < v2 && v1 < v3 {
        let mod_point = v1 + modulo - mod2(v1, modulo);
        if mod_point > v1 && mod_point <= v2 && mod_point <= v3 && v2 != mod_point {
            return Some(split_right(
                mod_point, i1, i2, i3, v1, v2, v3, vertices, indices, dim, axis, modulo,
            ));
        }
    } else if v1 > v2 && v1 > v3 {
        let mut m2 = mod2(v1, modulo);
        if m2 == 0. {
            m2 = modulo;
        }
        let mod_point = v1 - m2;
        if mod_point < v1 && mod_point >= v2 && mod_point >= v3 && v2 != mod_point {
            return Some(split_left(
                mod_point, i1, i2, i3, v1, v2, v3, vertices, indices, dim, axis, modulo,
            ));
        }
    }
    // 2 is corner
    if v2 < v1 && v2 < v3 {
        let mod_point = v2 + modulo - mod2(v2, modulo);
        if mod_point > v2
            && mod_point <= v3
            && mod_point <= v1
            && (v1 != mod_point || v3 != mod_point)
        {
            return Some(split_right(
                mod_point, i2, i3, i1, v2, v3, v1, vertices, indices, dim, axis, modulo,
            ));
        }
    } else if v2 > v1 && v2 > v3 {
        let mut m2 = mod2(v2, modulo);
        if m2 == 0. {
            m2 = modulo;
        }
        let mod_point = v2 - m2;
        if mod_point < v2
            && mod_point >= v3
            && mod_point >= v1
            && (v1 != mod_point || v3 != mod_point)
        {
            return Some(split_left(
                mod_point, i2, i3, i1, v2, v3, v1, vertices, indices, dim, axis, modulo,
            ));
        }
    }
    // 3 is corner
    if v3 < v1 && v3 < v2 {
        let mod_point = v3 + modulo - mod2(v3, modulo);
        if mod_point > v3
            && mod_point <= v1
            && mod_point <= v2
            && (v1 != mod_point || v2 != mod_point)
        {
            return Some(split_right(
                mod_point, i3, i1, i2, v3, v1, v2, vertices, indices, dim, axis, modulo,
            ));
        }
    } else if v3 > v1 && v3 > v2 {
        let mut m2 = mod2(v3, modulo);
        if m2 == 0. {
            m2 = modulo;
        }
        let mod_point = v3 - m2;
        if mod_point < v3
            && mod_point >= v1
            && mod_point >= v2
            && (v1 != mod_point || v2 != mod_point)
        {
            return Some(split_left(
                mod_point, i3, i1, i2, v3, v1, v2, vertices, indices, dim, axis, modulo,
            ));
        }
    }

    None
}

/// Creates a vertex at the specified split point, only manipulating the specified axis
#[allow(clippy::too_many_arguments)]
fn create_vertex(
    split_point: f64,
    i1: usize,
    i2: usize,
    v1: f64,
    v2: f64,
    vertices: &mut Vec<f64>,
    dim: usize,
    axis: usize,
) -> usize {
    let index = vertices.len() / dim;
    let travel_divisor = (v2 - v1) / (split_point - v1);
    for i in 0..2 {
        let va1 = vertices[i1 * dim + i];
        let va2 = vertices[i2 * dim + i];
        if i != axis {
            vertices.push(va1 + (va2 - va1) / travel_divisor);
        } else {
            vertices.push(split_point);
        }
    }
    index
}

/// Splits triangles based on a modulo value and adjusts vertices
#[allow(clippy::too_many_arguments)]
fn split_right(
    mod_point: f64,
    i1: usize,
    i2: usize,
    i3: usize,
    v1: f64,
    v2: f64,
    v3: f64,
    vertices: &mut Vec<f64>,
    indices: &mut Vec<usize>,
    dim: usize,
    axis: usize,
    modulo: f64,
) -> [usize; 3] {
    let mut mod_point = mod_point;
    // Creating the first set of split vertices
    let mut i12 = create_vertex(mod_point, i1, i2, v1, v2, vertices, dim, axis);
    let mut i13 = create_vertex(mod_point, i1, i3, v1, v3, vertices, dim, axis);
    indices.extend([i1, i12, i13]);

    mod_point += modulo;

    if v2 < v3 {
        while mod_point < v2 {
            indices.extend([i13, i12]);
            i13 = create_vertex(mod_point, i1, i3, v1, v3, vertices, dim, axis);
            indices.extend([i13, i13, i12]);
            i12 = create_vertex(mod_point, i1, i2, v1, v2, vertices, dim, axis);
            indices.push(i12);
            mod_point += modulo;
        }
        indices.extend([i13, i12, i2]);
        [i13, i2, i3]
    } else {
        while mod_point < v3 {
            indices.extend([i13, i12]);
            i13 = create_vertex(mod_point, i1, i3, v1, v3, vertices, dim, axis);
            indices.extend([i13, i13, i12]);
            i12 = create_vertex(mod_point, i1, i2, v1, v2, vertices, dim, axis);
            indices.push(i12);
            mod_point += modulo;
        }
        indices.extend([i13, i12, i3]);
        [i3, i12, i2]
    }
}

/// Splits triangles based on a modulo value and adjusts vertices
#[allow(clippy::too_many_arguments)]
fn split_left(
    mod_point: f64,
    i1: usize,
    i2: usize,
    i3: usize,
    v1: f64,
    v2: f64,
    v3: f64,
    vertices: &mut Vec<f64>,
    indices: &mut Vec<usize>,
    dim: usize,
    axis: usize,
    modulo: f64,
) -> [usize; 3] {
    let mut mod_point = mod_point;
    // first case is a standalone triangle
    let mut i12 = create_vertex(mod_point, i1, i2, v1, v2, vertices, dim, axis);
    let mut i13 = create_vertex(mod_point, i1, i3, v1, v3, vertices, dim, axis);
    indices.extend([i1, i12, i13]);
    mod_point -= modulo;
    if v2 > v3 {
        // create lines up to i2
        while mod_point > v2 {
            // next triangles are i13->i12->nexti13 and nexti13->i12->nexti12 so store in necessary order
            indices.extend([i13, i12]);
            i13 = create_vertex(mod_point, i1, i3, v1, v3, vertices, dim, axis);
            indices.extend([i13, i13, i12]);
            i12 = create_vertex(mod_point, i1, i2, v1, v2, vertices, dim, axis);
            indices.push(i12);
            // increment
            mod_point -= modulo;
        }
        // add v2 triangle if necessary
        indices.extend([i13, i12, i2]);
        // return the remaining triangle
        [i13, i2, i3]
    } else {
        // create lines up to i2
        while mod_point > v3 {
            // next triangles are i13->i12->nexti13 and nexti13->i12->nexti12 so store in necessary order
            indices.extend([i13, i12]);
            i13 = create_vertex(mod_point, i1, i3, v1, v3, vertices, dim, axis);
            indices.extend([i13, i13, i12]);
            i12 = create_vertex(mod_point, i1, i2, v1, v2, vertices, dim, axis);
            indices.push(i12);
            // increment
            mod_point -= modulo;
        }
        // add v3 triangle if necessary
        indices.extend([i13, i12, i3]);
        // return the remaining triangle
        [i3, i12, i2]
    }
}

/// Returns x modulo n (supports negative numbers)
fn mod2(x: f64, n: f64) -> f64 {
    ((x % n) + n) % n
}

/// Flattens a 2D or 3D array whether its a flat point ([x, y, z]) or object ({ x, y, z })
pub fn flatten(data: &[Vec<Vec<f64>>]) -> (Vec<f64>, Vec<usize>, usize) {
    let mut vertices = Vec::new();
    let mut hole_indices = Vec::new();
    let mut hole_index = 0;
    let mut dim = 2; // Assume 2D unless a 3D point is found

    for (i, line) in data.iter().enumerate() {
        for point in line {
            vertices.push(point[0]);
            vertices.push(point[1]);
            // Check if it's a 3D point
            if point.len() > 2 {
                vertices.push(point[2]);
                dim = 3; // Update dimensionality to 3D if any point is 3D
            }
        }
        if i > 0 {
            hole_index += data[i - 1].len();
            hole_indices.push(hole_index);
        }
    }

    (vertices, hole_indices, dim)
}

/// Convert a structure into a 2d vector array
pub fn convert_2d<P: Point2D>(data: &[Vec<P>]) -> Vec<Vec<Vec<f64>>> {
    data.iter()
        .map(|line| line.iter().map(|point| Vec::from([point.x(), point.y()])).collect())
        .collect()
}

/// Convert a structure into a 3d vector array
pub fn convert_3d<P: Point3D>(data: &[Vec<P>]) -> Vec<Vec<Vec<f64>>> {
    data.iter()
        .map(|line| line.iter().map(|point| Vec::from([point.x(), point.y(), point.z()])).collect())
        .collect()
}

/// Returns a percentage difference between the polygon area and its triangulation area;
/// used to verify correctness of triangulation
pub fn deviation(data: &[f64], hole_indices: &[usize], triangles: &[usize], dim: usize) -> f64 {
    let has_holes = !hole_indices.is_empty();
    let outer_len = if has_holes { hole_indices[0] * dim } else { data.len() };
    let mut polygon_area = fabs(signed_area(data, 0, outer_len, dim));

    if has_holes {
        for i in 0..hole_indices.len() {
            let start = hole_indices[i] * dim;
            let end =
                if i < hole_indices.len() - 1 { hole_indices[i + 1] * dim } else { data.len() };
            polygon_area -= fabs(signed_area(data, start, end, dim));
        }
    }

    let mut triangles_area = 0.;
    let mut i = 0;
    while i < triangles.len() {
        let a = triangles[i] * dim;
        let b = triangles[i + 1] * dim;
        let c = triangles[i + 2] * dim;
        triangles_area += fabs(
            (data[a] - data[c]) * (data[b + 1] - data[a + 1])
                - (data[a] - data[b]) * (data[c + 1] - data[a + 1]),
        );
        i += 3;
    }

    let zero = 0.;
    if polygon_area == zero && triangles_area == zero {
        zero
    } else {
        fabs((triangles_area - polygon_area) / polygon_area)
    }
}
