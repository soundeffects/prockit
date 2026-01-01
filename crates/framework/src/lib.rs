use bevy::{platform::collections::HashMap, prelude::*};
use bevy_trait_query::{One, RegisterExt};
use std::{any::TypeId, marker::PhantomData};

mod provides;
pub use provides::{NameQuery, Names, Provides};

mod hierarchy;
// pub use hierarchy::{
//     BinaryAxis, BinarySubdivision, Bounds, CustomSubdivision, HierarchicalNode, HierarchicalRoot,
//     HierarchyBuilder, OctreeSubdivision, QuadtreeAxis, QuadtreeSubdivision, Subdivision,
// };

#[derive(Default, Resource)]
pub struct ProvidesMap {
    map: HashMap<TypeId, Provides>,
}

impl ProvidesMap {
    pub fn add<T: 'static>(&mut self, provides: Provides) {
        self.map.insert(TypeId::of::<T>(), provides);
    }
}

// pub enum Sampling {
//     D1(f32),
//     D2(Vec2),
//     D3(Vec3),
//     D4(Vec4),
// }

pub struct ChildConstructor;

#[bevy_trait_query::queryable]
pub trait ProceduralNode {
    fn space(&self, transform: GlobalTransform) -> f64;

    fn in_bounds(&self, transform: GlobalTransform, position: Vec4) -> bool;

    fn subdivide(&self, provider: &Provides, child_constructor: ChildConstructor);

    fn placement(&self);

    fn generate(&mut self, provider: &Provides);

    fn init() -> Self
    where
        Self: Sized;

    fn provides() -> Provides
    where
        Self: Sized;

    fn empty(&self) -> bool;
}

#[derive(Component)]
pub struct Viewer;

pub struct ProceduralNodePlugin<ProceduralNodeType> {
    procedural_node_type: PhantomData<ProceduralNodeType>,
}

impl<ProceduralNodeType> Plugin for ProceduralNodePlugin<ProceduralNodeType>
where
    ProceduralNodeType: Component + ProceduralNode + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<ProvidesMap>()
            .register_component_as::<dyn ProceduralNode, ProceduralNodeType>()
            .add_systems(Startup, register_provides::<ProceduralNodeType>);
    }
}

fn register_provides<T: ProceduralNode + 'static>(mut provides_map: ResMut<ProvidesMap>) {
    provides_map.add::<T>(T::provides())
}

#[derive(Resource)]
pub struct ProckitFrameworkConfig {
    upper_visibility_limit: f32,
    lower_visibility_limit: f32,
}

impl Default for ProckitFrameworkConfig {
    fn default() -> Self {
        Self {
            upper_visibility_limit: 10.0,
            lower_visibility_limit: 1.0,
        }
    }
}

pub struct ProckitFrameworkPlugin;

impl Plugin for ProckitFrameworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProckitFrameworkConfig>()
            .add_systems(Update, resample);
    }
}

fn resample(
    mut _commands: Commands,
    config: Res<ProckitFrameworkConfig>,
    viewers: Query<&GlobalTransform, With<Viewer>>,
    leaf_nodes: Query<
        (One<&dyn ProceduralNode>, &GlobalTransform, Option<&ChildOf>),
        Without<Children>,
    >,
    branch_nodes: Query<(One<&dyn ProceduralNode>, &GlobalTransform), With<Children>>,
) {
    let mut leaves_once_removed = Vec::new();
    for (procedural_node, transform, optional_parent) in leaf_nodes {
        if let Some(min_distance) = viewers
            .iter()
            .map(|viewer_transform| {
                transform
                    .translation()
                    .distance_squared(viewer_transform.translation())
            })
            .min_by(|a, b| a.partial_cmp(b).unwrap())
        {
            if min_distance > config.upper_visibility_limit && !procedural_node.empty() {
                // upsample
            }
        }
        if let Some(parent) = optional_parent {
            leaves_once_removed.push(parent.0);
        }
    }

    for (procedural_node, transform) in leaves_once_removed
        .iter()
        .filter_map(|entity| branch_nodes.get(*entity).ok())
    {
        if let Some(min_distance) = viewers
            .iter()
            .map(|viewer_transform| {
                transform
                    .translation()
                    .distance_squared(viewer_transform.translation())
            })
            .min_by(|a, b| a.partial_cmp(b).unwrap())
        {
            if min_distance < config.lower_visibility_limit || procedural_node.empty() {
                // downsample
            }
        }
    }
}
