// ██▄ ██▀ █ █ ▀▄▀   █▀▄ ▄▀▀
// █▄█ █▄▄ ▀▄▀  █  ▄ █▀▄ ▄██

//! The `bevy` module is only available when the `bevy` feature flag is enabled,
//! and should encapsulate all integrations between this crate and the Bevy game
//! engine. The primary integrations are `Component` implementations and the
//! [`VoxelStoreDiagnosticsPlugin`], which provides diagnostics (such as number
//! of populated chunks) for all [`VoxelStore`]s in the ECS.

use crate::voxel_store::VoxelStore;
use bevy::{
    diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, RegisterDiagnostic},
    ecs::component::StorageType,
    prelude::*,
};

// Not using the Component derive macro because VoxelStore is external to this
// module
impl Component for VoxelStore {
    // Simply defining the storage type to be the default `Table`
    const STORAGE_TYPE: StorageType = StorageType::Table;
}

/// The `VOXEL_STORE_LEN` diagnostic path will store the total number of
/// populated voxel chunks of all voxel stores in the ECS
pub const VOXEL_STORE_LEN: DiagnosticPath = DiagnosticPath::const_new("voxel_store_len");

/// The `VoxelStoreDiagnosticsPlugin` will add the `VOXEL_STORE_LEN` diagnostic,
/// which measures the total number of populated voxel chunks of all voxel
/// stores in the ECS, and updates that diagnostic with its own system every
/// frame.
pub struct VoxelStoreDiagnosticsPlugin;
impl Plugin for VoxelStoreDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.register_diagnostic(Diagnostic::new(VOXEL_STORE_LEN).with_suffix(" chunks"))
            .add_systems(Update, update_diagnostics);
    }
}

// This is the only system registered by the `VoxelStoreDiagnosticsPlugin`,
// and simply measures the total number of populated chunks of all voxel stores
// in the ECS.
fn update_diagnostics(mut diagnostics: Diagnostics, voxel_stores: Query<&VoxelStore>) {
    diagnostics.add_measurement(&VOXEL_STORE_LEN, || {
        let mut total_len = 0;
        for voxel_store in &voxel_stores {
            total_len += voxel_store.len();
        }
        total_len as f64
    });
}
