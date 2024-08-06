// ▄▀▀ █▄▀ ██▀ █   ██▀ ▀█▀ ▄▀▄ █▄ █   █▀▄ ▄▀▀
// ▄██ █ █ █▄▄ █▄▄ █▄▄  █  ▀▄▀ █ ▀█ ▄ █▀▄ ▄██
//! This module contains components and systems which construct hierarchies of `Bone` entities,
//! which we call `Skeleton`s. We use this extra scaffolding because constructing parent/child
//! hierarchies within a Bevy system often requires substantial boilerplate. Creating a
//! `SkeletonDescriptor` abstraction allows us to define skeleton structures more tersely, and a
//! Bevy systeom can then automatically construct the true hierarchy in the ECS by consuming that
//! descriptor.
use crate::Bone;
use bevy::prelude::*;

/// A component marking an entity as the root for a parent/child hierachy of bones, considered in
/// total as a "skeleton".
#[derive(Component)]
pub struct Skeleton;

/// The `SkeletonDescElement` is a private struct used to describe a single bone, and link to the
/// bones children, as a member of a parent `SkeletonDescriptor` object.
#[derive(Clone)]
pub struct SkeletonDescElement {
    bone: Bone,
    children: Vec<Self>,
}

/// The `SkeletonDescriptor` is used as a simply initialized object describing a skeleton/bone
/// hierarchy. It will be consumed by a Bevy system and converted into multiple ECS entities in a
/// parent/child hierarchy representing the total skeleton.
#[derive(Component)]
pub struct SkeletonDescriptor {
    children: Vec<SkeletonDescElement>,
}

impl SkeletonDescriptor {
    /// Creates a new `SkeletonDescriptor` with the given children, which should be provided by the
    /// `SkeletonDescriptor::branch` and `SkeletonDescriptor::leaf` functions.
    ///
    /// ## Syntax Example
    /// ```
    /// let root = SkeletonDescriptor::root;
    /// let branch = SkeletonDescriptor::branch;
    /// let leaf = SkeletonDescriptor::leaf;
    ///
    /// let descriptor = root(&[
    ///     branch(
    ///         0.5, // Length
    ///         [0.0, 0.0], // Angles
    ///         &[leaf(0.5, [0.0, 0.0]), leaf(0.5, [0.0, 0.0])]
    ///     ),
    ///     leaf(0.5, [0.0, 0.0]),
    ///     leaf(0.5, [0.0, 0.0])]
    /// );
    /// ```
    pub fn root(children: &[SkeletonDescElement]) -> Self {
        Self {
            children: children.to_vec(),
        }
    }

    /// Creates a new element for a `SkeletonDescriptor` with the given length, angle difference
    /// from parent, and children, which should be provided by the `SkeletonDescriptor::branch` and
    /// `SkeletonDescriptor::leaf` functions. This method should be called as an argument to another
    /// `SkeletonDescriptor::branch` or `SkeletonDescriptor::root` function.
    ///
    /// ## Syntax Example
    /// ```
    /// let root = SkeletonDescriptor::root;
    /// let branch = SkeletonDescriptor::branch;
    /// let leaf = SkeletonDescriptor::leaf;
    ///
    /// let descriptor = root(&[
    ///     branch(
    ///         0.5, // Length
    ///         [0.0, 0.0], // Angles
    ///         &[leaf(0.5, [0.0, 0.0]), leaf(0.5, [0.0, 0.0])]
    ///     ),
    ///     leaf(0.5, [0.0, 0.0]),
    ///     leaf(0.5, [0.0, 0.0])]
    /// ```
    pub fn branch(
        length: f32,
        angle: [f32; 2],
        children: &[SkeletonDescElement],
    ) -> SkeletonDescElement {
        SkeletonDescElement {
            bone: Bone::new(length, Vec2::from_array(angle)),
            children: children.to_vec(),
        }
    }

    /// Creates a new element for a `SkeletonDescriptor` with the given length and angle difference
    /// from parent. This method should be called as an argument to another
    /// `SkeletonDescriptor::branch` or `SkeletonDescriptor::root` function.
    ///
    /// ## Syntax Example
    /// ```
    /// let root = SkeletonDescriptor::root;
    /// let branch = SkeletonDescriptor::branch;
    /// let leaf = SkeletonDescriptor::leaf;
    ///
    /// let descriptor = root(&[
    ///     branch(
    ///         0.5, // Length
    ///         [0.0, 0.0], // Angles
    ///         &[leaf(0.5, [0.0, 0.0]), leaf(0.5, [0.0, 0.0])]
    ///     ),
    ///     leaf(0.5, [0.0, 0.0]),
    ///     leaf(0.5, [0.0, 0.0])]
    /// );
    /// ```
    pub fn leaf(length: f32, angle: [f32; 2]) -> SkeletonDescElement {
        SkeletonDescElement {
            bone: Bone::new(length, Vec2::from_array(angle)),
            children: Vec::new(),
        }
    }
}

/// This system consumes all entities containing a `SkeletonDescriptor` component and spawns a
/// collection of entities into the ECS which match the parent/child hierarchy outlined in the
/// `SkeletonDescriptor` component.
pub(crate) fn construct_skeletons(
    mut commands: Commands,
    skeleton_descriptors: Query<(Entity, &Transform, &SkeletonDescriptor)>,
) {
    for (entity, transform, skeleton_descriptor) in &skeleton_descriptors {
        // Create the root Skeleton component
        let id = commands.spawn((transform.clone(), Skeleton)).id();

        // Initialize the stack to be the children of the root component
        let mut stack = Vec::new();
        for child in &skeleton_descriptor.children {
            stack.push((id, child));
        }

        // Iterate through the bone hierarchy using the stack
        while !stack.is_empty() {
            // We can unwrap because the stack is guaranteed not to be empty
            let (parent_id, element) = stack.pop().unwrap();

            // Spawn this bone as a child of its parent
            let id = commands.spawn(element.bone).id();
            commands.get_entity(parent_id).unwrap().add_child(id);

            // Add the children of this bone to the stack
            for child in &element.children {
                stack.push((id, child));
            }
        }

        // We finished adding the skeleton, so we remove the skeleton descriptor
        commands.entity(entity).despawn();
    }
}

/*
// Iterator
// --------
#[derive(Default, PartialEq, Eq)]
pub enum SimplificationMode {
    #[default]
    NONE,
    RECEDE,
}

#[derive(Default)]
pub struct SimplificationDescriptor {
    pub mode: SimplificationMode,
    pub threshold: f64,
    pub ease_threshold: f64,
}

#[derive(Default)]
pub struct SkeletonIteratorDescriptor {
    pub simplification: SimplificationDescriptor,
    pub camera_position: Vec3,
    pub lod_threshold: f32,
}

pub struct SkeletonIterator<'a> {
    descriptor: SkeletonIteratorDescriptor,
    current_index: usize,
    skeleton: &'a Skeleton,
}

impl<'a> Iterator for SkeletonIterator<'a> {
    type Item = ExplicitBone;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.current_index += 1;

            if self.current_index >= self.skeleton.bones.len() {
                return None;
            }

            let child_bone = self.skeleton.bones[self.current_index];
            let parent_bone = self.skeleton.bones[child_bone.parent_index];
            let total_bone = ExplicitBone {
                start: parent_bone.position,
                end: child_bone.position,
            };

            if self.descriptor.simplification.mode == SimplificationMode::NONE {
                return Some(total_bone);
            }

            let start_distance = total_bone.start().distance(self.descriptor.camera_position);
            let end_distance = total_bone.end().distance(self.descriptor.camera_position);
            let camera_distance = start_distance.min(end_distance);

            if total_bone.start().distance(total_bone.end())
                > (camera_distance / self.descriptor.lod_threshold)
            {
                return Some(total_bone);
            }
        }
    }
}
*/
