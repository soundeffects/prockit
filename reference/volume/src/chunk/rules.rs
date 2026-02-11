use super::{Octant, Position, Voxel, VoxelTag, simple_normal::SimpleNormal};

pub(crate) trait Rules<V: Voxel, const N: usize> {
    fn generate(&self, position: Position<N>) -> V;

    fn enhance(&self, tag: VoxelTag<V, N>, subvoxel: Octant) -> V;

    fn surface(&self, first: VoxelTag<V, N>, second: VoxelTag<V, N>) -> SimpleNormal;
}
