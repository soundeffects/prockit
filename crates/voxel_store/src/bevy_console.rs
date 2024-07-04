use bevy::prelude::*;
use bevy_console::{reply, AddConsoleCommand, ConsoleCommand, ConsolePlugin};
use clap::Parser;

use crate::voxel_store::VoxelStore;

pub struct VoxelStoreCommandPlugin;

impl Plugin for VoxelStoreCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ConsolePlugin)
            .add_console_command::<StoreInfoCommand, _>(store_info_command);
    }
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "voxelstore")]
struct StoreInfoCommand;

fn store_info_command(
    mut command: ConsoleCommand<StoreInfoCommand>,
    voxel_stores: Query<(Option<&Name>, &VoxelStore)>,
) {
    if command.take().is_some_and(|result| result.is_ok()) {
        for (optional_name, voxel_store) in &voxel_stores {
            if let Some(name) = optional_name {
                reply!(
                    command,
                    "The voxel store named {} has {} chunks.",
                    name,
                    voxel_store.len()
                );
            } else {
                reply!(
                    command,
                    "An unnamed voxel store has {} chunks.",
                    voxel_store.len()
                );
            }
        }

        command.ok();
    }
}
