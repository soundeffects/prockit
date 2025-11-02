use acore::volumes::Chunk;
use bevy::prelude::*;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use noiz::{math_noise::Pow2, prelude::*};
use rand::{Rng, rng};

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

fn get_generator() -> Box<dyn Fn(Vec3) -> u16> {
    let mut thread_rng = rng();
    match thread_rng.random_range(0..=2) {
        0 => Box::new(|_position: Vec3| rng().random_range(0..=1)),
        1 => {
            let x = thread_rng.random_range(-10.0..=10.0);
            let y = thread_rng.random_range(-10.0..=10.0);
            let z = thread_rng.random_range(-10.0..=10.0);
            Box::new(move |position: Vec3| {
                let sphere_center = Vec3::new(x, y, z);
                let sphere_radius = rng().random_range(1.0..=10.0);
                let distance = position.distance(sphere_center);
                (distance < sphere_radius) as u16
            })
        }
        2 => {
            let heightmap_noise = describe_heightmap_noise();
            Box::new(move |position: Vec3| {
                let height = heightmap_noise.sample(position.xz() / 256.0) * 256.0;
                (position.y < height) as u16
            })
        }
        _ => unreachable!(),
    }
}

fn chunk_bench(c: &mut Criterion) {
    let chunk_sizes: [usize; 5] = [4, 8, 16, 32, 64];
    let mut generators = Vec::with_capacity(100);
    for _ in 0..100 {
        generators.push(get_generator());
    }
    let mut group = c.benchmark_group("chunk_bench");
    let mut thread_rng = rng();
    for chunk_size in chunk_sizes {
        group.throughput(Throughput::Elements(
            (chunk_size * chunk_size * chunk_size) as u64,
        ));
        group.bench_function(format!("generation_{}", chunk_size), |b| {
            b.iter(|| {
                Chunk::generate(
                    chunk_size,
                    1,
                    GlobalTransform::default(),
                    &generators[thread_rng.random_range(0..generators.len())],
                )
            })
        });
        let chunk = Chunk::generate(
            chunk_size,
            1,
            GlobalTransform::default(),
            &generators[thread_rng.random_range(0..generators.len())],
        );
        group.bench_function(format!("meshing_{}", chunk_size), |b| {
            b.iter(|| chunk.mesh(&|v| v != 0))
        });
    }
    group.finish();
}

criterion_group!(chunk_benches, chunk_bench);
criterion_main!(chunk_benches);
