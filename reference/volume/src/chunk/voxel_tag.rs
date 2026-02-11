use super::{Position, Voxel};

pub(super) struct VoxelTag<V: Voxel, const N: usize> {
    data: V,
    position: Position<N>,
}

impl<V: Voxel, const N: usize> VoxelTag<V, N> {
    pub(super) fn new(data: V, position: Position<N>) -> Self {
        Self { data, position }
    }

    pub(super) fn data(&self) -> V {
        self.data
    }

    pub(super) fn position(&self) -> Position<N> {
        self.position
    }
}
