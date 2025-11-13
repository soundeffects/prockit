// █ █ ▄▀▄ ▀▄▀ ██▀ █     █▀▄ ▄▀▀ 
// ▀▄▀ ▀▄▀ █ █ █▄▄ █▄▄ ▄ █▀▄ ▄██ 

//! Defines the `Voxel` trait and a simple implementation of it.

use crate::volumes::Sign;
use num_traits::Unsigned;

/// All voxels must have a unique identifier, regardless of what
/// other properties the voxel may have. After the generation
/// step, voxels are stored by their id rather than their unique
/// properties.
pub type VoxelId = u8;
// TODO: use arbitrary byte string instead of u8

/// The `Voxel` trait allows you to define your own voxel data
/// for use with [`Volume`]s. It must implement a method to
/// determine whether it is opaque. At the boundary between an
/// opaque and transparent voxel there will be a surface.
pub trait Voxel: Sized + Default + Clone + From<VoxelId> + Into<VoxelId> {
    fn opaque(&self) -> bool;
    fn surface_normal(&self, other: &Self) -> Option<Sign> {
        match (self.opaque(), other.opaque()) {
            (true, false) => Some(Sign::Positive),
            (false, true) => Some(Sign::Negative),
            _ => None,
        }
    }
}

pub struct VoxelDeterminator {
    opaque: dyn Fn(&VoxelId) -> bool,
}

pub struct VoxelRegistry<V: Voxel> {
    types: Vec<V>
}

/// A simple implementation of the `Voxel` trait, which only
/// defines two types of voxels: one that is transparent and one
/// that is opaque.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum SimpleVoxel {
    #[default]
    Transparent,
    Opaque,
}

impl Voxel for SimpleVoxel {
    fn opaque(&self) -> bool {
        match self {
            Self::Transparent => false,
            Self::Opaque => true,
        }
    }
}

impl From<VoxelId> for SimpleVoxel {
    fn from(id: VoxelId) -> Self {
        match id {
            0 => Self::Transparent,
            1 => Self::Opaque,
            _ => panic!("Invalid voxel id: {}", id),
        }
    }
}

impl Into<VoxelId> for SimpleVoxel {
    fn into(self) -> VoxelId {
        match self {
            Self::Transparent => 0,
            Self::Opaque => 1,
        }
    }
}