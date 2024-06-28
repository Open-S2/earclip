struct Node<T: Float> {
    /// vertex index in coordinates array
    i: u32,
    /// vertex coordinate x
    x: T,
    /// vertex coordinate y
    y: T,
    /// z-order curve value
    z: i32,
    /// previous vertex nodes in a polygon ring
    prev: RefCell<Node>,
    /// next vertex nodes in a polygon ring
    next: RefCell<Node>,
    /// previous nodes in z-order
    prev_z: RefCell<Node>,
    /// next nodes in z-order
    next_z: RefCell<Node>,
    /// indicates whether this is a steiner point
    steiner: bool,
}
