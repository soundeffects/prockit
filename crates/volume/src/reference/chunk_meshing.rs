use acore::volumes::Chunk;
use bevy::prelude::*;
use criterion::{criterion_group, criterion_main, Criterion};
use noiz::{prelude::*, math_noise::Pow2};
use rand::{rng, Rng};

fn random_voxels(_position: Vec3) -> u16 {
    rng().random_range(0..=1)
}

fn smooth_voxels(position: Vec3) -> u16 {
    let x = rng().random_range(-10.0..=10.0);
    let y = rng().random_range(-10.0..=10.0);
    let z = rng().random_range(-10.0..=10.0);
    let sphere_center = Vec3::new(x, y, z);
    let sphere_radius = rng().random_range(1.0..=10.0);
    let distance = position.distance(sphere_center);
    if distance < sphere_radius {
        1
    } else {
        0
    }
}

fn describe_heightmap_noise() -> impl SampleableFor<Vec2, f32> {
    Noise {
        noise: Masked(
            LayeredNoise::new(
                NormedByDerivative::<f32, EuclideanLength, PeakDerivativeContribution>::default()
                    .with_falloff(0.3),
                Persistence(0.6),
                FractalLayers {
                    layer: Octave(BlendCellGradients::<
                        SimplexGrid,
                        SimplecticBlend,
                        QuickGradients,
                        true,
                    >::default()),
                    lacunarity: 1.8,
                    amount: 8,
                },
            ),
            (
                MixCellGradients::<OrthoGrid, Smoothstep, QuickGradients>::default(),
                SNormToUNorm,
                Pow2,
                RemapCurve::<Lerped<f32>, f32, false>::from(Lerped {
                    start: 0.5f32,
                    end: 1.0,
                }),
            ),
        ),
        ..default()
    }
}

fn heightmap_voxels(position: Vec3) -> u16 {
    let height = describe_heightmap_noise().sample(position.xz() / 256.0);
    if position.y < height {
        1
    } else {
        0
    }
}

fn random_16_chunk_meshing(c: &mut Criterion) {
    let chunk = Chunk::generate(16, 1, GlobalTransform::default(), random_voxels);
    c.bench_function("random_16_chunk_meshing", |b| b.iter(|| chunk.mesh(&|v| v != 0)));
}

fn smooth_16_chunk_meshing(c: &mut Criterion) {
    let chunk = Chunk::generate(16, 1, GlobalTransform::default(), smooth_voxels);
    c.bench_function("smooth_chunk_meshing", |b| b.iter(|| chunk.mesh(&|v| v != 0)));
}

fn heightmap_16_chunk_meshing(c: &mut Criterion) {
    let chunk = Chunk::generate(16, 1, GlobalTransform::default(), heightmap_voxels);
    c.bench_function("heightmap_16_chunk_meshing", |b| b.iter(|| chunk.mesh(&|v| v != 0)));
}

fn random_32_chunk_meshing(c: &mut Criterion) {
    let chunk = Chunk::generate(32, 1, GlobalTransform::default(), random_voxels);
    c.bench_function("random_32_chunk_meshing", |b| b.iter(|| chunk.mesh(&|v| v != 0)));
}

fn smooth_32_chunk_meshing(c: &mut Criterion) {
    let chunk = Chunk::generate(32, 1, GlobalTransform::default(), smooth_voxels);
    c.bench_function("smooth_32_chunk_meshing", |b| b.iter(|| chunk.mesh(&|v| v != 0)));
}

fn heightmap_32_chunk_meshing(c: &mut Criterion) {
    let chunk = Chunk::generate(32, 1, GlobalTransform::default(), heightmap_voxels);
    c.bench_function("heightmap_32_chunk_meshing", |b| b.iter(|| chunk.mesh(&|v| v != 0)));
}

criterion_group!(chunk_meshing_16_benches, random_16_chunk_meshing, smooth_16_chunk_meshing, heightmap_16_chunk_meshing);
criterion_group!(chunk_meshing_32_benches, random_32_chunk_meshing, smooth_32_chunk_meshing, heightmap_32_chunk_meshing);
criterion_main!(chunk_meshing_16_benches, chunk_meshing_32_benches);