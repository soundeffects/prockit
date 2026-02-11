use super::{Position, Voxel};

#[derive(Clone)]
pub(super) struct Storage<V: Voxel, const N: usize>([[[V; N]; N]; N]);

impl<V: Voxel, const N: usize> Default for Storage<V, N> {
    fn default() -> Self {
        Self([[[V::default(); N]; N]; N])
    }
}

impl<V: Voxel, const N: usize> Storage<V, N> {
    pub(super) fn get(&self, position: Position<N>) -> V {
        self.0[position.z()][position.y()][position.x()]
    }

    pub(super) fn set(&mut self, position: Position<N>, value: V) {
        self.0[position.z()][position.y()][position.x()] = value;
    }
}
