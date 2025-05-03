use criterion::{criterion_group, criterion_main, Criterion};
use earclip::{earcut, flatten_float};
use std::fs;

fn load_fixture(name: &str) -> (Vec<f64>, Vec<usize>, usize) {
    // load JSON
    type Coords = Vec<Vec<Vec<f64>>>;
    let s = fs::read_to_string("./tests/fixtures/".to_string() + name + ".json").unwrap();
    let expected = serde_json::from_str::<Coords>(&s).unwrap();

    // prepare input
    let (vertices, hole_indices, dim) = flatten_float(&expected); // dim => dimensions

    (vertices, hole_indices, dim)
}

fn bench(c: &mut Criterion) {
    c.bench_function("bad-hole", |b| {
        let (vertices, hole_indices, dim) = load_fixture("bad-hole");
        b.iter(|| {
            earcut(&vertices, &hole_indices, dim);
        })
    });

    c.bench_function("building", |b| {
        let (vertices, hole_indices, dim) = load_fixture("building");
        b.iter(|| {
            earcut(&vertices, &hole_indices, dim);
        })
    });

    c.bench_function("degenerate", |b| {
        let (vertices, hole_indices, dim) = load_fixture("degenerate");
        b.iter(|| {
            earcut(&vertices, &hole_indices, dim);
        })
    });

    c.bench_function("dude", |b| {
        let (vertices, hole_indices, dim) = load_fixture("dude");
        b.iter(|| {
            earcut(&vertices, &hole_indices, dim);
        })
    });

    c.bench_function("empty-square", |b| {
        let (vertices, hole_indices, dim) = load_fixture("empty-square");
        b.iter(|| {
            earcut(&vertices, &hole_indices, dim);
        })
    });

    c.bench_function("water", |b| {
        let (vertices, hole_indices, dim) = load_fixture("water");
        b.iter(|| {
            earcut(&vertices, &hole_indices, dim);
        })
    });

    c.bench_function("water2", |b| {
        let (vertices, hole_indices, dim) = load_fixture("water2");
        b.iter(|| {
            earcut(&vertices, &hole_indices, dim);
        })
    });

    c.bench_function("water3", |b| {
        let (vertices, hole_indices, dim) = load_fixture("water3");
        b.iter(|| {
            earcut(&vertices, &hole_indices, dim);
        })
    });

    c.bench_function("water3b", |b| {
        let (vertices, hole_indices, dim) = load_fixture("water3b");
        b.iter(|| {
            earcut(&vertices, &hole_indices, dim);
        })
    });

    c.bench_function("water4", |b| {
        let (vertices, hole_indices, dim) = load_fixture("water4");
        b.iter(|| {
            earcut(&vertices, &hole_indices, dim);
        })
    });

    c.bench_function("water-huge2", |b| {
        let (vertices, hole_indices, dim) = load_fixture("water-huge2");
        b.iter(|| {
            earcut(&vertices, &hole_indices, dim);
        })
    });

    c.bench_function("rain", |b| {
        let (vertices, hole_indices, dim) = load_fixture("rain");
        b.iter(|| {
            earcut(&vertices, &hole_indices, dim);
        })
    });

    c.bench_function("hilbert", |b| {
        let (vertices, hole_indices, dim) = load_fixture("hilbert");
        b.iter(|| {
            earcut(&vertices, &hole_indices, dim);
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
