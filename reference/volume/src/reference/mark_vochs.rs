use bevy::prelude::*;
use crate::{VochMap, VochPosition, VoxelViewer, GenerateVoch, VoxelPluginConfig, Voch};
use std::collections::VecDeque;

/// Finds all `VochPosition`s around `VoxelViewer`s that should
/// be generated
pub(crate) fn mark_vochs(
    mut commands: Commands,
    voch_map: Res<VochMap>,
    voxel_viewers: Query<(&VoxelViewer, &GlobalTransform)>,
    config: Res<VoxelPluginConfig>
) {
    let mut queue = VecDeque::new();
    for (viewer, transform) in voxel_viewers.iter() {
        let viewer_position= transform.translation();
        let voch_position = VochPosition::from_world(viewer_position, config.voch_length);
        let viewer_radius = viewer.radius();
        queue.push_back((voch_position, viewer_position, viewer_radius));
    }
    while let Some((voch_position, viewer_position, viewer_radius)) = queue.pop_front() {
        let world_position = voch_position.to_world(config.voch_length);
        let exceed_memory= config.voxel_memory_limit < voch_map.logical_memory() + size_of::<Voch>();
        let exceed_radius = world_position.distance(viewer_position) > viewer_radius;
        if exceed_memory || exceed_radius {
            break;
        }
        if voch_map.get(&voch_position).is_none() {
            commands.spawn((
                Transform::from_translation(world_position),
                voch_position,
                GenerateVoch,
            ));
            println!("Spawning voch at {:?}", world_position);
        }
        for adjacent in voch_position.adjacents() {
            queue.push_back((adjacent, viewer_position, viewer_radius));
        }
    }
}