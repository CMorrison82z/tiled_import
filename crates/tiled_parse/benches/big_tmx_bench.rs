use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tiled_parse::parse::parse_embedded;

pub fn criterion_benchmark(c: &mut Criterion) {
    let data_as_utf8 = String::from_utf8(std::fs::read("map.tmx").unwrap()).unwrap();

    c.bench_function("TMX ", |b| b.iter(|| parse_embedded(data_as_utf8.as_str()).unwrap()));
}

criterion_group!{
    name = benches;
    config = Criterion::default()
        .sample_size(500)
        .measurement_time(std::time::Duration::from_secs(20));
    targets = criterion_benchmark
}
criterion_main!(benches);
