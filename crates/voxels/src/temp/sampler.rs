// ▄▀▀ ▄▀▄ █▄ ▄█ █▀▄ █   ██▀ █▀▄   █▀▄ ▄▀▀
// ▄██ █▀█ █ ▀ █ █▀  █▄▄ █▄▄ █▀▄ ▄ █▀▄ ▄██

//! The `sampler` module encapsulates the [`Sampler`] struct and all associated
//! methods. [`Sampler`]s are user created functions used to provide the data
//! that gets written to the [`VoxelStore`] as it iterates over a writing range.

/// The `Sampler` struct will grant structure for sampling from a spatial range,
/// intended to be used by users of this crate with their own functions passed
/// in to write their data into the voxel store. As of now, this functionality
/// is unimplemented.
pub struct Sampler;
