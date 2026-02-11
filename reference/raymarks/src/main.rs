mod context;
mod shaders;

use context::BenchmarkContext;
use log::info;

#[forbid(unsafe_code)]
#[forbid(missing_docs)]

/// Main entry point for benchmarking.
fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_secs()
        .init();
    run_all();
}

/// Run all benchmarks.
fn run_all() {
    let context = BenchmarkContext::new_sync();
    bunny_rasterization(context, vec![(512, 512)], vec![1000]);
}

/// Benchmark which renders configurable amounts of Stanford bunny models using rasterization
/// at configurable resolutions. At the moment, it only renders a single triangle.
fn bunny_rasterization(
    mut context: BenchmarkContext,
    resolutions: Vec<(u32, u32)>,
    bunny_counts: Vec<u32>,
) {
    for size in resolutions {
        context.resize_render_target(size);
        context.rasterization_pass();
        context.submit();
        context.save_render_target_sync("bunny_rasterization");
    }
    info!("Bunny rasterization benchmark complete.");
}
