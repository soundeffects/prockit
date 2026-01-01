# AGENTS.md

This file provides guidance to AI agents when working with code in this repository.

## Project Overview

Prockit is a procedural toolkit tightly integrated with the Bevy game engine for generating game and virtual environment elements on the fly (terrain, foliage, creatures, characters, sounds, etc.). The project emphasizes **data abstraction over traditional triangle meshes**, exploring voxels, skeletons, particles, and raymarched rendering with hierarchical level-of-detail (LOD) systems.

**Code Mirrors:**
- GitHub: https://github.com/soundeffects/prockit
- Codeberg: https://codeberg.org/soundeffects/prockit

## Build and Development Commands

### Building
```bash
# Build all crates in workspace
cargo build

# Build specific crate
cargo build -p prockit_volume
cargo build -p prockit_framework
cargo build -p prockit_skeletons
cargo build -p prockit_rasterless
cargo build -p raymarks
cargo build -p prockit_linter

# Release build (uses thin LTO, codegen-units=1)
cargo build --release
```

### Testing
```bash
# Run all tests
cargo test

# Test specific crate
cargo test -p prockit_volume

# Run specific test by name
cargo test <test_name>

# Test without running (compile-check only)
cargo test --no-run
```

### Benchmarking
```bash
# Run all benchmarks (uses criterion)
cargo bench

# Run benchmarks for specific crate
cargo bench -p prockit_volume

# Run specific benchmark
cargo bench <bench_name>
```

### Running Examples
```bash
# Volume crate: First-person voxel viewer
cargo run -p prockit_volume --example fpv

# Skeletons crate: Bone LOD demonstration
cargo run -p prockit_skeletons --example bone_lod

# Rasterless crate: Simple raymarching demo
cargo run -p prockit_rasterless --example simple
```

### Checking Code
```bash
# Fast compile check without building
cargo check

# Check specific crate
cargo check -p prockit_volume
```

## Setup Requirements

**Linux Linker (Recommended):**
The project recommends using `mold` linker for faster compile times on Linux:
- Ubuntu: `sudo apt-get install mold clang`
- Fedora: `sudo dnf install mold clang`
- Arch: `sudo pacman -S mold clang`

**GPU Requirements:**
- Rasterless crate requires `BGRA8UNORM_STORAGE` feature (available on DX12, Vulkan, Metal)
- Raymarks benchmarks prefer high-performance GPU adapter

## Crate Architecture

### prockit_volume (Core Voxel System)
**Purpose:** Hierarchical voxel representation and chunk-based volumetric data with mesh generation.

**Key Abstractions:**
- `Chunk<V, N>`: Generic chunk system where `V` is voxel type and `N` is compile-time dimension
  - Uses 3D array storage `[[[V; N]; N]; N]` for cache efficiency
  - Supports hierarchical LOD through octant-based enhancement (2^3 subdivision)
- `Rules<V, N>` trait: Defines voxel generation, enhancement, and surface detection logic
- `Position<N>`: Type-safe 3D indexing with bounds checking
- `Octant` enum: 8-way subdivision for recursive LOD structure
- `CubicDirection`: 6-directional spatial navigation (Nx, Px, Ny, Py, Nz, Pz)
- `Volume` component: Bevy-integrated voxel volume with transform support
- `Viewer` component: Observer for distance-based level-of-detail

**Mesh Generation:**
- `compile_mesh()`: Converts chunk faces to Bevy `Mesh` with positions, normals, indices
- Interior and boundary face generation for seamless chunk meshing
- `Face` struct: Stores mesh faces with position and normal data

### prockit_framework (Procedural Node Framework)
**Purpose:** Abstract framework for hierarchical procedural generation with distance-based LOD.

**Key Abstractions:**
- `ProceduralNode` trait: Polymorphic interface for generative systems
  - `generate()`: Create content for bounded regions
  - `provides()`: Declare output types this node produces
  - `empty()`: Check if node has generated content
- `Bounds` enum: Geometric regions (currently Cube variant)
- `Viewer` component: Observer triggering distance-based resampling
- `ProvidesMap` resource: Registry of node output types
- `ProceduralNodePlugin<T>`: Generic plugin wrapper using `bevy_trait_query`
- `resample()` system: Distance-based LOD management
  - Upper visibility limit: triggers enhancement (splitting)
  - Lower visibility limit: triggers compression (merging)

**Pattern:** Uses `bevy_trait_query` for polymorphic querying of procedural nodes in ECS.

### prockit_skeletons (Skeletal Animation)
**Purpose:** Tree-structured skeletal hierarchies and procedural bone generation.

**Key Abstractions:**
- `Bone` component: Stores length and angle pair (axial, latitudinal) with `derive()` method for parent context transformations
- `Skeleton` component: Root marker for hierarchical bone structures
- `SkeletonDescriptor`: Declarative DSL for describing skeleton hierarchies
- `SkeletonDescElement`: Recursive tree nodes with bone and children
- `construct_skeletons()` system: Converts descriptors to ECS hierarchy at startup
- `SkeletonGizmosPlugin`: Debug visualization using Bevy gizmos

**Pattern:** Descriptor pattern for declarative hierarchy definition, converting to ECS at runtime. Uses stack-based traversal for non-recursive processing.

### prockit_rasterless (Volumetric Rendering)
**Purpose:** Alternative rendering using compute-shader raymarching instead of rasterization.

**Note:** Uses Bevy 0.14 (different from main workspace's 0.17).

**Key Components:**
- `Graphics`: wgpu rendering backend with pure compute-based rendering
  - Single compute pass for visibility, lighting, post-processing
  - No rasterization pipeline—all effects in shader
- `Camera`: ShaderType camera with position, direction, up, FOV
- `AppState`: winit-based application handler for window management
- `Ellipsoid`: Volumetric shape primitive

### raymarks (Benchmark Suite)
**Purpose:** Isolated performance benchmarking framework for rendering operations.

**Key Components:**
- `BenchmarkContext`: GPU device abstraction for consistent benchmarking
  - Manages large buffers (up to 1GB) for voxel storage
  - Creates render pipelines and saves output to PNG
  - Polling-based async execution with `pollster`

### prockit_linter
Currently minimal placeholder. Likely intended for shader/configuration validation.

## Important Design Patterns

### Hierarchical LOD Strategy
The codebase uses consistent hierarchical LOD across multiple systems:
- **Volume:** Chunk upsampling via octants (2^3 subdivision per enhancement)
- **Framework:** Distance-based resampling with configurable visibility limits
- **Both:** Viewer-centric dynamic detail management

### Procedural Generation Approaches
1. **Rules-based:** `Rules<V, N>` trait for customizable voxel generation
2. **Descriptor DSL:** `SkeletonDescriptor` for declarative bone hierarchies
3. **Trait polymorphism:** `bevy_trait_query` for runtime dispatch in ECS

### Rendering Paradigms
- **Traditional:** Mesh generation from chunk faces → Bevy render pipeline
- **Volumetric:** Compute-shader raymarching (rasterless crate)
- **Hybrid potential:** Combination of skeletal meshes + voxel rendering

## Version Considerations

**Bevy Versions:**
- **0.17:** Main crates (volume, framework, linter)
- **0.14:** Skeletons and rasterless (older implementations)

**Rust Edition:**
- **2024:** Most crates (experimental edition)
- **2021:** raymarks and some older crates

**wgpu Versions:**
- raymarks: 23.0.1
- rasterless: 0.20

When modifying code, be mindful of these version differences. Cross-crate changes may require version alignment.

## Build Profile Configuration

**Development Profile:**
- Main app: opt-level 1
- Dependencies: opt-level 3
- Rationale: Fast iteration with optimized dependencies

**Release Profile:**
- codegen-units: 1
- LTO: thin
- opt-level: 3

## Key Dependencies

**Core:**
- `bevy`: Game engine integration
- `noiz`: Noise generation for procedural content
- `rand`: Randomization

**Rendering:**
- `wgpu`: Low-level GPU API
- `winit`: Window management (rasterless)
- `glam`: Math library

**Development:**
- `criterion`: Benchmarking framework
- `bevy_trait_query`: Trait-based ECS queries
- `env_logger` / `log`: Logging

## Development Stage Notes

**Active Development Areas** (from recent commits):
- Chunk meshing between LOD boundaries
- Chunk internals reorganization
- Volume crate architecture refinement

**Known TODOs:**
- Split/merge volume systems (currently unimplemented)
- Framework has two design versions (current and reference)
- Comprehensive test coverage
