//! Benchmark for usemeter operations

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use usemeter::{Event, Meter, StorageBackend};
use usemeter::storage::SqliteBackend;
use chrono::Utc;

fn bench_record_event(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("record_event", |b| {
        b.to_async(&rt).iter(|| {
            let storage = SqliteBackend::new_in_memory().unwrap();
            let meter = Meter::new(storage);

            async {
                meter.initialize().await.unwrap();

                let event = Event::builder()
                    .event_type("api_call")
                    .user_id("user-123")
                    .metric("tokens", 1000)
                    .metric("duration_ms", 250)
                    .build()
                    .unwrap();

                meter.record(event).await.unwrap();
            }
        });
    });
}

fn bench_record_batch(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("record_batch");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.to_async(&rt).iter(|| {
                let storage = SqliteBackend::new_in_memory().unwrap();
                let meter = Meter::new(storage);

                async {
                    meter.initialize().await.unwrap();

                    let events: Vec<_> = (0..size)
                        .map(|i| {
                            Event::builder()
                                .event_type("api_call")
                                .user_id(&format!("user-{}", i))
                                .metric("tokens", 1000)
                                .build()
                                .unwrap()
                        })
                        .collect();

                    meter.record_batch(events).await.unwrap();
                }
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_record_event, bench_record_batch);
criterion_main!(benches);
