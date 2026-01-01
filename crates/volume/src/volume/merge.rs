use super::{Viewer, Volume};
use bevy::prelude::*;

pub(super) fn merge_volumes(
    mut commands: Commands,
    viewers: Query<&Viewer>,
    volumes: Query<&Volume>,
) {
    todo!();
}
