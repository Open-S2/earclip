use earclip::{deviation, earcut, flatten_float};
use std::fs;

#[test]
fn indices_2d() {
    let verts: Vec<f64> = Vec::from([10.0, 0.0, 0.0, 50.0, 60.0, 60.0, 70.0, 10.0]);
    let indices: Vec<usize> = earcut(&verts, &[], 2);
    assert_eq!(indices, vec![1, 0, 3, 3, 2, 1]);
}

#[test]
fn indices_3d() {
    let verts: Vec<f64> =
        Vec::from([10.0, 0.0, 0.0, 0.0, 50.0, 0.0, 60.0, 60.0, 0.0, 70.0, 10.0, 0.0]);
    let indices: Vec<usize> = earcut(&verts, &[], 3);
    assert_eq!(indices, vec![1, 0, 3, 3, 2, 1]);
}

#[test]
fn empty() {
    let verts: Vec<f64> = Vec::new();
    let indices: Vec<usize> = earcut(&verts, &[], 2);
    let expected: Vec<usize> = Vec::new();
    assert_eq!(indices, expected);
}

#[test]
fn infinite_loop() {
    let verts: Vec<f64> = Vec::from([
        1.0, 2.0, 2.0, 2.0, 1.0, 2.0, 1.0, 1.0, 1.0, 2.0, 4.0, 1.0, 5.0, 1.0, 3.0, 2.0, 4.0, 2.0,
        4.0, 1.0,
    ]);
    let indices: Vec<usize> = earcut(&verts, &[5], 2);
    assert_eq!(indices, vec![8, 5, 6]);
}

fn test_fixture(name: &str, num_triangles: usize, expected_deviation: f64) {
    // load JSON
    type Coords = Vec<Vec<Vec<f64>>>;
    let s = fs::read_to_string("./tests/fixtures/".to_string() + name + ".json").unwrap();
    let expected = serde_json::from_str::<Coords>(&s).unwrap();

    // prepare input
    let (vertices, hole_indices, dim) = flatten_float(&expected); // dim => dimensions

    let indices: Vec<usize> = earcut(&vertices, &hole_indices, dim);

    // check
    assert!(indices.len() == num_triangles * 3);
    if !indices.is_empty() {
        assert!(deviation(&vertices, &hole_indices, &indices, dim) <= expected_deviation);
    }
}

#[test]
fn fixture_building() {
    test_fixture("building", 13, 0.0);
}

#[test]
fn fixture_dude() {
    test_fixture("dude", 106, 2e-15);
}

#[test]
fn fixture_water() {
    test_fixture("water", 2482, 0.0008);
}

#[test]
fn fixture_water2() {
    test_fixture("water2", 1212, 0.0);
}

#[test]
fn fixture_water3() {
    test_fixture("water3", 197, 0.0);
}

#[test]
fn fixture_water3b() {
    test_fixture("water3b", 25, 0.0);
}

#[test]
fn fixture_water4() {
    test_fixture("water4", 705, 0.0);
}

#[test]
fn fixture_water_huge() {
    test_fixture("water-huge", 5179, 0.0011);
}

#[test]
fn fixture_water_huge2() {
    test_fixture("water-huge2", 4462, 0.0028);
}

#[test]
fn fixture_degenerate() {
    test_fixture("degenerate", 0, 0.0);
}

#[test]
fn fixture_bad_hole() {
    test_fixture("bad-hole", 42, 0.019);
}

#[test]
fn fixture_empty_square() {
    test_fixture("empty-square", 0, 0.0);
}

#[test]
fn fixture_issue16() {
    test_fixture("issue16", 12, 4e-16);
}

#[test]
fn fixture_issue17() {
    test_fixture("issue17", 11, 2e-16);
}

#[test]
fn fixture_steiner() {
    test_fixture("steiner", 9, 0.0);
}

#[test]
fn fixture_issue29() {
    test_fixture("issue29", 40, 2e-15);
}

#[test]
fn fixture_issue34() {
    test_fixture("issue34", 139, 0.0);
}

#[test]
fn fixture_issue35() {
    test_fixture("issue35", 844, 0.0);
}

#[test]
fn fixture_self_touching() {
    test_fixture("self-touching", 124, 2e-13);
}

#[test]
fn fixture_outside_ring() {
    test_fixture("outside-ring", 64, 0.0);
}

#[test]
fn fixture_simplified_us_border() {
    test_fixture("simplified-us-border", 120, 0.0);
}

#[test]
fn fixture_touching_holes() {
    test_fixture("touching-holes", 57, 0.0);
}

#[test]
fn fixture_hole_touching_outer() {
    test_fixture("hole-touching-outer", 77, 0.0);
}

#[test]
fn fixture_hilbert() {
    test_fixture("hilbert", 1024, 0.0);
}

#[test]
fn fixture_issue45() {
    test_fixture("issue45", 10, 0.0);
}

#[test]
fn fixture_eberly_3() {
    test_fixture("eberly-3", 73, 0.0);
}

#[test]
fn fixture_eberly_6() {
    test_fixture("eberly-6", 1429, 2e-14);
}

#[test]
fn fixture_issue52() {
    test_fixture("issue52", 109, 0.0);
}

#[test]
fn fixture_shared_points() {
    test_fixture("shared-points", 4, 0.0);
}

#[test]
fn fixture_bad_diagonals() {
    test_fixture("bad-diagonals", 7, 0.0);
}

#[test]
fn fixture_issue83() {
    test_fixture("issue83", 0, 0.0);
}

#[test]
fn fixture_issue107() {
    test_fixture("issue107", 0, 0.0);
}

#[test]
fn fixture_issue111() {
    test_fixture("issue111", 19, 0.0);
}

#[test]
fn fixture_collinear_boxy() {
    test_fixture("boxy", 57, 0.0);
}

#[test]
fn fixture_collinear_diagonal() {
    test_fixture("collinear-diagonal", 14, 0.0);
}

#[test]
fn fixture_issue119() {
    test_fixture("issue119", 18, 0.0);
}

#[test]
fn fixture_hourglass() {
    test_fixture("hourglass", 2, 0.0);
}

#[test]
fn fixture_touching2() {
    test_fixture("touching2", 8, 0.0);
}

#[test]
fn fixture_touching3() {
    test_fixture("touching3", 15, 0.0);
}

#[test]
fn fixture_touching4() {
    test_fixture("touching4", 20, 0.0);
}

#[test]
fn fixture_rain() {
    test_fixture("rain", 2681, 0.0);
}

#[test]
fn fixture_issue131() {
    test_fixture("issue131", 12, 0.0);
}

#[test]
fn fixture_infinite_loop_jhl() {
    test_fixture("infinite-loop-jhl", 0, 0.0);
}

#[test]
fn fixture_filtered_bridge_jhl() {
    test_fixture("filtered-bridge-jhl", 25, 0.0);
}

#[test]
fn fixture_issue149() {
    test_fixture("issue149", 2, 0.0);
}

#[test]
fn fixture_issue142() {
    test_fixture("issue142", 4, 0.13);
}
