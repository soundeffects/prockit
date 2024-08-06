// █ █ ▄▀▄ ▀▄▀ ██▀ █     █▀▄ ▄▀▀
// ▀▄▀ ▀▄▀ █ █ █▄▄ █▄▄ ▄ █▀▄ ▄██

//! The `voxel` module contains the [`Voxel`] type and all related methods. The
//! [`Voxel`] type is just a wrapper for any 32-bit value, and provides many
//! functions for easy conversion between other 32-bit values and [`Voxel`]s.

/// A `Voxel` value is simply a 32-bit container for any data the user of this
/// crate wishes to store. The `Voxel` type implements conversions to/from `u32`
/// types, and in future should have conversions for several more types,
/// including `f32`. Note that `Voxel` values must occupy 32-bits. There are no
/// smaller or larger voxel allocations supported by this crate.
#[derive(Clone, Copy, Default)]
pub struct Voxel {
    value: u32,
}

impl Voxel {
    /// Creates a new `Voxel` with its data set to the bits of the `u32` passed
    /// to it.
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

impl From<u32> for Voxel {
    fn from(value: u32) -> Self {
        Self { value }
    }
}

impl Into<u32> for Voxel {
    fn into(self) -> u32 {
        self.value
    }
}
