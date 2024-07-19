use earclip::earclip;

#[test]
fn empty() {
    let polygon: Vec<Vec<Vec<f64>>> = vec![];
    let (vertices, indices) = earclip::<f64, usize>(&polygon, None, None);
    assert_eq!(vertices, vec![]);
    assert_eq!(indices, vec![]);
}

#[test]
fn simple() {
    let polygon = vec![vec![vec![0.0, 0.0, 0.0], vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0]]];
    let (vertices, indices) = earclip::<f64, usize>(&polygon, None, None);
    assert_eq!(vertices, vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
    assert_eq!(indices, vec![1, 2, 0]);
}

#[test]
fn flat_points() {
    let geometry = vec![
        vec![
            vec![3506.0, -2048.0],
            vec![7464.0, 402.0],
            vec![-2048.0, 2685.0],
            vec![-2048.0, -2048.0],
            vec![3506.0, -2048.0]
        ],
        vec![
            vec![-2048.0, -37.0],
            vec![1235.0, 747.0],
            vec![338.0, -1464.0],
            vec![-116.0, -1188.0],
            vec![-2048.0, -381.0],
            vec![-2048.0, -37.0]
        ],
        vec![
            vec![-1491.0, -1981.0],
            vec![-1300.0, -1800.0],
            vec![-1155.0, -1981.0],
            vec![-1491.0, -1981.0]
        ],
    ];
    let (vertices, indices) = earclip::<f64, usize>(&geometry, None, None);
    assert_eq!(vertices, vec![
        3506.0, -2048.0, 7464.0, 402.0, -2048.0, 2685.0, -2048.0, -2048.0, 3506.0, -2048.0, -2048.0,
        -37.0, 1235.0, 747.0, 338.0, -1464.0, -116.0, -1188.0, -2048.0, -381.0, -2048.0, -37.0,
        -1491.0, -1981.0, -1300.0, -1800.0, -1155.0, -1981.0, -1491.0, -1981.0,
    ]);
    assert_eq!(indices, vec![
        3, 11, 12, 13, 11, 3, 2, 5, 6, 7, 8, 9, 9, 3, 12, 13, 3, 0, 1, 2, 6, 7, 9, 12, 12, 13, 0, 0,
        1, 6, 7, 12, 0, 0, 6, 7,
    ]);
}
