use bevy::prelude::IVec3;
use std::ops::Neg;

#[derive(Clone, Copy)]
pub(super) enum CubicDirection {
    Nx,
    Px,
    Ny,
    Py,
    Nz,
    Pz,
}

impl CubicDirection {
    pub(super) const ALL: [CubicDirection; 6] = [
        CubicDirection::Nx,
        CubicDirection::Px,
        CubicDirection::Ny,
        CubicDirection::Py,
        CubicDirection::Nz,
        CubicDirection::Pz,
    ];

    pub(super) const POSITIVE: [CubicDirection; 3] =
        [CubicDirection::Px, CubicDirection::Py, CubicDirection::Pz];

    pub(super) fn as_ivec3(&self) -> IVec3 {
        match self {
            CubicDirection::Nx => IVec3::NEG_X,
            CubicDirection::Px => IVec3::X,
            CubicDirection::Ny => IVec3::NEG_Y,
            CubicDirection::Py => IVec3::Y,
            CubicDirection::Nz => IVec3::NEG_Z,
            CubicDirection::Pz => IVec3::Z,
        }
    }
}

impl Neg for CubicDirection {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            CubicDirection::Nx => CubicDirection::Px,
            CubicDirection::Px => CubicDirection::Nx,
            CubicDirection::Ny => CubicDirection::Py,
            CubicDirection::Py => CubicDirection::Ny,
            CubicDirection::Nz => CubicDirection::Pz,
            CubicDirection::Pz => CubicDirection::Nz,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_tests() {
        todo!()
    }
}
