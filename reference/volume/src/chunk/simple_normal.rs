use super::{CubicDirection, Face};
use bevy::prelude::Vec3;

/// A one-dimensional and unquantified normal of a surface. Such a normal may be positive,
/// negative, or zero only.
pub(super) enum SimpleNormal {
    Positive,
    Negative,
    None,
}

impl SimpleNormal {
    #[inline]
    pub(super) fn as_face(&self, origin: Vec3, normal: CubicDirection) -> Option<Face> {
        match self {
            SimpleNormal::Positive => Some(Face::new(origin, normal)),
            SimpleNormal::Negative => Some(Face::new(origin, normal)),
            SimpleNormal::None => None,
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
