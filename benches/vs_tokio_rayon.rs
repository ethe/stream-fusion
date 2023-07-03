use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use stream_fusion::{execution::Execution, flat::FlatStream, futures, sync::StreamSync};

#[inline(never)]
async fn fusion_chain<I: Iterator<Item = i32> + Send + 'static>(iter: I) -> Option<u64> {
    StreamSync::new(futures::stream::iter(iter))
        .map(|item| {
            let mut s = DefaultHasher::new();
            item.hash(&mut s);
            s.finish()
        })
        .reduce(|r, item| r ^ item)
        .execute(8, 128)
        .await
}

fn fusion(c: &mut Criterion) {
    let range = 0..4096;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(8)
        .enable_all()
        .build()
        .unwrap();

    c.bench_function("fusion", |b| {
        b.to_async(&runtime)
            .iter(|| black_box(fusion_chain(black_box(range.clone()))));
    });
}

criterion_group!(benches, fusion);
criterion_main!(benches);
