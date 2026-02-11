// █▀▄ █▀▄ ▄▀▄ ▄▀▀ █▄▀ █ ▀█▀    ▄▀▀ █▄▀ ██▀ █   ██▀ ▀█▀ ▄▀▄ █▄ █ ▄▀▀
// █▀  █▀▄ ▀▄▀ ▀▄▄ █ █ █  █ ▄▄▄ ▄██ █ █ █▄▄ █▄▄ █▄▄  █  ▀▄▀ █ ▀█ ▄██
//! The `prockit_skeletons` crate defines the `Skeleton` and `Bone` components, which can have
//! dynamic tree-shaped topologies and from which meshes can be constructed at runtime. This creates
//! a foundation for procedurally generating trees, animals, and characters. It also allows for easy
//! physics simulation or animation of those objects by manipulating the underlying bone structure.
#![deny(missing_docs, rustdoc::all)]

use bevy::prelude::*;

mod bone;
mod generators;
mod gizmos;
mod skeleton;

pub use bone::Bone;
pub use generators::{degrees_to_radians, stick_figure};
pub use gizmos::SkeletonGizmosPlugin;
pub use skeleton::{Skeleton, SkeletonDescriptor};

/// The `SkeletonPlugin` is the main plugin for the `prockit_skeletons` crate. It adds the
/// required systems for skeleton construction.
pub struct SkeletonPlugin;

impl Plugin for SkeletonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, skeleton::construct_skeletons);
    }
}
