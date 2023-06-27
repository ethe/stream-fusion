#![feature(iter_next_chunk)]

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rayon::prelude::*;
use stream_fusion::{IntoFusion, WindowIterator};
use tokio::task::spawn_blocking;

#[inline(never)]
async fn fusion_chain<I: Iterator<Item = i32>, const N: usize>(iter: I) -> Option<u64> {
    WindowIterator::<_, 256>::from(iter)
        .fusion()
        .map(|item| {
            let mut s = DefaultHasher::new();
            item.hash(&mut s);
            s.finish()
        })
        .reduce::<_, N>(|r, item| r ^ item)
        .await
        .unwrap()
}

#[inline(never)]
fn sync_chain<I: Iterator<Item = i32>>(iter: I) -> Option<u64> {
    iter.map(|item| {
        let mut s = DefaultHasher::new();
        item.hash(&mut s);
        s.finish()
    })
    .map(|item| {
        let mut s = DefaultHasher::new();
        item.hash(&mut s);
        s.finish()
    })
    .map(|item| {
        let mut s = DefaultHasher::new();
        item.hash(&mut s);
        s.finish()
    })
    .reduce(|r, item| r ^ item)
}

#[inline(never)]
async fn rayon_chain<I: Iterator<Item = i32>>(mut iter: I) -> Option<u64> {
    let pool = Arc::new(
        rayon::ThreadPoolBuilder::new()
            .num_threads(8)
            .build()
            .unwrap(),
    );
    let mut tasks = Vec::with_capacity(8);
    loop {
        let pool = pool.clone();
        match iter.next_chunk::<256>() {
            Ok(item) => tasks.push(spawn_blocking(move || {
                pool.install(move || {
                    item.into_par_iter()
                        .map(|item| {
                            let mut s = DefaultHasher::new();
                            item.hash(&mut s);
                            s.finish()
                        })
                        .map(|item| {
                            let mut s = DefaultHasher::new();
                            item.hash(&mut s);
                            s.finish()
                        })
                        .map(|item| {
                            let mut s = DefaultHasher::new();
                            item.hash(&mut s);
                            s.finish()
                        })
                        .sum()
                })
            })),
            Err(iter) => {
                let v = iter.collect::<Vec<_>>();
                tasks.push(spawn_blocking(move || {
                    pool.install(move || {
                        v.into_par_iter()
                            .map(|item| {
                                let mut s = DefaultHasher::new();
                                item.hash(&mut s);
                                s.finish()
                            })
                            .map(|item| {
                                let mut s = DefaultHasher::new();
                                item.hash(&mut s);
                                s.finish()
                            })
                            .map(|item| {
                                let mut s = DefaultHasher::new();
                                item.hash(&mut s);
                                s.finish()
                            })
                            .sum::<u64>()
                    })
                }));
                break;
            }
        }
    }

    let mut i = 0;
    for task in tasks {
        i += task.await.unwrap();
    }
    Some(i)
}

fn fusion(c: &mut Criterion) {
    let range = 0..4096;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(8)
        .enable_all()
        .build()
        .unwrap();

    c.bench_with_input(
        BenchmarkId::new("fusion with 32", range.len()),
        &range,
        |b, range| {
            b.to_async(&runtime)
                .iter(|| black_box(fusion_chain::<_, 32>(black_box(range.clone()))));
        },
    );
}

fn sync(c: &mut Criterion) {
    let range = 0..4096;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(8)
        .enable_all()
        .build()
        .unwrap();

    c.bench_with_input(BenchmarkId::new("sync", range.len()), &range, |b, range| {
        b.to_async(&runtime)
            .iter(|| async { black_box(sync_chain(black_box(range.clone()))) });
    });
}

fn rayon(c: &mut Criterion) {
    let range = 0..4096;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(8)
        .enable_all()
        .build()
        .unwrap();

    c.bench_with_input(
        BenchmarkId::new("spawn_blocking + rayon", range.len()),
        &range,
        |b, range| {
            b.to_async(&runtime)
                .iter(|| black_box(rayon_chain(black_box(range.clone()))));
        },
    );
}

fn pure_rayon(c: &mut Criterion) {
    let range = 0..4096;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(8)
        .build()
        .unwrap();

    c.bench_with_input(
        BenchmarkId::new("pure rayon", range.len()),
        &range,
        |b, range| {
            b.iter(|| {
                black_box(pool.install(|| {
                    black_box(range.clone())
                        .into_par_iter()
                        .map(|item| {
                            let mut s = DefaultHasher::new();
                            item.hash(&mut s);
                            s.finish()
                        })
                        .map(|item| {
                            let mut s = DefaultHasher::new();
                            item.hash(&mut s);
                            s.finish()
                        })
                        .map(|item| {
                            let mut s = DefaultHasher::new();
                            item.hash(&mut s);
                            s.finish()
                        })
                        .sum::<u64>()
                }))
            });
        },
    );
}

criterion_group!(benches, fusion, sync, rayon, pure_rayon);
criterion_main!(benches);
