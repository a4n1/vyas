use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use vyas::{
    bench_support::{build_voxels, generate_mesh_stats},
    config::RenderConfig,
    position::GridPosition,
};

fn bench_generate_mesh(c: &mut Criterion) {
    let render_config = RenderConfig::default();
    let chunk_position = GridPosition::default();

    let voxels = build_voxels(&render_config);

    let mesh_stats = generate_mesh_stats(
        black_box(&chunk_position),
        black_box(&voxels),
        black_box(&render_config),
    );

    let mut group = c.benchmark_group("generate_mesh");

    group.throughput(Throughput::Elements(render_config.chunk_size.pow(3) as u64));

    group.bench_with_input(
        BenchmarkId::new(
            "generate-mesh",
            format!(
                "vertices={} indices={}",
                mesh_stats.vertices, mesh_stats.indices,
            ),
        ),
        &voxels,
        |b, voxels| {
            b.iter(|| {
                black_box(generate_mesh_stats(
                    black_box(&chunk_position),
                    black_box(voxels),
                    black_box(&render_config),
                ))
            });
        },
    );

    group.finish();
}

criterion_group!(benches, bench_generate_mesh);
criterion_main!(benches);
