use alloc::vec::Vec;
use core::{borrow::BorrowMut, cmp::Ordering, f64, ptr};
use libm::fabs;

macro_rules! node {
    ($self:ident.$nodes:ident, $index:expr) => {{
        assert!($index < $self.$nodes.len(), "Index out of bounds");
        &$self.$nodes[$index]
    }};
    ($nodes:ident, $index:expr) => {{
        assert!($index < $nodes.len(), "Index out of bounds");
        &$nodes[$index]
    }};
}

macro_rules! node_mut {
    ($self:ident.$nodes:ident, $index:expr) => {{
        assert!($index < $self.$nodes.len(), "Index out of bounds");
        &mut $self.$nodes[$index]
    }};
    ($nodes:ident, $index:expr) => {{
        assert!($index < $nodes.len(), "Index out of bounds");
        &mut $nodes[$index]
    }};
}

/// Nodes help form a LinkedList and track information about the point itself and its neighbours
pub struct Node {
    /// vertex index in coordinates array
    i: usize,
    /// z-order curve value
    z: i32,
    /// vertex coordinates x
    xy: [f64; 2],
    /// previous vertex nodes in a polygon ring
    prev_i: usize,
    /// next vertex nodes in a polygon ring
    next_i: usize,
    /// previous nodes in z-order
    prev_z_i: Option<usize>,
    /// next nodes in z-order
    next_z_i: Option<usize>,
    /// indicates whether this is a steiner point
    steiner: bool,
}

struct LinkInfo {
    prev_i: usize,
    next_i: usize,
    prev_z_i: Option<usize>,
    next_z_i: Option<usize>,
}

impl Node {
    fn new(i: usize, xy: [f64; 2]) -> Self {
        Self { i, xy, prev_i: 1, next_i: 1, z: 0, prev_z_i: None, next_z_i: None, steiner: false }
    }

    fn link_info(&self) -> LinkInfo {
        LinkInfo {
            prev_i: self.prev_i,
            next_i: self.next_i,
            prev_z_i: self.prev_z_i,
            next_z_i: self.next_z_i,
        }
    }
}

/// Instance of the earcut algorithm.
pub struct Store<'a> {
    data: &'a [f64],
    nodes: &'a mut Vec<Node>,
    queue: &'a mut Vec<(usize, f64)>,
}

impl<'a> Store<'a> {
    /// Creates a new instance of the earcut algorithm.
    /// You can reuse a single instance for multiple triangulations to reduce memory allocations.
    pub fn new(
        data: &'a [f64],
        nodes: &'a mut Vec<Node>,
        queue: &'a mut Vec<(usize, f64)>,
    ) -> Self {
        let store = Self { data, nodes, queue };
        store.nodes.push(Node::new(0, [f64::INFINITY, f64::INFINITY])); // dummy node

        store
    }

    fn reset(&mut self, capacity: usize) {
        self.nodes.clear();
        self.nodes.reserve(capacity);
        self.nodes.push(Node::new(0, [f64::INFINITY, f64::INFINITY])); // dummy node
    }
}

/// Performs the earcut triangulation on a polygon.
/// The API is similar to the original JavaScript implementation, except you can provide a vector for the output indices.
pub fn earcut(data: &[f64], hole_indices: &[usize], dim: usize) -> Vec<usize> {
    let nodes = &mut Vec::new();
    let queue = &mut Vec::new();
    let mut store = Store::new(data, nodes, queue);
    let mut triangles_out: Vec<usize> = Vec::new();
    if store.data.len() < 3 {
        return triangles_out;
    }
    earcut_impl(&mut store, hole_indices, &mut triangles_out, dim);

    triangles_out
}

/// Performs the earcut triangulation on a polygon.
pub fn earcut_impl(
    store: &mut Store,
    hole_indices: &[usize],
    triangles_out: &mut Vec<usize>,
    dim: usize,
) {
    triangles_out.reserve(store.data.len() + 1);
    store.reset(store.data.len() / 2 * 3);

    let has_holes = !hole_indices.is_empty();
    let outer_len: usize = if has_holes { hole_indices[0] * dim } else { store.data.len() };

    // create nodes
    let Some(mut outer_node_i) = linked_list(store, 0, outer_len, dim, true) else {
        return;
    };
    let outer_node = node!(store.nodes, outer_node_i);
    if outer_node.next_i == outer_node.prev_i {
        return;
    }
    if has_holes {
        outer_node_i = eliminate_holes(store, hole_indices, outer_node_i, dim);
    }

    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut inv_size = 0.;

    // if the shape is not too simple, we'll use z-order curve hash later; calculate polygon bbox
    if store.data.len() > 80 * dim {
        // Initialize min_x, min_y, max_x, max_y with the first coordinate
        if !store.data.is_empty() {
            min_x = store.data[0];
            min_y = store.data[1];
            max_x = store.data[0];
            max_y = store.data[1];
        }

        for i in (dim..outer_len).step_by(dim) {
            let x = store.data[i];
            let y = store.data[i + 1];
            if x < min_x {
                min_x = x;
            }
            if y < min_y {
                min_y = y;
            }
            if x > max_x {
                max_x = x;
            }
            if y > max_y {
                max_y = y;
            }
        }

        // Calculate inv_size used for z-order curve hash scaling
        let bbox_width = max_x - min_x;
        let bbox_height = max_y - min_y;
        let largest_dim = f64::max(bbox_width, bbox_height);
        if largest_dim != 0. {
            inv_size = 32767.0 / largest_dim;
        } else {
            inv_size = 0.;
        }
    }

    earcut_linked(
        store.nodes.borrow_mut(),
        outer_node_i,
        triangles_out,
        dim,
        min_x,
        min_y,
        inv_size,
        Pass::P0,
    );
}

/// create a circular doubly linked list from polygon points in the specified winding order
fn linked_list(
    store: &mut Store,
    start: usize,
    end: usize,
    dim: usize,
    clockwise: bool,
) -> Option<usize> {
    let mut last_i: Option<usize> = None;

    if clockwise == (signed_area(store.data.borrow_mut(), start, end, dim) > 0.) {
        let mut i = start;
        while i < end {
            last_i = Some(insert_node(
                store.nodes.borrow_mut(),
                i,
                [store.data[i], store.data[i + 1]],
                last_i,
            ));
            i += dim;
        }
    } else {
        let mut i = end - dim;
        while i >= start {
            last_i = Some(insert_node(
                store.nodes.borrow_mut(),
                i,
                [store.data[i], store.data[i + 1]],
                last_i,
            ));
            if i == 0 {
                break;
            } // Prevent underflow
            i -= dim;
        }
    };

    if let Some(li) = last_i {
        let last = node!(store.nodes, li);
        if equals(last, node!(store.nodes, last.next_i)) {
            let ll = last.link_info();
            let (_, next_i) = remove_node(store.nodes.borrow_mut(), ll);
            last_i = Some(next_i);
        }
    }

    last_i
}

/// link every hole into the outer loop, producing a single-ring polygon without holes
fn eliminate_holes(
    store: &mut Store,
    hole_indices: &[usize],
    mut outer_node_i: usize,
    dim: usize,
) -> usize {
    store.queue.clear();
    for (i, hi) in hole_indices.iter().enumerate() {
        let start = *hi * dim;
        let end =
            if i < hole_indices.len() - 1 { hole_indices[i + 1] * dim } else { store.data.len() };
        if let Some(list_i) = linked_list(store, start, end, dim, false) {
            let list = &mut node_mut!(store.nodes, list_i);
            if list_i == list.next_i {
                list.steiner = true;
            }
            let (leftmost_i, leftmost) = get_leftmost(store.nodes.borrow_mut(), list_i);
            store.queue.push((leftmost_i, leftmost.xy[0]));
        }
    }

    store
        .queue
        .sort_unstable_by(|(_a, ax), (_b, bx)| ax.partial_cmp(bx).unwrap_or(Ordering::Equal));

    // process holes from left to right
    for &(q, _) in store.queue.iter() {
        outer_node_i = eliminate_hole(store.nodes.borrow_mut(), q, outer_node_i);
    }

    outer_node_i
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pass {
    P0 = 0,
    P1 = 1,
    P2 = 2,
}

/// main ear slicing loop which triangulates a polygon (given as a linked list)
#[allow(clippy::too_many_arguments)]
fn earcut_linked(
    nodes: &mut Vec<Node>,
    ear_i: usize,
    triangles: &mut Vec<usize>,
    dim: usize,
    min_x: f64,
    min_y: f64,
    inv_size: f64,
    pass: Pass,
) {
    let mut ear_i = ear_i;

    // interlink polygon nodes in z-order
    if pass == Pass::P0 && inv_size != 0. {
        index_curve(nodes, ear_i, min_x, min_y, inv_size);
    }

    let mut stop_i = ear_i;

    // iterate through ears, slicing them one by one
    loop {
        let ear = node!(nodes, ear_i);
        if ear.prev_i == ear.next_i {
            break;
        }
        let ni = ear.next_i;

        let (is_ear, prev, next) = if inv_size != 0. {
            is_ear_hashed(nodes, ear, min_x, min_y, inv_size)
        } else {
            is_ear(nodes, ear)
        };
        if is_ear {
            let next_i = next.i;
            let next_next_i = next.next_i;

            // cut off the triangle
            triangles.push(prev.i / dim);
            triangles.push(ear.i / dim);
            triangles.push(next_i / dim);

            let ll = ear.link_info();
            remove_node(nodes, ll);

            // skipping the next vertex leads to less sliver triangles
            (ear_i, stop_i) = (next_next_i, next_next_i);

            continue;
        }

        ear_i = ni;

        // if we looped through the whole remaining polygon and can't find any more ears
        if ear_i == stop_i {
            if pass == Pass::P0 {
                // try filtering points and slicing again
                ear_i = filter_points(nodes, ear_i, None);
                earcut_linked(nodes, ear_i, triangles, dim, min_x, min_y, inv_size, Pass::P1);
            } else if pass == Pass::P1 {
                // if this didn't work, try curing all small self-intersections locally
                let filtered = filter_points(nodes, ear_i, None);
                ear_i = cure_local_intersections(nodes, filtered, triangles, dim);
                earcut_linked(nodes, ear_i, triangles, dim, min_x, min_y, inv_size, Pass::P2);
            } else if pass == Pass::P2 {
                // as a last resort, try splitting the remaining polygon into two
                split_earcut(nodes, ear_i, triangles, dim, min_x, min_y, inv_size);
            }
            return;
        }
    }
}

/// check whether a polygon node forms a valid ear with adjacent nodes
fn is_ear<'a>(nodes: &'a [Node], ear: &'a Node) -> (bool, &'a Node, &'a Node) {
    let b = ear;
    let a = node!(nodes, b.prev_i);
    let c = node!(nodes, b.next_i);

    if area(a, b, c) >= 0. {
        // reflex, can't be an ear
        return (false, a, c);
    }

    // now make sure we don't have other points inside the potential ear

    // triangle bbox
    let x0 = a.xy[0].min(b.xy[0].min(c.xy[0]));
    let y0 = a.xy[1].min(b.xy[1].min(c.xy[1]));
    let x1 = a.xy[0].max(b.xy[0].max(c.xy[0]));
    let y1 = a.xy[1].max(b.xy[1].max(c.xy[1]));

    let mut p = node!(nodes, c.next_i);
    let mut p_prev = node!(nodes, p.prev_i);
    while !ptr::eq(p, a) {
        let p_next = node!(nodes, p.next_i);
        if (p.xy[0] >= x0 && p.xy[0] <= x1 && p.xy[1] >= y0 && p.xy[1] <= y1)
            && point_in_triangle(a.xy, b.xy, c.xy, p.xy)
            && area(p_prev, p, p_next) >= 0.
        {
            return (false, a, c);
        }
        (p_prev, p) = (p, p_next);
    }
    (true, a, c)
}

fn is_ear_hashed<'a>(
    nodes: &'a [Node],
    ear: &'a Node,
    min_x: f64,
    min_y: f64,
    inv_size: f64,
) -> (bool, &'a Node, &'a Node) {
    let b = ear;
    let a = node!(nodes, b.prev_i);
    let c = node!(nodes, b.next_i);

    if area(a, b, c) >= 0. {
        // reflex, can't be an ear
        return (false, a, c);
    }

    // triangle bbox
    let xy_min = [a.xy[0].min(b.xy[0].min(c.xy[0])), a.xy[1].min(b.xy[1].min(c.xy[1]))];
    let xy_max = [a.xy[0].max(b.xy[0].max(c.xy[0])), a.xy[1].max(b.xy[1].max(c.xy[1]))];

    // z-order range for the current triangle bbox;
    let min_z = z_order(xy_min, min_x, min_y, inv_size);
    let max_z = z_order(xy_max, min_x, min_y, inv_size);

    let mut o_p = ear.prev_z_i.map(|i| node!(nodes, i));
    let mut o_n = ear.next_z_i.map(|i| node!(nodes, i));

    // look for points inside the triangle in both directions
    loop {
        let Some(p) = o_p else { break };
        if p.z < min_z {
            break;
        };
        let Some(n) = o_n else { break };
        if n.z > max_z {
            break;
        };

        if ((p.xy[0] >= xy_min[0])
            & (p.xy[0] <= xy_max[0])
            & (p.xy[1] >= xy_min[1])
            & (p.xy[1] <= xy_max[1]))
            && (!ptr::eq(p, a) && !ptr::eq(p, c))
            && point_in_triangle(a.xy, b.xy, c.xy, p.xy)
            && area(node!(nodes, p.prev_i), p, node!(nodes, p.next_i)) >= 0.
        {
            return (false, a, c);
        }
        o_p = p.prev_z_i.map(|i| node!(nodes, i));

        if ((n.xy[0] >= xy_min[0])
            & (n.xy[0] <= xy_max[0])
            & (n.xy[1] >= xy_min[1])
            & (n.xy[1] <= xy_max[1]))
            && (!ptr::eq(n, a) && !ptr::eq(n, c))
            && point_in_triangle(a.xy, b.xy, c.xy, n.xy)
            && area(node!(nodes, n.prev_i), n, node!(nodes, n.next_i)) >= 0.
        {
            return (false, a, c);
        }
        o_n = n.next_z_i.map(|i| node!(nodes, i));
    }

    // look for remaining points in decreasing z-order
    while let Some(p) = o_p {
        if p.z < min_z {
            break;
        };
        if ((p.xy[0] >= xy_min[0])
            & (p.xy[0] <= xy_max[0])
            & (p.xy[1] >= xy_min[1])
            & (p.xy[1] <= xy_max[1]))
            && (!ptr::eq(p, a) && !ptr::eq(p, c))
            && point_in_triangle(a.xy, b.xy, c.xy, p.xy)
            && area(node!(nodes, p.prev_i), p, node!(nodes, p.next_i)) >= 0.
        {
            return (false, a, c);
        }
        o_p = p.prev_z_i.map(|i| node!(nodes, i));
    }

    // look for remaining points in increasing z-order
    while let Some(n) = o_n {
        if n.z > max_z {
            break;
        };
        if ((n.xy[0] >= xy_min[0])
            & (n.xy[0] <= xy_max[0])
            & (n.xy[1] >= xy_min[1])
            & (n.xy[1] <= xy_max[1]))
            && (!ptr::eq(n, a) && !ptr::eq(n, c))
            && point_in_triangle(a.xy, b.xy, c.xy, n.xy)
            && area(node!(nodes, n.prev_i), n, node!(nodes, n.next_i)) >= 0.
        {
            return (false, a, c);
        }
        o_n = n.next_z_i.map(|i| node!(nodes, i));
    }

    (true, a, c)
}

/// go through all polygon nodes and cure small local self-intersections
fn cure_local_intersections(
    nodes: &mut [Node],
    mut start_i: usize,
    triangles: &mut Vec<usize>,
    dim: usize,
) -> usize {
    let mut p_i = start_i;
    loop {
        let p = node!(nodes, p_i);
        let p_next_i = p.next_i;
        let p_next = node!(nodes, p_next_i);
        let b_i = p_next.next_i;
        let a = node!(nodes, p.prev_i);
        let b = node!(nodes, b_i);

        if !equals(a, b)
            && intersects(a, p, p_next, b)
            && locally_inside(nodes, a, b)
            && locally_inside(nodes, b, a)
        {
            triangles.extend([a.i / dim, p.i / dim, b.i / dim]);

            let b_next_i = b.next_i;
            remove_node(nodes, p.link_info());
            let pnl = node!(nodes, p_next_i).link_info();
            remove_node(nodes, pnl);

            (p_i, start_i) = (b_next_i, b_i);
        } else {
            p_i = p.next_i;
        }

        if p_i == start_i {
            return filter_points(nodes, p_i, None);
        }
    }
}

/// try splitting polygon into two and triangulate them independently
fn split_earcut(
    nodes: &mut Vec<Node>,
    start_i: usize,
    triangles: &mut Vec<usize>,
    dim: usize,
    min_x: f64,
    min_y: f64,
    inv_size: f64,
) {
    // look for a valid diagonal that divides the polygon into two
    let mut ai = start_i;
    let mut a = node!(nodes, ai);
    loop {
        let a_next = node!(nodes, a.next_i);
        let a_prev = node!(nodes, a.prev_i);
        let mut bi = a_next.next_i;

        while bi != a.prev_i {
            let b = node!(nodes, bi);
            if a.i != b.i && is_valid_diagonal(nodes, a, b, a_next, a_prev) {
                // split the polygon in two by the diagonal
                let mut ci = split_polygon(nodes, ai, bi);

                // filter colinear points around the cuts
                let end_i = Some(node!(nodes, ai).next_i);
                ai = filter_points(nodes, ai, end_i);
                let end_i = Some(node!(nodes, ci).next_i);
                ci = filter_points(nodes, ci, end_i);

                // run earcut on each half
                earcut_linked(nodes, ai, triangles, dim, min_x, min_y, inv_size, Pass::P0);
                earcut_linked(nodes, ci, triangles, dim, min_x, min_y, inv_size, Pass::P0);
                return;
            }
            bi = b.next_i;
        }

        ai = a.next_i;
        if ai == start_i {
            return;
        }
        a = a_next;
    }
}

/// interlink polygon nodes in z-order
fn index_curve(nodes: &mut [Node], start_i: usize, min_x: f64, min_y: f64, inv_size: f64) {
    let mut p_i = start_i;
    let mut p = node_mut!(nodes, p_i);

    loop {
        if p.z == 0 {
            p.z = z_order(p.xy, min_x, min_y, inv_size);
        }
        p.prev_z_i = Some(p.prev_i);
        p.next_z_i = Some(p.next_i);
        p_i = p.next_i;
        p = node_mut!(nodes, p_i);
        if p_i == start_i {
            break;
        }
    }

    let p_prev_z_i = p.prev_z_i.take().unwrap();
    node_mut!(nodes, p_prev_z_i).next_z_i = None;
    sort_linked(nodes, p_i);
}

/// Simon Tatham's linked list merge sort algorithm
/// http://www.chiark.greenend.org.uk/~sgtatham/algorithms/listsort.html
fn sort_linked(nodes: &mut [Node], list_i: usize) {
    let mut in_size: usize = 1;
    let mut list_i = Some(list_i);

    loop {
        let mut p_i = list_i;
        list_i = None;
        let mut tail_i: Option<usize> = None;
        let mut num_merges = 0;

        while let Some(p_i_s) = p_i {
            num_merges += 1;
            let mut q_i = node!(nodes, p_i_s).next_z_i;
            let mut p_size: usize = 1;
            for _ in 1..in_size {
                if let Some(i) = q_i {
                    p_size += 1;
                    q_i = node!(nodes, i).next_z_i;
                } else {
                    break;
                }
            }
            let mut q_size = in_size;

            loop {
                let e_i = if p_size > 0 {
                    let Some(p_i_s) = p_i else { break };
                    if q_size > 0 {
                        if let Some(q_i_s) = q_i {
                            if node!(nodes, p_i_s).z <= node!(nodes, q_i_s).z {
                                p_size -= 1;
                                let e = node_mut!(nodes, p_i_s);
                                e.prev_z_i = tail_i;
                                p_i = e.next_z_i;
                                p_i_s
                            } else {
                                q_size -= 1;
                                let e = node_mut!(nodes, q_i_s);
                                e.prev_z_i = tail_i;
                                q_i = e.next_z_i;
                                q_i_s
                            }
                        } else {
                            p_size -= 1;
                            let e = node_mut!(nodes, p_i_s);
                            e.prev_z_i = tail_i;
                            p_i = e.next_z_i;
                            p_i_s
                        }
                    } else {
                        p_size -= 1;
                        let e = node_mut!(nodes, p_i_s);
                        e.prev_z_i = tail_i;
                        p_i = e.next_z_i;
                        p_i_s
                    }
                } else if q_size > 0 {
                    if let Some(q_i_s) = q_i {
                        q_size -= 1;
                        let e = node_mut!(nodes, q_i_s);
                        e.prev_z_i = tail_i;
                        q_i = e.next_z_i;
                        q_i_s
                    } else {
                        break;
                    }
                } else {
                    break;
                };

                if let Some(tail_i) = tail_i {
                    node_mut!(nodes, tail_i).next_z_i = Some(e_i);
                } else {
                    list_i = Some(e_i);
                }
                tail_i = Some(e_i);
            }

            p_i = q_i;
        }

        node_mut!(nodes, tail_i.unwrap()).next_z_i = None;
        if num_merges <= 1 {
            break;
        }
        in_size *= 2;
    }
}

/// find the leftmost node of a polygon ring
fn get_leftmost(nodes: &[Node], start_i: usize) -> (usize, &Node) {
    let mut p_i = start_i;
    let mut p = node!(nodes, p_i);
    let mut leftmost_i = start_i;
    let mut leftmost = p;

    loop {
        if p.xy[0] < leftmost.xy[0] || (p.xy[0] == leftmost.xy[0] && p.xy[1] < leftmost.xy[1]) {
            (leftmost_i, leftmost) = (p_i, p);
        }
        p_i = p.next_i;
        if p_i == start_i {
            return (leftmost_i, leftmost);
        }
        p = node!(nodes, p_i);
    }
}

/// check if a diagonal between two polygon nodes is valid (lies in polygon interior)
fn is_valid_diagonal(nodes: &[Node], a: &Node, b: &Node, a_next: &Node, a_prev: &Node) -> bool {
    let b_next = node!(nodes, b.next_i);
    let b_prev = node!(nodes, b.prev_i);
    // dones't intersect other edges
    (((a_next.i != b.i) && (a_prev.i != b.i)) && !intersects_polygon(nodes, a, b))
        // locally visible
        && ((locally_inside(nodes, a, b) && locally_inside(nodes, b, a) && middle_inside(nodes, a, b))
            // does not create opposite-facing sectors
            && (area(a_prev, a, b_prev) != 0. || area(a, b_prev, b) != 0.)
            // special zero-length case
            || equals(a, b)
                && area(a_prev, a, a_next) > 0.
                && area(b_prev, b, b_next) > 0.)
}

/// check if two segments intersect
fn intersects(p1: &Node, q1: &Node, p2: &Node, q2: &Node) -> bool {
    let o1 = sign(area(p1, q1, p2));
    let o2 = sign(area(p1, q1, q2));
    let o3 = sign(area(p2, q2, p1));
    let o4 = sign(area(p2, q2, q1));
    ((o1 != o2) & (o3 != o4)) // general case
        || (o3 == 0 && on_segment(p2, p1, q2)) // p2, q2 and p1 are collinear and p1 lies on p2q2
        || (o4 == 0 && on_segment(p2, q1, q2)) // p2, q2 and q1 are collinear and q1 lies on p2q2
        || (o2 == 0 && on_segment(p1, q2, q1)) // p1, q1 and q2 are collinear and q2 lies on p1q1
        || (o1 == 0 && on_segment(p1, p2, q1)) // p1, q1 and p2 are collinear and p2 lies on p1q1
}

/// check if a polygon diagonal intersects any polygon segments
fn intersects_polygon(nodes: &[Node], a: &Node, b: &Node) -> bool {
    let mut p = a;
    loop {
        let p_next = node!(nodes, p.next_i);
        if (((p.i != a.i) && (p.i != b.i)) && ((p_next.i != a.i) && (p_next.i != b.i)))
            && intersects(p, p_next, a, b)
        {
            return true;
        }
        p = p_next;
        if ptr::eq(p, a) {
            return false;
        }
    }
}

/// check if the middle point of a polygon diagonal is inside the polygon
fn middle_inside(nodes: &[Node], a: &Node, b: &Node) -> bool {
    let mut p = a;
    let mut inside = false;
    let two = 2.;
    let (px, py) = ((a.xy[0] + b.xy[0]) / two, (a.xy[1] + b.xy[1]) / two);
    loop {
        let p_next = node!(nodes, p.next_i);
        inside ^= (p.xy[1] > py) != (p_next.xy[1] > py)
            && p_next.xy[1] != p.xy[1]
            && (px
                < (p_next.xy[0] - p.xy[0]) * (py - p.xy[1]) / (p_next.xy[1] - p.xy[1]) + p.xy[0]);
        p = p_next;
        if ptr::eq(p, a) {
            return inside;
        }
    }
}

/// find a bridge between vertices that connects hole with an outer ring and and link it
fn eliminate_hole(nodes: &mut Vec<Node>, hole_i: usize, outer_node_i: usize) -> usize {
    let Some(bridge_i) = find_hole_bridge(nodes, node!(nodes, hole_i), outer_node_i) else {
        return outer_node_i;
    };
    let bridge_reverse_i = split_polygon(nodes, bridge_i, hole_i);

    // filter collinear points around the cuts
    let end_i = Some(node!(nodes, bridge_reverse_i).next_i);
    filter_points(nodes, bridge_reverse_i, end_i);
    let end_i = Some(node!(nodes, bridge_i).next_i);
    filter_points(nodes, bridge_i, end_i)
}

/// check if a polygon diagonal is locally inside the polygon
fn locally_inside(nodes: &[Node], a: &Node, b: &Node) -> bool {
    let a_prev = node!(nodes, a.prev_i);
    let a_next = node!(nodes, a.next_i);
    if area(a_prev, a, a_next) < 0. {
        area(a, b, a_next) >= 0. && area(a, a_prev, b) >= 0.
    } else {
        area(a, b, a_prev) < 0. || area(a, a_next, b) < 0.
    }
}

/// David Eberly's algorithm for finding a bridge between hole and outer polygon
fn find_hole_bridge(nodes: &[Node], hole: &Node, outer_node_i: usize) -> Option<usize> {
    let mut p_i = outer_node_i;
    let mut qx = f64::NEG_INFINITY;
    let mut m_i: Option<usize> = None;

    // find a segment intersected by a ray from the hole's leftmost point to the left;
    // segment's endpoint with lesser x will be potential connection point
    let mut p = node!(nodes, p_i);
    loop {
        let p_next = node!(nodes, p.next_i);
        if hole.xy[1] <= p.xy[1] && hole.xy[1] >= p_next.xy[1] && p_next.xy[1] != p.xy[1] {
            let x = p.xy[0]
                + (hole.xy[1] - p.xy[1]) * (p_next.xy[0] - p.xy[0]) / (p_next.xy[1] - p.xy[1]);
            if x <= hole.xy[0] && x > qx {
                qx = x;
                m_i = Some(if p.xy[0] < p_next.xy[0] { p_i } else { p.next_i });
                if x == hole.xy[0] {
                    // hole touches outer segment; pick leftmost endpoint
                    return m_i;
                }
            }
        }
        p_i = p.next_i;
        if p_i == outer_node_i {
            break;
        }
        p = p_next;
    }

    let mut m_i = m_i?;

    // look for points inside the triangle of hole point, segment intersection and endpoint;
    // if there are no points found, we have a valid connection;
    // otherwise choose the point of the minimum angle with the ray as connection point

    let stop_i = m_i;
    let mut m = node!(nodes, m_i);
    let mxmy = m.xy;
    let mut tan_min = f64::INFINITY;

    p_i = m_i;
    let mut p = m;

    loop {
        if (((hole.xy[0] >= p.xy[0]) & (p.xy[0] >= mxmy[0])) && hole.xy[0] != p.xy[0])
            && point_in_triangle(
                [if hole.xy[1] < mxmy[1] { hole.xy[0] } else { qx }, hole.xy[1]],
                mxmy,
                [if hole.xy[1] < mxmy[1] { qx } else { hole.xy[0] }, hole.xy[1]],
                p.xy,
            )
        {
            let tan = fabs(hole.xy[1] - p.xy[1]) / (hole.xy[0] - p.xy[0]);
            if locally_inside(nodes, p, hole)
                && (tan < tan_min
                    || (tan == tan_min
                        && (p.xy[0] > m.xy[0]
                            || (p.xy[0] == m.xy[0] && sector_contains_sector(nodes, m, p)))))
            {
                (m_i, m) = (p_i, p);
                tan_min = tan;
            }
        }

        p_i = p.next_i;
        if p_i == stop_i {
            return Some(m_i);
        }
        p = node!(nodes, p_i);
    }
}

/// whether sector in vertex m contains sector in vertex p in the same coordinates
fn sector_contains_sector(nodes: &[Node], m: &Node, p: &Node) -> bool {
    area(node!(nodes, m.prev_i), m, node!(nodes, p.prev_i)) < 0.
        && area(node!(nodes, p.next_i), m, node!(nodes, m.next_i)) < 0.
}

/// eliminate colinear or duplicate points
fn filter_points(nodes: &mut [Node], start_i: usize, end_i: Option<usize>) -> usize {
    let mut end_i = end_i.unwrap_or(start_i);

    let mut p_i = start_i;
    let mut p = node!(nodes, p_i);
    loop {
        let p_next = node!(nodes, p.next_i);
        if !p.steiner && (equals(p, p_next) || area(node!(nodes, p.prev_i), p, p_next) == 0.) {
            let (prev_i, next_i) = remove_node(nodes, p.link_info());
            (p_i, end_i) = (prev_i, prev_i);
            if p_i == next_i {
                return end_i;
            }
            p = node!(nodes, p_i);
        } else {
            p_i = p.next_i;
            if p_i == end_i {
                return end_i;
            }
            p = p_next;
        };
    }
}

/// link two polygon vertices with a bridge; if the vertices belong to the same ring, it splits polygon into two;
/// if one belongs to the outer ring and another to a hole, it merges it into a single ring
fn split_polygon(nodes: &mut Vec<Node>, a_i: usize, b_i: usize) -> usize {
    debug_assert!(!nodes.is_empty());
    let a2_i = nodes.len();
    let b2_i = nodes.len() + 1;

    let a = node_mut!(nodes, a_i);
    let mut a2 = Node::new(a.i, a.xy);
    let an_i = a.next_i;
    a.next_i = b_i;
    a2.prev_i = b2_i;
    a2.next_i = an_i;

    let b = node_mut!(nodes, b_i);
    let mut b2 = Node::new(b.i, b.xy);
    let bp_i = b.prev_i;
    b.prev_i = a_i;
    b2.next_i = a2_i;
    b2.prev_i = bp_i;

    node_mut!(nodes, an_i).prev_i = a2_i;
    node_mut!(nodes, bp_i).next_i = b2_i;

    nodes.extend([a2, b2]);

    b2_i
}

/// create a node and optionally link it with previous one (in a circular doubly linked list)
fn insert_node(nodes: &mut Vec<Node>, i: usize, xy: [f64; 2], last: Option<usize>) -> usize {
    let mut p = Node::new(i, xy);
    let p_i = nodes.len();
    match last {
        Some(last_i) => {
            let last = node_mut!(nodes, last_i);
            let last_next_i = last.next_i;
            (p.next_i, last.next_i) = (last_next_i, p_i);
            p.prev_i = last_i;
            node_mut!(nodes, last_next_i).prev_i = p_i;
        }
        None => {
            (p.prev_i, p.next_i) = (p_i, p_i);
        }
    }
    nodes.push(p);
    p_i
}

fn remove_node(nodes: &mut [Node], pl: LinkInfo) -> (usize, usize) {
    let prev = node_mut!(nodes, pl.prev_i);
    prev.next_i = pl.next_i;
    if let Some(prev_z_i) = pl.prev_z_i {
        if prev_z_i == pl.prev_i {
            prev.next_z_i = pl.next_z_i;
        } else {
            node_mut!(nodes, prev_z_i).next_z_i = pl.next_z_i;
        }
    }

    let next = node_mut!(nodes, pl.next_i);
    next.prev_i = pl.prev_i;
    if let Some(next_z_i) = pl.next_z_i {
        if next_z_i == pl.next_i {
            next.prev_z_i = pl.prev_z_i;
        } else {
            node_mut!(nodes, next_z_i).prev_z_i = pl.prev_z_i;
        }
    }

    (pl.prev_i, pl.next_i)
}

/// check if a point lies within a convex triangle
pub fn signed_area(data: &[f64], start: usize, end: usize, dim: usize) -> f64 {
    let mut sum = 0.;
    let mut i = start;
    let mut j = end - dim;
    while i < end {
        sum += (data[j] - data[i]) * (data[i + 1] + data[j + 1]);
        j = i;
        i += dim;
    }

    sum
}

/// z-order of a point given coords and inverse of the longer side of data bbox
fn z_order(xy: [f64; 2], min_x: f64, min_y: f64, inv_size: f64) -> i32 {
    // coords are transformed into non-negative 15-bit integer range
    let x = (xy[0] - min_x) * inv_size;
    let y = (xy[1] - min_y) * inv_size;
    let mut xy = (x as i64) << 32 | y as i64;
    xy = (xy | (xy << 8)) & 0x00FF00FF00FF00FF;
    xy = (xy | (xy << 4)) & 0x0F0F0F0F0F0F0F0F;
    xy = (xy | (xy << 2)) & 0x3333333333333333;
    xy = (xy | (xy << 1)) & 0x5555555555555555;
    (xy >> 32 | xy << 1) as i32
}

#[allow(clippy::too_many_arguments)]
fn point_in_triangle(a: [f64; 2], b: [f64; 2], c: [f64; 2], p: [f64; 2]) -> bool {
    ((c[0] - p[0]) * (a[1] - p[1]) >= (a[0] - p[0]) * (c[1] - p[1]))
        && ((a[0] - p[0]) * (b[1] - p[1]) >= (b[0] - p[0]) * (a[1] - p[1]))
        && ((b[0] - p[0]) * (c[1] - p[1]) >= (c[0] - p[0]) * (b[1] - p[1]))
}

/// signed area of a triangle
fn area(p: &Node, q: &Node, r: &Node) -> f64 {
    (q.xy[1] - p.xy[1]) * (r.xy[0] - q.xy[0]) - (q.xy[0] - p.xy[0]) * (r.xy[1] - q.xy[1])
}

/// check if two points are equal
fn equals(p1: &Node, p2: &Node) -> bool {
    p1.xy == p2.xy
}

/// for collinear points p, q, r, check if point q lies on segment pr
fn on_segment(p: &Node, q: &Node, r: &Node) -> bool {
    ((q.xy[0] <= p.xy[0].max(r.xy[0])) & (q.xy[1] <= p.xy[1].max(r.xy[1])))
        && ((q.xy[0] >= p.xy[0].min(r.xy[0])) & (q.xy[1] >= p.xy[1].min(r.xy[1])))
}

fn sign(v: f64) -> i32 {
    (v > 0.) as i32 - (v < 0.) as i32
}
