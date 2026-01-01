mod config;
mod merge;
mod split;
mod viewer;

use super::chunk::Chunk;
use bevy::prelude::*;
pub use config::Config;
use merge::merge_volumes;
use split::split_volumes;
use viewer::Viewer;

#[derive(Component)]
#[require(GlobalTransform)]
pub struct Volume {
    config: Config,
}

impl Volume {
    pub(super) fn new(config: Config) -> Self {
        Self { config }
    }
}

pub(super) fn setup(mut app: App) {
    app.add_systems(Update, (split_volumes, merge_volumes));
}
