# Hierarchical Procedural Generation Framework - Implementation Plan

## Overview
Create a flexible hierarchical node system for procedural generation in Bevy that allows users to define:
- Spatial bounding regions with multiple types (Cube, AABB, Sphere, etc.)
- Flexible user-defined subdivision patterns (not locked to octrees)
- Integration with existing LOD/viewer distance system
- Nodes that generate volumes/chunks hierarchically

## User Requirements
1. **Purpose**: Nodes generate volumes/chunks based on LOD
2. **Bounding regions**: Extend existing `Bounds` enum with more variants and actual data
3. **Splitting**: Flexible user-defined splits - any number of children with custom spatial subdivisions
4. **LOD Integration**: Use existing Viewer/visibility limits/resample() system

## Architecture

### 1. Enhanced Bounds System
**Location**: `/home/james/source/prockit/crates/framework/src/hierarchy.rs` (new file)

The current `Bounds` enum at framework/src/lib.rs:30 is just a unit variant `Cube` with no data. Replace it with:

```rust
#[derive(Clone, Debug)]
pub enum Bounds {
    Cube { center: Vec3, half_extent: f32 },
    Aabb { min: Vec3, max: Vec3 },
    Sphere { center: Vec3, radius: f32 },
    Obb { center: Vec3, half_extents: Vec3, rotation: Quat },
    Cylinder { center: Vec3, height: f32, radius: f32, axis: Vec3 },
}

impl Bounds {
    pub fn view_area(&self, viewer_transform: &GlobalTransform) -> f32;
    pub fn center(&self) -> Vec3;
    pub fn max_extent(&self) -> f32;
    pub fn transform(&self, transform: &Transform) -> Self;
}
```

### 2. Subdivision Trait System
**Location**: `/home/james/source/prockit/crates/framework/src/hierarchy.rs`

Generic interface for flexible splitting:

```rust
pub trait Subdivision: Send + Sync + 'static {
    fn subdivide(&self, parent: &Bounds) -> Vec<(Bounds, Transform)>;
    fn child_count(&self) -> usize;
}
```

Built-in implementations:
- `OctreeSubdivision` - 8-way split (mirrors volume crate's Octant pattern)
- `QuadtreeSubdivision { split_axis: QuadtreeAxis }` - 4-way split for terrain
- `BinarySubdivision { axis: BinaryAxis }` - 2-way split along one axis
- `CustomSubdivision` - user-provided closure for arbitrary patterns

### 3. Hierarchical Node Component
**Location**: `/home/james/source/prockit/crates/framework/src/hierarchy.rs`

```rust
#[derive(Component)]
pub struct HierarchicalNode {
    pub bounds: Bounds,
    pub subdivision: Option<Box<dyn Subdivision>>,
    pub level: u32,
    pub max_depth: u32,
}

impl HierarchicalNode {
    pub fn new(bounds: Bounds) -> Self;
    pub fn with_subdivision(self, subdivision: impl Subdivision) -> Self;
    pub fn with_max_depth(self, max_depth: u32) -> Self;
    pub fn can_subdivide(&self) -> bool;
    pub fn generate_children(&self) -> Option<Vec<(Bounds, Transform)>>;
}
```

Builder pattern for ergonomics:

```rust
pub struct HierarchyBuilder<T: ProceduralNode> {
    bounds: Bounds,
    subdivision: Option<Box<dyn Subdivision>>,
    max_depth: u32,
    node: T,
}

impl<T: ProceduralNode + Component> HierarchyBuilder<T> {
    pub fn spawn(self, commands: &mut Commands) -> Entity;
}
```

### 4. LOD Integration Systems
**Location**: `/home/james/source/prockit/crates/framework/src/hierarchy.rs`

Two new systems to add to Update schedule:

```rust
pub fn split_hierarchical_nodes(
    commands: Commands,
    config: Res<ProckitFrameworkConfig>,
    viewers: Query<&GlobalTransform, With<Viewer>>,
    nodes: Query<(Entity, &HierarchicalNode, &GlobalTransform, One<&dyn ProceduralNode>), Without<Children>>,
    provides_map: Res<ProvidesMap>,
)
```

- Checks if leaf nodes exceed `upper_visibility_limit`
- Calls `HierarchicalNode::generate_children()`
- Spawns child entities with increased `level`

```rust
pub fn merge_hierarchical_nodes(
    commands: Commands,
    config: Res<ProckitFrameworkConfig>,
    viewers: Query<&GlobalTransform, With<Viewer>>,
    nodes: Query<(Entity, &HierarchicalNode, &GlobalTransform, One<&dyn ProceduralNode>, &Children)>,
)
```

- Checks if branch nodes fall below `lower_visibility_limit`
- Despawns all children recursively

### 5. Volume Attachment (Optional Extension)
**Location**: `/home/james/source/prockit/crates/framework/src/hierarchy.rs`

For connecting generated chunks to hierarchical nodes:

```rust
#[derive(Component)]
pub struct VolumeAttachment<V: Voxel, const N: usize> {
    pub chunk: Chunk<V, N>,
}

pub trait VolumeNode<V: Voxel, const N: usize>: ProceduralNode {
    fn generate_chunk(&mut self, bounds: &Bounds, rules: &impl Rules<V, N>) -> Chunk<V, N>;
}
```

## File Changes

### New File
- **`/home/james/source/prockit/crates/framework/src/hierarchy.rs`**
  - Complete implementation of bounds, subdivision, components, and systems

### Modified Files
- **`/home/james/source/prockit/crates/framework/src/lib.rs`**
  - Add `mod hierarchy;` at line 5 (after `mod provides;`)
  - Remove old `Bounds` enum definition (lines 30-46)
  - Add re-exports: `pub use hierarchy::{Bounds, Subdivision, HierarchicalNode, HierarchyBuilder, OctreeSubdivision, QuadtreeSubdivision, BinarySubdivision};`
  - Add systems to `ProckitFrameworkPlugin::build()`:
    ```rust
    .add_systems(Update, (resample, split_hierarchical_nodes, merge_hierarchical_nodes))
    ```

## Implementation Steps

1. **Create hierarchy.rs with enhanced Bounds**
   - Define new `Bounds` enum with all variants and data fields
   - Implement `view_area()`, `center()`, `max_extent()`, `transform()`
   - Add unit tests for bounds operations

2. **Implement Subdivision trait and built-ins**
   - Define `Subdivision` trait
   - Implement `OctreeSubdivision` using octant pattern (reference: volume/src/octant.rs)
   - Implement `QuadtreeSubdivision` and `BinarySubdivision`
   - Implement `CustomSubdivision` wrapper

3. **Add HierarchicalNode component**
   - Define component with bounds, subdivision, level, max_depth
   - Implement builder methods
   - Create `HierarchyBuilder` for ergonomic entity spawning

4. **Implement LOD systems**
   - `split_hierarchical_nodes` - spawns children when view_area exceeds upper limit
   - `merge_hierarchical_nodes` - despawns children when view_area below lower limit
   - Helper functions for node splitting/merging

5. **Integrate with existing framework**
   - Update lib.rs to import hierarchy module
   - Remove old Bounds definition
   - Add re-exports
   - Register systems in plugin

6. **Add volume attachment (optional)**
   - Define `VolumeAttachment` component
   - Define `VolumeNode` trait extension
   - Create example showing integration with volume crate

## Usage Example

```rust
use prockit_framework::{*, hierarchy::*};

#[derive(Component)]
struct TerrainNode {
    density: Vec<f32>,
}

impl ProceduralNode for TerrainNode {
    fn generate(&mut self, bounds: &Bounds, provider: &Provides) {
        // Generate terrain for this bounds region
    }

    fn provides() -> Provides {
        Provides::new()
    }

    fn empty(&self) -> bool {
        self.density.is_empty()
    }
}

fn setup(mut commands: Commands) {
    let bounds = Bounds::Cube {
        center: Vec3::ZERO,
        half_extent: 1000.0,
    };

    HierarchyBuilder::new(bounds, TerrainNode { density: Vec::new() })
        .with_subdivision(OctreeSubdivision)
        .with_max_depth(8)
        .spawn(&mut commands);
}
```

## Key Design Decisions

1. **Flexible subdivision**: Users can provide any subdivision pattern via trait, not locked to octrees
2. **Type-erased subdivision**: Using `Box<dyn Subdivision>` for flexibility (could optimize later with handles)
3. **Integration with existing LOD**: Reuses ProckitFrameworkConfig visibility limits and Viewer component
4. **Child generation on-demand**: Children are spawned/despawned by systems based on viewer distance
5. **Volume attachment optional**: Can be used independently or integrated with volume crate

## Trade-offs

**Pros:**
- Maximum flexibility for subdivision patterns
- Clean integration with existing framework
- Type-safe bounds operations
- Extensible to new bounds types

**Cons:**
- More complex than fixed octree
- Dynamic dispatch overhead for Subdivision trait
- Memory overhead storing subdivision in each node (could share via resources)

## References

- Existing `ProceduralNode` trait: framework/src/lib.rs:19-28
- Current `Bounds` (replaced): framework/src/lib.rs:30-46
- LOD resample system: framework/src/lib.rs:94-142
- Octant pattern: volume/src/octant.rs
- Chunk enhancement: volume/src/chunk/mod.rs
- Rules trait: volume/src/chunk/rules.rs:3-9

## Implementation Status

All implementation steps completed:
- ✅ Enhanced Bounds system with 5 bounding region types
- ✅ Subdivision trait with 4 built-in implementations
- ✅ HierarchicalNode component and HierarchyBuilder
- ✅ LOD integration systems (split/merge)
- ✅ Integration with existing framework in lib.rs
- ✅ Volume attachment documentation (commented template)
- ✅ Code compiles successfully
