mod chunk;
mod cubic_direction;
mod octant;
mod volume;
mod voxel;

use bevy::prelude::{App, Plugin};
pub use volume::Volume;

pub struct VolumePlugin;

impl Plugin for VolumePlugin {
    fn build(&self, app: &mut App) {}
}
