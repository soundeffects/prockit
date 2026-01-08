use crate::{ProceduralNode, ProckitFrameworkConfig, ProvidesMap, Viewer};
use bevy::prelude::*;
use bevy_trait_query::One;

/// Spatial bounding region that can be subdivided hierarchically
#[derive(Clone, Debug)]
pub enum Bounds {
    /// Axis-aligned cube with center and half-extent
    Cube { center: Vec3, half_extent: f32 },

    /// Axis-aligned bounding box with min/max corners
    Aabb { min: Vec3, max: Vec3 },

    /// Sphere with center and radius
    Sphere { center: Vec3, radius: f32 },

    /// Oriented bounding box with center, half-extents, and rotation
    Obb {
        center: Vec3,
        half_extents: Vec3,
        rotation: Quat,
    },

    /// Cylinder with base center, height, and radius
    Cylinder {
        center: Vec3,
        height: f32,
        radius: f32,
        axis: Vec3, // normalized axis direction
    },
}

impl Bounds {
    /// Approximate view area from a given viewpoint (for LOD calculations)
    /// This is used by the resample() system to determine splitting/merging
    pub fn view_area(&self, viewer_transform: &GlobalTransform) -> f32 {
        match self {
            Bounds::Cube {
                center,
                half_extent,
            } => {
                let distance = viewer_transform.translation().distance(*center);
                let size = half_extent * 2.0;
                (size * size) / (distance * distance + 1.0)
            }
            Bounds::Aabb { min, max } => {
                let center = (*min + *max) * 0.5;
                let size = *max - *min;
                let distance = viewer_transform.translation().distance(center);
                let max_extent = size.max_element();
                (max_extent * max_extent) / (distance * distance + 1.0)
            }
            Bounds::Sphere { center, radius } => {
                let distance = viewer_transform.translation().distance(*center);
                let area = std::f32::consts::PI * radius * radius;
                area / (distance * distance + 1.0)
            }
            Bounds::Obb {
                center,
                half_extents,
                ..
            } => {
                let distance = viewer_transform.translation().distance(*center);
                let max_extent = half_extents.max_element();
                (max_extent * max_extent * 4.0) / (distance * distance + 1.0)
            }
            Bounds::Cylinder {
                center,
                height,
                radius,
                ..
            } => {
                let distance = viewer_transform.translation().distance(*center);
                let max_dimension = height.max(*radius);
                (max_dimension * max_dimension) / (distance * distance + 1.0)
            }
        }
    }

    /// Get the approximate center of the bounds
    pub fn center(&self) -> Vec3 {
        match self {
            Bounds::Cube { center, .. } => *center,
            Bounds::Aabb { min, max } => (*min + *max) * 0.5,
            Bounds::Sphere { center, .. } => *center,
            Bounds::Obb { center, .. } => *center,
            Bounds::Cylinder { center, .. } => *center,
        }
    }

    /// Get an approximate maximum extent (for distance calculations)
    pub fn max_extent(&self) -> f32 {
        match self {
            Bounds::Cube { half_extent, .. } => *half_extent,
            Bounds::Aabb { min, max } => (*max - *min).max_element() * 0.5,
            Bounds::Sphere { radius, .. } => *radius,
            Bounds::Obb { half_extents, .. } => half_extents.max_element(),
            Bounds::Cylinder { height, radius, .. } => height.max(*radius),
        }
    }

    /// Transform bounds by a spatial transform (for positioning child bounds)
    pub fn transform(&self, transform: &Transform) -> Self {
        match self {
            Bounds::Cube {
                center,
                half_extent,
            } => Bounds::Cube {
                center: transform.transform_point(*center),
                half_extent: *half_extent * transform.scale.max_element(),
            },
            Bounds::Aabb { min, max } => {
                let new_min = transform.transform_point(*min);
                let new_max = transform.transform_point(*max);
                Bounds::Aabb {
                    min: new_min.min(new_max),
                    max: new_min.max(new_max),
                }
            }
            Bounds::Sphere { center, radius } => Bounds::Sphere {
                center: transform.transform_point(*center),
                radius: *radius * transform.scale.max_element(),
            },
            Bounds::Obb {
                center,
                half_extents,
                rotation,
            } => Bounds::Obb {
                center: transform.transform_point(*center),
                half_extents: *half_extents * transform.scale,
                rotation: transform.rotation * *rotation,
            },
            Bounds::Cylinder {
                center,
                height,
                radius,
                axis,
            } => Bounds::Cylinder {
                center: transform.transform_point(*center),
                height: *height * transform.scale.y,
                radius: *radius * transform.scale.x.max(transform.scale.z),
                axis: transform.rotation * *axis,
            },
        }
    }
}

/// Describes how a spatial region subdivides into child regions
pub trait Subdivision: Send + Sync + 'static {
    /// Generate child bounds from a parent bounds
    /// Returns a vector of (child_bounds, local_transform) pairs
    fn subdivide(&self, parent: &Bounds) -> Vec<(Bounds, Transform)>;

    /// Number of children this subdivision produces (for optimization/validation)
    fn child_count(&self) -> usize {
        // Default implementation calls subdivide, but can be overridden for efficiency
        self.subdivide(&Bounds::Cube {
            center: Vec3::ZERO,
            half_extent: 1.0,
        })
        .len()
    }
}

/// Octree subdivision: splits a cube into 8 equal sub-cubes
#[derive(Clone, Copy, Debug)]
pub struct OctreeSubdivision;

impl Subdivision for OctreeSubdivision {
    fn subdivide(&self, parent: &Bounds) -> Vec<(Bounds, Transform)> {
        match parent {
            Bounds::Cube {
                center,
                half_extent,
            } => {
                let quarter_extent = half_extent / 2.0;
                let mut children = Vec::with_capacity(8);

                // Use octant pattern from volume crate
                for octant in [
                    Vec3::new(-1.0, -1.0, -1.0), // NxNyNz
                    Vec3::new(-1.0, -1.0, 1.0),  // NxNyPz
                    Vec3::new(-1.0, 1.0, -1.0),  // NxPyNz
                    Vec3::new(-1.0, 1.0, 1.0),   // NxPyPz
                    Vec3::new(1.0, -1.0, -1.0),  // PxNyNz
                    Vec3::new(1.0, -1.0, 1.0),   // PxNyPz
                    Vec3::new(1.0, 1.0, -1.0),   // PxPyNz
                    Vec3::new(1.0, 1.0, 1.0),    // PxPyPz
                ] {
                    let child_center = *center + octant * quarter_extent;
                    let child_bounds = Bounds::Cube {
                        center: child_center,
                        half_extent: quarter_extent,
                    };
                    let transform = Transform::from_translation(child_center);
                    children.push((child_bounds, transform));
                }

                children
            }
            _ => panic!("OctreeSubdivision only works with Bounds::Cube"),
        }
    }

    fn child_count(&self) -> usize {
        8
    }
}

/// Quadtree subdivision: splits along two axes (2D plane)
#[derive(Clone, Copy, Debug)]
pub struct QuadtreeSubdivision {
    pub split_axis: QuadtreeAxis,
}

#[derive(Clone, Copy, Debug)]
pub enum QuadtreeAxis {
    XZ, // Horizontal plane (typical for terrain)
    XY, // Front-facing plane
    YZ, // Side-facing plane
}

impl Subdivision for QuadtreeSubdivision {
    fn subdivide(&self, parent: &Bounds) -> Vec<(Bounds, Transform)> {
        match parent {
            Bounds::Cube {
                center,
                half_extent,
            } => {
                let quarter_extent = half_extent / 2.0;
                let mut children = Vec::with_capacity(4);

                let offsets = match self.split_axis {
                    QuadtreeAxis::XZ => [
                        Vec3::new(-quarter_extent, 0.0, -quarter_extent),
                        Vec3::new(-quarter_extent, 0.0, quarter_extent),
                        Vec3::new(quarter_extent, 0.0, -quarter_extent),
                        Vec3::new(quarter_extent, 0.0, quarter_extent),
                    ],
                    QuadtreeAxis::XY => [
                        Vec3::new(-quarter_extent, -quarter_extent, 0.0),
                        Vec3::new(-quarter_extent, quarter_extent, 0.0),
                        Vec3::new(quarter_extent, -quarter_extent, 0.0),
                        Vec3::new(quarter_extent, quarter_extent, 0.0),
                    ],
                    QuadtreeAxis::YZ => [
                        Vec3::new(0.0, -quarter_extent, -quarter_extent),
                        Vec3::new(0.0, -quarter_extent, quarter_extent),
                        Vec3::new(0.0, quarter_extent, -quarter_extent),
                        Vec3::new(0.0, quarter_extent, quarter_extent),
                    ],
                };

                for offset in offsets {
                    let child_center = *center + offset;
                    let child_bounds = Bounds::Cube {
                        center: child_center,
                        half_extent: quarter_extent,
                    };
                    let transform = Transform::from_translation(child_center);
                    children.push((child_bounds, transform));
                }

                children
            }
            _ => panic!("QuadtreeSubdivision only works with Bounds::Cube"),
        }
    }

    fn child_count(&self) -> usize {
        4
    }
}

/// Binary subdivision: splits along a single axis
#[derive(Clone, Copy, Debug)]
pub struct BinarySubdivision {
    pub axis: BinaryAxis,
}

#[derive(Clone, Copy, Debug)]
pub enum BinaryAxis {
    X,
    Y,
    Z,
}

impl Subdivision for BinarySubdivision {
    fn subdivide(&self, parent: &Bounds) -> Vec<(Bounds, Transform)> {
        match parent {
            Bounds::Cube {
                center,
                half_extent,
            } => {
                let quarter_extent = half_extent / 2.0;
                let offset = match self.axis {
                    BinaryAxis::X => Vec3::new(quarter_extent, 0.0, 0.0),
                    BinaryAxis::Y => Vec3::new(0.0, quarter_extent, 0.0),
                    BinaryAxis::Z => Vec3::new(0.0, 0.0, quarter_extent),
                };

                vec![
                    (
                        Bounds::Cube {
                            center: *center - offset,
                            half_extent: quarter_extent,
                        },
                        Transform::from_translation(*center - offset),
                    ),
                    (
                        Bounds::Cube {
                            center: *center + offset,
                            half_extent: quarter_extent,
                        },
                        Transform::from_translation(*center + offset),
                    ),
                ]
            }
            _ => panic!("BinarySubdivision only works with Bounds::Cube"),
        }
    }

    fn child_count(&self) -> usize {
        2
    }
}

/// Custom user-defined subdivision using a closure
pub struct CustomSubdivision {
    subdivider: Box<dyn Fn(&Bounds) -> Vec<(Bounds, Transform)> + Send + Sync>,
    count: usize,
}

impl CustomSubdivision {
    pub fn new<F>(count: usize, subdivider: F) -> Self
    where
        F: Fn(&Bounds) -> Vec<(Bounds, Transform)> + Send + Sync + 'static,
    {
        Self {
            subdivider: Box::new(subdivider),
            count,
        }
    }
}

impl Subdivision for CustomSubdivision {
    fn subdivide(&self, parent: &Bounds) -> Vec<(Bounds, Transform)> {
        (self.subdivider)(parent)
    }

    fn child_count(&self) -> usize {
        self.count
    }
}

/// Component that defines hierarchical splitting behavior for a ProceduralNode
#[derive(Component)]
pub struct HierarchicalNode {
    /// The bounds of this node's region
    pub bounds: Bounds,

    /// How this node subdivides into children (if at all)
    pub subdivision: Option<Box<dyn Subdivision>>,

    /// Current LOD level (0 = root, increases with depth)
    pub level: u32,

    /// Maximum depth for subdivision (prevents infinite splitting)
    pub max_depth: u32,
}

impl HierarchicalNode {
    /// Create a new hierarchical node with bounds
    pub fn new(bounds: Bounds) -> Self {
        Self {
            bounds,
            subdivision: None,
            level: 0,
            max_depth: 10, // Reasonable default
        }
    }

    /// Set the subdivision strategy
    pub fn with_subdivision(mut self, subdivision: impl Subdivision) -> Self {
        self.subdivision = Some(Box::new(subdivision));
        self
    }

    /// Set the maximum subdivision depth
    pub fn with_max_depth(mut self, max_depth: u32) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Check if this node can be subdivided
    pub fn can_subdivide(&self) -> bool {
        self.subdivision.is_some() && self.level < self.max_depth
    }

    /// Generate child bounds for subdivision
    pub fn generate_children(&self) -> Option<Vec<(Bounds, Transform)>> {
        if !self.can_subdivide() {
            return None;
        }

        self.subdivision.as_ref().map(|s| s.subdivide(&self.bounds))
    }
}

/// Marker component indicating this entity is a hierarchical root
#[derive(Component)]
pub struct HierarchicalRoot;

/// Builder for creating hierarchical procedural node hierarchies
pub struct HierarchyBuilder<T: ProceduralNode> {
    bounds: Bounds,
    subdivision: Option<Box<dyn Subdivision>>,
    max_depth: u32,
    node: T,
}

impl<T: ProceduralNode + Component> HierarchyBuilder<T> {
    pub fn new(bounds: Bounds, node: T) -> Self {
        Self {
            bounds,
            subdivision: None,
            max_depth: 10,
            node,
        }
    }

    pub fn with_subdivision(mut self, subdivision: impl Subdivision) -> Self {
        self.subdivision = Some(Box::new(subdivision));
        self
    }

    pub fn with_max_depth(mut self, max_depth: u32) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Spawn the root entity with all necessary components
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let mut hierarchical_node = HierarchicalNode::new(self.bounds);
        hierarchical_node.max_depth = self.max_depth;
        if let Some(subdivision) = self.subdivision {
            hierarchical_node.subdivision = Some(subdivision);
        }

        commands
            .spawn((
                self.node,
                hierarchical_node,
                HierarchicalRoot,
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id()
    }
}

/// System that splits hierarchical nodes when viewers get close
pub fn split_hierarchical_nodes(
    mut commands: Commands,
    config: Res<ProckitFrameworkConfig>,
    viewers: Query<&GlobalTransform, With<Viewer>>,
    nodes: Query<
        (
            Entity,
            &HierarchicalNode,
            &GlobalTransform,
            One<&dyn ProceduralNode>,
        ),
        Without<Children>,
    >,
    _provides_map: Res<ProvidesMap>,
) {
    for (entity, hierarchical_node, _transform, procedural_node) in nodes.iter() {
        if !hierarchical_node.can_subdivide() {
            continue;
        }

        // Calculate view area using minimum distance to any viewer
        let view_area = viewers
            .iter()
            .map(|viewer_transform| hierarchical_node.bounds.view_area(viewer_transform))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        // Split if we're beyond the upper visibility limit and node is not empty
        if view_area > config.upper_visibility_limit && !procedural_node.empty() {
            if let Some(children_specs) = hierarchical_node.generate_children() {
                split_node(&mut commands, entity, hierarchical_node, children_specs);
            }
        }
    }
}

/// System that merges hierarchical node children when viewers move away
pub fn merge_hierarchical_nodes(
    mut commands: Commands,
    config: Res<ProckitFrameworkConfig>,
    viewers: Query<&GlobalTransform, With<Viewer>>,
    nodes: Query<(
        Entity,
        &HierarchicalNode,
        &GlobalTransform,
        One<&dyn ProceduralNode>,
        &Children,
    )>,
) {
    for (entity, hierarchical_node, _transform, procedural_node, children) in nodes.iter() {
        // Calculate view area using minimum distance to any viewer
        let view_area = viewers
            .iter()
            .map(|viewer_transform| hierarchical_node.bounds.view_area(viewer_transform))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        // Merge if we're within the lower visibility limit or node is empty
        if view_area < config.lower_visibility_limit || procedural_node.empty() {
            merge_node(&mut commands, entity, children);
        }
    }
}

/// Helper function to split a node into children
fn split_node(
    commands: &mut Commands,
    parent_entity: Entity,
    parent_node: &HierarchicalNode,
    children_specs: Vec<(Bounds, Transform)>,
) {
    let mut child_entities = Vec::with_capacity(children_specs.len());

    for (child_bounds, child_transform) in children_specs {
        // Create child hierarchical node
        let mut child_hierarchical = HierarchicalNode::new(child_bounds);
        child_hierarchical.level = parent_node.level + 1;
        child_hierarchical.max_depth = parent_node.max_depth;

        // Clone the subdivision strategy if present
        // Note: This is a limitation - we can't easily clone Box<dyn Subdivision>
        // In practice, users should ensure their subdivision strategies are clonable
        // or manage subdivision separately
        child_hierarchical.subdivision = None; // TODO: Handle subdivision cloning

        let child_entity = commands
            .spawn((
                child_hierarchical,
                child_transform,
                GlobalTransform::default(),
            ))
            .id();

        child_entities.push(child_entity);
    }

    // Update parent with children
    commands.entity(parent_entity).add_children(&child_entities);
}

/// Helper function to merge node children back into parent
fn merge_node(commands: &mut Commands, _parent_entity: Entity, children: &Children) {
    // Despawn all children
    for child in children.iter() {
        commands.entity(child).despawn();
    }
}

// ============================================================================
// Volume Attachment System (Optional Integration with Volume Crate)
// ============================================================================
//
// These components and traits provide integration points for attaching
// generated chunks/volumes to hierarchical nodes. This is marked as a
// feature for future extension and requires the volume crate types.
//
// Uncomment and use these when integrating with the volume crate:
//
// use prockit_volume::{Chunk, Voxel, Rules};
//
// /// Component that marks a hierarchical node as having an attached volume/chunk
// #[derive(Component)]
// pub struct VolumeAttachment<V: Voxel, const N: usize> {
//     pub chunk: Chunk<V, N>,
// }
//
// impl<V: Voxel, const N: usize> VolumeAttachment<V, N> {
//     pub fn new(chunk: Chunk<V, N>) -> Self {
//         Self { chunk }
//     }
// }
//
// /// Helper trait to extend ProceduralNode with volume generation
// pub trait VolumeNode<V: Voxel, const N: usize>: ProceduralNode {
//     /// Generate a chunk for this node's bounds
//     fn generate_chunk(&mut self, bounds: &Bounds, rules: &impl Rules<V, N>) -> Chunk<V, N>;
// }
//
// Example system for automatically generating chunks for hierarchical leaf nodes:
//
// pub fn attach_volumes_to_leaves<V: Voxel + 'static, T: Component + VolumeNode<V, N>, const N: usize>(
//     mut commands: Commands,
//     new_leaves: Query<
//         (Entity, &HierarchicalNode, One<&mut dyn VolumeNode<V, N>>),
//         (Without<Children>, Without<VolumeAttachment<V, N>>)
//     >,
//     rules: Res<UserRules>, // User-provided Rules implementation
// ) {
//     for (entity, hierarchical_node, mut volume_node) in new_leaves.iter() {
//         let chunk = volume_node.generate_chunk(&hierarchical_node.bounds, &*rules);
//         commands.entity(entity).insert(VolumeAttachment::new(chunk));
//     }
// }
