// ▄▀  ██▀ █▄ █ ██▀ █▀▄ ▄▀▄ ▀█▀ ▄▀▄ █▀▄ ▄▀▀   █▀▄ ▄▀▀
// ▀▄█ █▄▄ █ ▀█ █▄▄ █▀▄ █▀█  █  ▀▄▀ █▀▄ ▄██ ▄ █▀▄ ▄██
//! This module contains a set of functions used to generate skeleton descriptors. At the time of
//! writing, these are mostly used for testing purposes, but in future they will be used to generate
//! useful game content.
use crate::SkeletonDescriptor;
use std::f32::consts::PI;

/// Converts an integer amount of angular degrees to radians.
pub fn degrees_to_radians(degrees: i32) -> f32 {
    degrees as f32 * PI / 180.0
}

/// Generates a skeleton descriptor that looks like a stick figure.
pub fn stick_figure() -> SkeletonDescriptor {
    // For brevity, we will use variables to make namespace identifiers unnecessary for the skeleton
    // descriptor functions.
    let root = SkeletonDescriptor::root;
    let branch = SkeletonDescriptor::branch;
    let leaf = SkeletonDescriptor::leaf;

    root(&[
        // The lower body and legs
        branch(
            0.5,
            [0.0, degrees_to_radians(-90)],
            &[
                leaf(0.75, [degrees_to_radians(90), degrees_to_radians(30)]),
                leaf(0.75, [degrees_to_radians(-90), degrees_to_radians(30)]),
            ],
        ),
        // The arms
        leaf(0.75, [degrees_to_radians(90), degrees_to_radians(90)]),
        leaf(0.75, [degrees_to_radians(-90), degrees_to_radians(90)]),
        // The neck and head
        branch(
            0.1,
            [0.0, degrees_to_radians(90)],
            &[
                leaf(0.1, [degrees_to_radians(90), degrees_to_radians(60)]),
                leaf(0.1, [degrees_to_radians(-90), degrees_to_radians(60)]),
            ],
        ),
    ])
}
