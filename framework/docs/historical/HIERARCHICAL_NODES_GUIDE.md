# Hierarchical Procedural Generation Framework - User Guide

## Overview

The hierarchical procedural generation framework provides a flexible system for managing hierarchical trees of procedural nodes in Bevy. Users can define spatial bounding regions, specify how those regions split into higher levels of detail, and have the framework automatically manage LOD transitions based on viewer distance.

## Implementation Summary

### Files Created/Modified

#### New File
- **`crates/framework/src/hierarchy.rs`** (624 lines)
  - Complete hierarchical node system implementation

#### Modified File
- **`crates/framework/src/lib.rs`**
  - Integrated hierarchy module
  - Removed old simple `Bounds` enum
  - Added re-exports for public API
  - Registered LOD systems in plugin

## Core Concepts

### 1. Spatial Bounding Regions

The framework provides five types of spatial bounds:

```rust
pub enum Bounds {
    Cube { center: Vec3, half_extent: f32 },
    Aabb { min: Vec3, max: Vec3 },
    Sphere { center: Vec3, radius: f32 },
    Obb { center: Vec3, half_extents: Vec3, rotation: Quat },
    Cylinder { center: Vec3, height: f32, radius: f32, axis: Vec3 },
}
```

**Key Methods:**
- `view_area(&self, viewer_transform: &GlobalTransform) -> f32` - Calculates apparent size from viewer's perspective (for LOD)
- `center(&self) -> Vec3` - Gets the approximate center point
- `max_extent(&self) -> f32` - Gets the maximum extent/radius
- `transform(&self, transform: &Transform) -> Self` - Transforms bounds by a spatial transform

### 2. Subdivision Strategies

The `Subdivision` trait allows flexible spatial splitting patterns:

```rust
pub trait Subdivision: Send + Sync + 'static {
    fn subdivide(&self, parent: &Bounds) -> Vec<(Bounds, Transform)>;
    fn child_count(&self) -> usize;
}
```

#### Built-in Implementations

**OctreeSubdivision** - 8-way split into octants
```rust
HierarchyBuilder::new(bounds, node)
    .with_subdivision(OctreeSubdivision)
    .spawn(&mut commands);
```

**QuadtreeSubdivision** - 4-way split along a plane (great for terrain)
```rust
HierarchyBuilder::new(bounds, node)
    .with_subdivision(QuadtreeSubdivision {
        split_axis: QuadtreeAxis::XZ, // Horizontal plane
    })
    .spawn(&mut commands);
```

**BinarySubdivision** - 2-way split along an axis
```rust
HierarchyBuilder::new(bounds, node)
    .with_subdivision(BinarySubdivision {
        axis: BinaryAxis::Y, // Split along Y axis
    })
    .spawn(&mut commands);
```

**CustomSubdivision** - User-provided closure for completely custom patterns
```rust
let custom = CustomSubdivision::new(6, |parent_bounds| {
    match parent_bounds {
        Bounds::Sphere { center, radius } => {
            // Create 6 cube faces around sphere
            let face_size = radius / 2.0;
            vec![
                (Bounds::Cube {
                    center: *center + Vec3::X * radius,
                    half_extent: face_size
                }, Transform::from_translation(*center + Vec3::X * radius)),
                // ... 5 more faces
            ]
        }
        _ => vec![],
    }
});

HierarchyBuilder::new(bounds, node)
    .with_subdivision(custom)
    .spawn(&mut commands);
```

### 3. HierarchicalNode Component

The `HierarchicalNode` component manages hierarchical behavior:

```rust
#[derive(Component)]
pub struct HierarchicalNode {
    pub bounds: Bounds,
    pub subdivision: Option<Box<dyn Subdivision>>,
    pub level: u32,
    pub max_depth: u32,
}
```

**Key Methods:**
- `new(bounds: Bounds) -> Self` - Create a new hierarchical node
- `with_subdivision(self, subdivision: impl Subdivision) -> Self` - Set subdivision strategy
- `with_max_depth(self, max_depth: u32) -> Self` - Set maximum recursion depth
- `can_subdivide(&self) -> bool` - Check if node can split
- `generate_children(&self) -> Option<Vec<(Bounds, Transform)>>` - Generate child bounds

### 4. HierarchyBuilder

Ergonomic builder pattern for creating hierarchical trees:

```rust
pub struct HierarchyBuilder<T: ProceduralNode>

impl<T: ProceduralNode + Component> HierarchyBuilder<T> {
    pub fn new(bounds: Bounds, node: T) -> Self;
    pub fn with_subdivision(self, subdivision: impl Subdivision) -> Self;
    pub fn with_max_depth(self, max_depth: u32) -> Self;
    pub fn spawn(self, commands: &mut Commands) -> Entity;
}
```

## Automatic LOD Management

The framework integrates with the existing `ProckitFrameworkConfig` and `Viewer` component to automatically manage LOD transitions.

### Systems

Two systems run in the `Update` schedule:

**`split_hierarchical_nodes`**
- Monitors leaf nodes (nodes without children)
- Calculates `view_area` from all viewers
- Splits nodes when `view_area > upper_visibility_limit`
- Only splits non-empty nodes (`!procedural_node.empty()`)

**`merge_hierarchical_nodes`**
- Monitors branch nodes (nodes with children)
- Calculates `view_area` from all viewers
- Merges (despawns children) when `view_area < lower_visibility_limit`
- Also merges empty nodes regardless of distance

### Configuration

Configure LOD thresholds via `ProckitFrameworkConfig`:

```rust
app.insert_resource(ProckitFrameworkConfig {
    upper_visibility_limit: 10.0,  // Split threshold
    lower_visibility_limit: 1.0,   // Merge threshold
});
```

### Viewer Setup

Mark entities as viewers:

```rust
commands.spawn((
    Camera3d::default(),
    Viewer, // This component marks the viewpoint
    Transform::from_xyz(0.0, 5.0, 10.0),
    GlobalTransform::default(),
));
```

## Complete Usage Example

Here's a complete example showing how to create a hierarchical terrain system:

```rust
use bevy::prelude::*;
use prockit_framework::{*, hierarchy::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ProckitFrameworkPlugin)
        .add_plugins(ProceduralNodePlugin::<TerrainNode>::default())
        .insert_resource(ProckitFrameworkConfig {
            upper_visibility_limit: 10.0,
            lower_visibility_limit: 1.0,
        })
        .add_systems(Startup, setup)
        .run();
}

#[derive(Component, Default)]
struct TerrainNode {
    density: Vec<f32>,
}

impl ProceduralNode for TerrainNode {
    fn generate(&mut self, bounds: &Bounds, provider: &Provides) {
        // Generate terrain density field for this bounds region
        let center = bounds.center();
        let extent = bounds.max_extent();

        // Example: simple density based on position
        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    let pos = center + Vec3::new(
                        (x as f32 - 16.0) * extent / 16.0,
                        (y as f32 - 16.0) * extent / 16.0,
                        (z as f32 - 16.0) * extent / 16.0,
                    );

                    // Simple noise function (replace with actual noise)
                    let density = (pos.y + 10.0 * (pos.x * 0.1).sin()) / 20.0;
                    self.density.push(density);
                }
            }
        }
    }

    fn provides() -> Provides {
        Provides::new()
    }

    fn empty(&self) -> bool {
        self.density.is_empty()
    }
}

fn setup(mut commands: Commands) {
    // Create viewer/camera
    commands.spawn((
        Camera3d::default(),
        Viewer,
        Transform::from_xyz(0.0, 50.0, 100.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
    ));

    // Create hierarchical terrain root
    let terrain_bounds = Bounds::Cube {
        center: Vec3::ZERO,
        half_extent: 1000.0,
    };

    HierarchyBuilder::new(terrain_bounds, TerrainNode::default())
        .with_subdivision(OctreeSubdivision)
        .with_max_depth(8)
        .spawn(&mut commands);
}
```

## Advanced Examples

### Example 1: Quadtree Terrain

Perfect for 2D height-based terrain:

```rust
let terrain_bounds = Bounds::Cube {
    center: Vec3::ZERO,
    half_extent: 512.0,
};

HierarchyBuilder::new(terrain_bounds, TerrainNode::default())
    .with_subdivision(QuadtreeSubdivision {
        split_axis: QuadtreeAxis::XZ, // Horizontal plane
    })
    .with_max_depth(6)
    .spawn(&mut commands);
```

### Example 2: Planetary LOD with Custom Subdivision

Non-uniform subdivision for planetary surfaces:

```rust
let planet_subdivision = CustomSubdivision::new(6, |parent_bounds| {
    match parent_bounds {
        Bounds::Sphere { center, radius } => {
            let face_size = radius * 0.5;
            vec![
                // +X face
                (Bounds::Cube {
                    center: *center + Vec3::X * radius,
                    half_extent: face_size
                }, Transform::from_translation(*center + Vec3::X * radius)),
                // -X face
                (Bounds::Cube {
                    center: *center - Vec3::X * radius,
                    half_extent: face_size
                }, Transform::from_translation(*center - Vec3::X * radius)),
                // ... +Y, -Y, +Z, -Z faces
            ]
        }
        Bounds::Cube { center, half_extent } => {
            // Subdivide cube faces into quadtree
            // ... implementation
            vec![]
        }
        _ => vec![],
    }
});

HierarchyBuilder::new(
    Bounds::Sphere {
        center: Vec3::ZERO,
        radius: 6371.0  // Earth radius in km
    },
    PlanetNode::new()
)
.with_subdivision(planet_subdivision)
.with_max_depth(12)
.spawn(&mut commands);
```

### Example 3: Asymmetric Binary Splits

Useful for streaming worlds or regions with different detail requirements:

```rust
HierarchyBuilder::new(bounds, node)
    .with_subdivision(BinarySubdivision {
        axis: BinaryAxis::X,  // Split along X axis
    })
    .with_max_depth(10)
    .spawn(&mut commands);
```

## Integration with Volume Crate

The framework includes commented template code for integrating with the volume crate. To attach generated chunks to hierarchical nodes:

1. Uncomment the volume attachment code in `hierarchy.rs`
2. Implement the `VolumeNode` trait on your procedural node
3. Add the `attach_volumes_to_leaves` system to your app

Example integration:

```rust
impl VolumeNode<MyVoxel, 32> for TerrainNode {
    fn generate_chunk(&mut self, bounds: &Bounds, rules: &impl Rules<MyVoxel, 32>) -> Chunk<MyVoxel, 32> {
        // Convert bounds to chunk
        let mut chunk = Chunk::default();

        // Use rules to generate voxels based on density data
        for position in Position::iter() {
            let voxel = rules.generate(position);
            chunk.set(position, voxel);
        }

        chunk
    }
}
```

## Best Practices

1. **Choose the Right Subdivision Pattern**
   - Octree: Uniform 3D detail (voxel terrain, volume effects)
   - Quadtree: Height-based terrain, 2D games with depth
   - Binary: Streaming worlds, asymmetric detail requirements
   - Custom: Planetary rendering, special geometries

2. **Set Appropriate max_depth**
   - Start conservative (6-8 levels)
   - Monitor performance and adjust
   - Consider memory constraints (8^depth nodes for octrees)

3. **Tune LOD Thresholds**
   - `upper_visibility_limit`: Higher = splits later (less detail)
   - `lower_visibility_limit`: Higher = merges sooner (less detail preserved)
   - Balance between visual quality and performance

4. **Optimize ProceduralNode::empty()**
   - This is called frequently by LOD systems
   - Keep it fast (simple boolean check ideally)
   - Use to skip generating empty regions (air, deep underground, etc.)

5. **Integration with Rendering**
   - Generate meshes in `ProceduralNode::generate()`
   - Use the volume crate for voxel-based meshing
   - Consider async generation for heavy procedural work

## API Reference

### Exports from `prockit_framework::hierarchy`

```rust
pub use hierarchy::{
    // Core types
    Bounds,
    Subdivision,
    HierarchicalNode,
    HierarchicalRoot,
    HierarchyBuilder,

    // Subdivision implementations
    OctreeSubdivision,
    QuadtreeSubdivision,
    QuadtreeAxis,
    BinarySubdivision,
    BinaryAxis,
    CustomSubdivision,

    // Systems (automatically registered via ProckitFrameworkPlugin)
    split_hierarchical_nodes,
    merge_hierarchical_nodes,
};
```

## Known Limitations

1. **Subdivision Cloning**: Currently, child nodes don't inherit the parent's subdivision strategy. This is a known limitation and may be addressed in future versions.

2. **Dynamic Dispatch Overhead**: The `Subdivision` trait uses `Box<dyn Subdivision>` for maximum flexibility, which has a small runtime cost. For performance-critical applications, consider using handles or resources instead.

3. **Memory Usage**: Each node stores its own bounds and subdivision strategy. For very deep hierarchies, consider sharing subdivision strategies via resources.

## Future Enhancements

Potential future additions to the framework:

- **Transition Zones**: Smooth blending between LOD levels
- **Neighbor Finding**: For mesh stitching across LOD boundaries
- **Async Generation**: Background task support for heavy generation
- **Serialization**: Save/load hierarchical state
- **Frustum Culling**: Skip processing invisible nodes
- **Shared Subdivision Strategies**: Resource-based subdivision sharing

## Troubleshooting

**Nodes aren't splitting:**
- Check that `upper_visibility_limit` is set appropriately
- Ensure `Viewer` component is present on camera
- Verify `ProceduralNode::empty()` returns `false`
- Check that `max_depth` hasn't been reached

**Nodes split too aggressively:**
- Increase `upper_visibility_limit`
- Decrease `max_depth`
- Adjust bounds `view_area()` calculation

**Performance issues:**
- Reduce `max_depth`
- Optimize `ProceduralNode::generate()`
- Consider async generation for heavy work
- Profile to identify bottlenecks

## Conclusion

The hierarchical procedural generation framework provides a powerful, flexible foundation for managing LOD-based procedural content in Bevy. By combining spatial bounding regions, user-defined subdivision strategies, and automatic LOD management, it enables efficient generation of complex procedural worlds that adapt to viewer distance.

For more information, see:
- `HIERARCHICAL_NODES_PLAN.md` - Detailed implementation plan
- `crates/framework/src/hierarchy.rs` - Source code
- `crates/framework/src/lib.rs` - Framework integration
