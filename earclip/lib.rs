#![no_std]
#![deny(missing_docs)]
//! The `earclip` module is a tool to convert 2D and 3D polygons into a triangle mesh designed to be
//! fast, efficient, and sphere capable.
//!
//! Basic usage:
//! ```rust
//! use earclip::earclip;
//!
//! let polygon = vec![vec![vec![0.0, 0.0, 0.0], vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0]]];
//! let (vertices, indices) = earclip::<f64, usize>(&polygon, None, None);
//! assert_eq!(vertices, vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
//! assert_eq!(indices, vec![1, 2, 0]);
//! ```

extern crate alloc;

/// The `earcut` module
pub mod earcut;

use alloc::vec::Vec;
use num_traits::{Float, Unsigned};
pub use earcut::{earcut, signed_area};

/// Index of a vertex
pub trait Index: Copy + Unsigned {
    /// Cast to usize
    fn into_usize(self) -> usize;
    /// Cast from usize
    fn from_usize(v: usize) -> Self;
}
impl Index for u16 {
    fn into_usize(self) -> usize {
        self as usize
    }
    fn from_usize(v: usize) -> Self {
        v as Self
    }
}
impl Index for usize {
    fn into_usize(self) -> usize {
        self
    }
    fn from_usize(v: usize) -> Self {
        v as Self
    }
}
impl Index for u32 {
    fn into_usize(self) -> usize {
        self as usize
    }
    fn from_usize(v: usize) -> Self {
        v as Self
    }
}

/// A trait that must be implemented by any type that wants to represent a 2D point
pub trait Point2D<T: Float> {
    /// Get the x-coordinate
    fn x(&self) -> T;
    /// Get the y-coordinate
    fn y(&self) -> T;
}

/// A trait that must be implemented by any type that wants to represent a 3D point
pub trait Point3D<T: Float> {
    /// Get the x-coordinate
    fn x(&self) -> T;
    /// Get the y-coordinate
    fn y(&self) -> T;
    /// Get the z-coordinate
    fn z(&self) -> T;
}
  
/// An earcut polygon generator with tesselation support
pub fn earclip<T: Float, N: Index>(
    polygon: &[Vec<Vec<T>>],
    modulo: Option<T>,
    offset: Option<N>,
) -> (Vec<T>, Vec<N>) {
    let modulo = modulo.unwrap_or(T::infinity());
    let offset = offset.unwrap_or(N::zero());

    // Use earcut to build standard triangle set
    let (mut vertices, hole_indices, dim) = flatten(polygon); // dim => dimensions
    let mut indices = earcut(&vertices, &hole_indices, dim);

    // tesselate if necessary
    if modulo != T::infinity() {
        tesselate(&mut vertices, &mut indices, modulo, 2);
    }

    // update offsets
    indices = indices
        .into_iter()
        .map(|index| index + offset)
        .collect();

    (vertices, indices)
}

/// Tesselates the flattened polygon
pub fn tesselate<T: Float, N: Index>(
    vertices: &mut Vec<T>,
    indices: &mut Vec<N>,
    modulo: T,
    dim: usize,
) {
    for axis in 0..dim.into_usize() {
        let mut i = 0;
        while i < indices.len() {
            let a = indices[i].into_usize();
            let b = indices[i + 1].into_usize();
            let c = indices[i + 2].into_usize();
    
            if let Some(new_triangle) = split_if_necessary(a, b, c, vertices, indices, dim, axis, modulo) {
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
fn split_if_necessary<T: Float, N: Index>(
    i1: usize,
    i2: usize,
    i3: usize,
    vertices: &mut Vec<T>,
    indices: &mut Vec<N>,
    dim: usize,
    axis: usize,
    modulo: T,
) -> Option<[N; 3]> {
    let v1 = vertices[i1 * dim + axis];
    let v2 = vertices[i2 * dim + axis];
    let v3 = vertices[i3 * dim + axis];
    // 1 is corner
    if v1 < v2 && v1 < v3 {
      let mod_point = v1 + modulo - mod2(v1, modulo);
      if mod_point > v1 && mod_point <= v2 && mod_point <= v3 && v2 != mod_point {
        return Some(split_right(mod_point, i1, i2, i3, v1, v2, v3, vertices, indices, dim, axis, modulo));
      }
    } else if v1 > v2 && v1 > v3 {
      let mut m2 = mod2(v1, modulo);
      if m2.is_zero() { m2 = modulo; }
      let mod_point = v1 - m2;
      if mod_point < v1 && mod_point >= v2 && mod_point >= v3 && v2 != mod_point {
        return Some(split_left(mod_point, i1, i2, i3, v1, v2, v3, vertices, indices, dim, axis, modulo));
      }
    }
    // 2 is corner
    if v2 < v1 && v2 < v3 {
      let mod_point = v2 + modulo - mod2(v2, modulo);
      if mod_point > v2 && mod_point <= v3 && mod_point <= v1 && (v1 != mod_point || v3 != mod_point) {
        return Some(split_right(mod_point, i2, i3, i1, v2, v3, v1, vertices, indices, dim, axis, modulo));
      }
    } else if v2 > v1 && v2 > v3 {
      let mut m2 = mod2(v2, modulo);
      if m2.is_zero() { m2 = modulo; }
      let mod_point = v2 - m2;
      if mod_point < v2 && mod_point >= v3 && mod_point >= v1 && (v1 != mod_point || v3 != mod_point) {
        return Some(split_left(mod_point, i2, i3, i1, v2, v3, v1, vertices, indices, dim, axis, modulo));
      }
    }
    // 3 is corner
    if v3 < v1 && v3 < v2 {
      let mod_point = v3 + modulo - mod2(v3, modulo);
      if mod_point > v3 && mod_point <= v1 && mod_point <= v2 && (v1 != mod_point || v2 != mod_point) {
        return Some(split_right(mod_point, i3, i1, i2, v3, v1, v2, vertices, indices, dim, axis, modulo));
      }
    } else if v3 > v1 && v3 > v2 {
      let mut m2 = mod2(v3, modulo);
      if m2.is_zero() { m2 = modulo; }
      let mod_point = v3 - m2;
      if mod_point < v3 && mod_point >= v1 && mod_point >= v2 && (v1 != mod_point || v2 != mod_point) {
        return Some(split_left(mod_point, i3, i1, i2, v3, v1, v2, vertices, indices, dim, axis, modulo));
      }
    }

    None
  }

/// Creates a vertex at the specified split point, only manipulating the specified axis
#[allow(clippy::too_many_arguments)]
fn create_vertex<T: Float>(
    split_point: T,
    i1: usize,
    i2: usize,
    v1: T,
    v2: T,
    vertices: &mut Vec<T>,
    dim: usize,
    axis: usize,
) -> usize {
    let index = vertices.len() / dim;
    let travel_divisor = (v2 - v1) / (split_point - v1);
    for i in 0..2 {
        let va1 = vertices[i1 * dim + i];
        let va2 = vertices[i2 * dim + i];
        if i != axis { vertices.push(va1 + (va2 - va1) / travel_divisor); }
        else { vertices.push(split_point); }
    }
    index
}

/// Splits triangles based on a modulo value and adjusts vertices
#[allow(clippy::too_many_arguments)]
fn split_right<T: Float, N: Index>(
    mod_point: T,
    i1: usize,
    i2: usize,
    i3: usize,
    v1: T,
    v2: T,
    v3: T,
    vertices: &mut Vec<T>,
    indices: &mut Vec<N>,
    dim: usize,
    axis: usize,
    modulo: T,
) -> [N; 3] {
    let mut mod_point = mod_point;
    // Creating the first set of split vertices
    let mut i12 = create_vertex(mod_point, i1, i2, v1, v2,  vertices, dim, axis);
    let mut i13 = create_vertex(mod_point, i1, i3, v1, v3, vertices, dim, axis);
    indices.extend([i1, i12, i13].map(N::from_usize));

    mod_point = mod_point + modulo;

    if v2 < v3 {
        while mod_point < v2 {
            indices.extend([i13, i12].map(N::from_usize));
            i13 = create_vertex(mod_point, i1, i3, v1, v3, vertices, dim, axis);
            indices.extend([i13, i13, i12].map(N::from_usize));
            i12 = create_vertex(mod_point, i1, i2, v1, v2, vertices, dim, axis);
            indices.push(N::from_usize(i12));
            mod_point = mod_point + modulo;
        }
        indices.extend([i13, i12, i2].map(N::from_usize));
        [i13, i2, i3].map(N::from_usize)
    } else {
        while mod_point < v3 {
            indices.extend([i13, i12].map(N::from_usize));
            i13 = create_vertex(mod_point, i1, i3, v1, v3, vertices, dim, axis);
            indices.extend([i13, i13, i12].map(N::from_usize));
            i12 = create_vertex(mod_point, i1, i2, v1, v2, vertices, dim, axis);
            indices.push(N::from_usize(i12));
            mod_point = mod_point + modulo;
        }
        indices.extend([i13, i12, i3].map(N::from_usize));
        [i3, i12, i2].map(N::from_usize)
    }
}

/// Splits triangles based on a modulo value and adjusts vertices
#[allow(clippy::too_many_arguments)]
fn split_left<T: Float, N: Index>(
    mod_point: T,
    i1: usize,
    i2: usize,
    i3: usize,
    v1: T,
    v2: T,
    v3: T,
    vertices: &mut Vec<T>,
    indices: &mut Vec<N>,
    dim: usize,
    axis: usize,
    modulo: T,
) -> [N; 3] {
    let mut mod_point = mod_point;
    // first case is a standalone triangle
    let mut i12 = create_vertex(mod_point, i1, i2, v1, v2, vertices, dim, axis);
    let mut i13 = create_vertex(mod_point, i1, i3, v1, v3, vertices, dim, axis);
    indices.extend([i1, i12, i13].map(N::from_usize));
    mod_point = mod_point - modulo;
    if v2 > v3 {
      // create lines up to i2
      while mod_point > v2 {
        // next triangles are i13->i12->nexti13 and nexti13->i12->nexti12 so store in necessary order
        indices.extend([i13, i12].map(N::from_usize));
        i13 = create_vertex(mod_point, i1, i3, v1, v3, vertices, dim, axis);
        indices.extend([i13, i13, i12].map(N::from_usize));
        i12 = create_vertex(mod_point, i1, i2, v1, v2, vertices, dim, axis);
        indices.push(N::from_usize(i12));
        // increment
        mod_point = mod_point - modulo;
      }
      // add v2 triangle if necessary
      indices.extend([i13, i12, i2].map(N::from_usize));
      // return the remaining triangle
        [i13, i2, i3].map(N::from_usize)
    } else {
      // create lines up to i2
      while mod_point > v3 {
        // next triangles are i13->i12->nexti13 and nexti13->i12->nexti12 so store in necessary order
        indices.extend([i13, i12].map(N::from_usize));
        i13 = create_vertex(mod_point, i1, i3, v1, v3, vertices, dim, axis);
        indices.extend([i13, i13, i12].map(N::from_usize));
        i12 = create_vertex(mod_point, i1, i2, v1, v2, vertices, dim, axis);
        indices.push(N::from_usize(i12));
        // increment
        mod_point = mod_point - modulo;
      }
      // add v3 triangle if necessary
      indices.extend([i13, i12, i3].map(N::from_usize));
      // return the remaining triangle
      [i3, i12, i2].map(N::from_usize)
    }
  }

/// Returns x modulo n (supports negative numbers)
fn mod2<T: Float>(x: T, n: T) -> T {
    ((x % n) + n) % n
}

/// Flattens a 2D or 3D array whether its a flat point ([x, y, z]) or object ({ x, y, z })
pub fn flatten<T: Float, N: Index>(data: &[Vec<Vec<T>>]) -> (Vec<T>, Vec<N>, usize) {
    let mut vertices = Vec::new();
    let mut hole_indices = Vec::new();
    let mut hole_index = N::zero();
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
            hole_index = hole_index + N::from_usize(data[i - 1].len());
            hole_indices.push(hole_index);
        }
    }

    (vertices, hole_indices, dim)
}

/// Convert a structure into a 2d vector array
pub fn convert_2d<T: Float, P: Point2D<T>>(data: &[Vec<P>]) -> Vec<Vec<Vec<T>>> {
    data.iter().map(|line| line.iter().map(|point| {
        Vec::from([point.x(), point.y()])
    }).collect()).collect()
}

/// Convert a structure into a 3d vector array
pub fn convert_3d<T: Float, P: Point3D<T>>(data: &[Vec<P>]) -> Vec<Vec<Vec<T>>> {
    data.iter().map(|line| line.iter().map(|point| {
        Vec::from([point.x(), point.y(), point.z()])
    }).collect()).collect()
}

/// Returns a percentage difference between the polygon area and its triangulation area;
/// used to verify correctness of triangulation
pub fn deviation<T: Float, N: Index>(
    data: &[T],
    hole_indices: &[N],
    triangles: &[N],
    dim: usize
) -> T {
    let has_holes = !hole_indices.is_empty();
    let outer_len = if has_holes {
        hole_indices[0].into_usize() * dim
     } else { data.len() };
    let mut polygon_area = T::abs(signed_area(data, 0, outer_len, dim));
  
    if has_holes {
    for i in 0..hole_indices.len() {
        let start = hole_indices[i].into_usize() * dim;
        let end = if i < hole_indices.len() - 1 {
            hole_indices[i + 1].into_usize() * dim
        } else { data.len() };
        polygon_area = polygon_area - T::abs(signed_area(data, start, end, dim));
      }
    }
  
    let mut triangles_area = T::zero();
    let mut i = 0;
    while i < triangles.len() {
        let a = triangles[i].into_usize() * dim;
        let b = triangles[i + 1].into_usize() * dim;
        let c = triangles[i + 2].into_usize() * dim;
        triangles_area = triangles_area + T::abs(
            (data[a] - data[c]) * (data[b + 1] - data[a + 1]) -
            (data[a] - data[b]) * (data[c + 1] - data[a + 1]),
        );
        i += 3;
    }
  
    let zero = T::zero();
    if polygon_area == zero && triangles_area == zero {
        zero
    } else { T::abs((triangles_area - polygon_area) / polygon_area) }
}