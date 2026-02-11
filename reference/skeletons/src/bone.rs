// ██▄ ▄▀▄ █▄ █ ██▀   █▀▄ ▄▀▀
// █▄█ ▀▄▀ █ ▀█ █▄▄ ▄ █▀▄ ▄██
//! This module describes the `Bone` component. It implements both accessor functions for fields of
//! the `Bone` struct and utilities for positioning a `Bone` relative to its parent.
use bevy::prelude::*;

/// The `Bone` component marks an entity as a member of a skeleton. It specifies both length and
/// angle that, when applied to an extrusion from the parent, will create a line segment between the
/// parent position and the new position.
///
/// The length of the bone is measured in world units.
///
/// Both components of the angle vector are measured in radians. The x component of the angle vector
/// describes a rotation about the parent's direction as the axis, and the y component describes a
/// 'latitudinal' rotation that bends back towards the negative of the parent's direction.
#[derive(Clone, Copy, Component)]
pub struct Bone {
    length: f32,
    angle: Vec2,
}

/// The `ParentContext` type represents the direction and the tangent of the parent bone, which is
/// used by the child bone to position itself relative to the parent.
pub type ParentContext = (Vec3, Vec3);

impl Bone {
    /// Creates a new bone component with the specified length and angle.
    ///
    /// The length of the bone is measured in world units.
    ///
    /// Both components of the angle vector are measured in radians. The x component of the angle
    /// vector describes a rotation about the parent's direction as the axis, and the y component
    /// describes a 'latitudinal' rotation that bends back towards the negative of the parent's
    /// direction.
    pub fn new(length: f32, angle: Vec2) -> Self {
        Self { length, angle }
    }

    /// Returns the length of the bone, in world units.
    pub fn length(&self) -> f32 {
        self.length
    }

    /// Returns the angle vector of the bone. Both components of the vector are measured in radians.
    /// The x component describes a rotation about the parent's direction as the axis of rotation,
    /// and the y component describes a 'latitudinal' rotation that rotates back towards the
    /// negative of the parent's direction.
    pub fn angle(&self) -> Vec2 {
        self.angle
    }

    /// Takes a `ParentContext` as input, and returns a new `ParentContext` transformed by this
    /// bone's angle vector.
    pub fn derive(&self, parent_context: ParentContext) -> ParentContext {
        // Get parent context values and ensure that they are normalized
        let (direction, tangent) = parent_context;
        let (direction, tangent) = (direction.normalize(), tangent.normalize());

        // Rotate about the parent's direction as the axis for the x component
        let axial_rotation = Quat::from_axis_angle(direction, self.angle.x);

        // Rotate the tangent with axial_rotation
        let rotated_tangent = axial_rotation * tangent;

        // Rotate about the cotangent (cross of direction and tangent) as the axis for the y
        // component
        let latitudinal_rotation =
            Quat::from_axis_angle(direction.cross(rotated_tangent), self.angle.y);

        (
            latitudinal_rotation * direction,
            latitudinal_rotation * rotated_tangent,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Bone;
    use bevy::prelude::*;
    use std::f32::consts::PI;

    // Test that the derive function rotates counter-clockwise.
    #[test]
    fn test_derive_counter_clockwise() {
        // Bone defines a rotation of 90 degrees backwards towards the negative of the parents
        // direction
        let bone = Bone::new(1.0, Vec2::new(0.0, PI / 2.0));
        let (direction, tangent) = bone.derive((Vec3::X, Vec3::Y));

        // Rotating 90 degrees backwards should make the new direction equal to the old tangent,
        // and the new tangent should be equal to the negative of the old direction.
        assert!(direction.distance(Vec3::Y) < 0.001);
        assert!(tangent.distance(Vec3::NEG_X) < 0.001);
    }
}
