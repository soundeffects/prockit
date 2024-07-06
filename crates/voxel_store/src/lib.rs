// █   █ ██▄   █▀▄ ▄▀▀
// █▄▄ █ █▄█ ▄ █▀▄ ▄██

//! Simple voxel data storage. There are many solutions for volumetric data,
//! but this crate optimizes for three main goals.
//!
//! First, it aims to be reasonably compact, such that large spaces can fit in
//! RAM and GPU VRAM of consumer hardware. This is achieved with a sparse
//! hierarchy, eliminating detail where none is required. Further, it has been
//! designed such that a limited set can be loaded, and the rest stored on disk
//! or in CPU RAM, leaving the GPU VRAM footprint small. In future, the crate
//! should also use compression algorithms on regions with high detail for even
//! further reduction.
//!
//! Second, it intends to accelerate raymarching applications. Sparse hierarchy
//! is useful here, but bitmasking has also been used for further elimination of
//! detail during ray tracing. See the docs on the [`Chunk`] struct for more
//! information on the bitmasking.
//!
//! Finally, it aims to keep the developer experience for both using and
//! maintaining the crate as simple as possible. The crate makes decisions which
//! compromise to best achieve the stated goals above, and very few generics are
//! available to the user to customize the voxel store. The user cannot set
//! voxel size or chunk size, for example. As long as the user agrees with the
//! goals above, there should be few decisions left for the user to worry about.
//!
//! Notable non-goals are write efficiency and random access efficiency.
//! Optimizing for these criteria is often in direct opposition towards
//! raymarching optimizations. Furthermore, write and random access are rare
//! operations relative to raymarching. Write operations can also be queued such
//! that the voxel world is eventually consistent with operations applied to it,
//! without any loss in framerate of the application.
//!
//! A basic usage example is provided below.
//!
//! ```
//! // Create an empty voxel store
//! let mut voxel_store = VoxelStore::new();
//!
//! // The voxel store is empty, so we write to it. The new values are as
//! // defined by the sampler function, and writes happen within the range of
//! // -10 to 10 in all three spatial axes.
//! voxel_store.write(-10..=10, -10..=10, -10..=10, Sampler);
//! ```
//!
//! The `bevy` feature flag can be enabled for this crate for basic integrations
//! with the Bevy game engine. This includes basic info, events, and component
//! declarations for `VoxelStore` structs.

#![deny(missing_docs)]
#![deny(rustdoc::all)]

mod chunk;
mod sampler;
mod voxel;
mod voxel_store;

#[cfg(feature = "bevy")]
mod bevy;

/// The prelude module provides a single import for all user-facing types and
/// methods in the `voxel_store` crate. All users of the crate should import
/// `voxel_store::prelude::*`.
pub mod prelude {
    pub use crate::sampler::Sampler;
    pub use crate::voxel::Voxel;
    pub use crate::voxel_store::VoxelStore;

    #[cfg(feature = "bevy")]
    pub use crate::bevy::*;
}
