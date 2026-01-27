use criterion::{Criterion, black_box, criterion_group, criterion_main};
use t::interpreter::Transform;
use t::operators::Columnate;
use t::value::{Array, Level, Value};

fn make_test_data(rows: usize, cols: usize) -> Value {
    let elements: Vec<Value> = (0..rows)
        .map(|r| {
            let cells: Vec<Value> = (0..cols)
                .map(|c| {
                    // Vary the string lengths to simulate real data
                    let len = ((r * 7 + c * 13) % 20) + 1;
                    Value::Text("x".repeat(len))
                })
                .collect();
            Value::Array(Array::from((cells, Level::Word)))
        })
        .collect();
    Value::Array(Array::from((elements, Level::Line)))
}

fn bench_columnate(c: &mut Criterion) {
    let small = make_test_data(100, 10);
    let medium = make_test_data(10_000, 10);
    let large = make_test_data(100_000, 10);

    c.bench_function("columnate_100x10", |b| {
        b.iter(|| {
            let input = small.deep_copy();
            black_box(Columnate.apply(input).unwrap())
        })
    });

    c.bench_function("columnate_10kx10", |b| {
        b.iter(|| {
            let input = medium.deep_copy();
            black_box(Columnate.apply(input).unwrap())
        })
    });

    c.bench_function("columnate_100kx10", |b| {
        b.iter(|| {
            let input = large.deep_copy();
            black_box(Columnate.apply(input).unwrap())
        })
    });
}

criterion_group!(benches, bench_columnate);
criterion_main!(benches);
