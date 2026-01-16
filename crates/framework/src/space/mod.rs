//! Spatial abstractions and level-of-detail management for procedural generation.
//!
//! This module provides the core spatial infrastructure for the prockit framework:
//!
//! - [`Space`] trait: Defines coordinate systems and transform operations
//! - [`RealSpace`]: Concrete 3D Euclidean space implementation using Bevy transforms
//! - [`Subdivision`] and [`Subdivisions`]: Types for defining child nodes during generation
//! - [`Viewer`]: Component marking entities that influence LOD calculations
//! - [`PendingGenerate`]: Marker component to trigger node subdivision
//!
//! # Architecture
//!
//! The space module handles the geometric aspects of procedural generation:
//!
//! 1. **Coordinate System Abstraction**: The [`Space`] trait allows different spatial
//!    representations to plug into the framework.
//!
//! 2. **Transform Hierarchies**: Transforms propagate from parent to child nodes via
//!    [`Space::push_transform`], allowing hierarchical positioning.
//!
//! 3. **Level-of-Detail**: The [`Viewer`] component and threshold system automatically
//!    subdivide or collapse nodes based on viewer distance and memory limits.
//!
//! # Example
//!
//! ```
//! use prockit_framework::{Space, RealSpace, Viewer, Subdivision, MB, FrameworkPlugin};
//! use bevy::prelude::*;
//!
//! // Configure the framework for 3D space
//! let plugin = FrameworkPlugin::new()
//!     .with_space::<RealSpace>(64 * MB, 0.5);
//! ```

mod defs;
mod generate;
mod resample;

pub use defs::{RealSpace, Space};
use generate::EmptyNode;
pub(crate) use generate::GenerateTask;
pub use generate::{PendingGenerate, Subdivision, Subdivisions};
pub(crate) use resample::Thresholds;
pub use resample::Viewer;
