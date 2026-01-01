use super::{CubicDirection, Face};
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

pub(super) fn compile_mesh(faces: Vec<Face>) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    let mut current_index = 0;
    for face in faces {
        let offsets = [
            Vec3::new(-0.5, -0.5, 0.),
            Vec3::new(-0.5, 0.5, 0.),
            Vec3::new(0.5, -0.5, 0.),
            Vec3::new(0.5, 0.5, 0.),
        ];

        let offsets = match face.normal() {
            CubicDirection::Px => offsets.map(|offset| offset.zxy()),
            CubicDirection::Nx => offsets.map(|offset| offset.zxy()),
            CubicDirection::Py => offsets.map(|offset| offset.xzy()),
            CubicDirection::Ny => offsets.map(|offset| offset.xzy()),
            _ => offsets,
        };

        for offset in offsets {
            let direction = face.normal().as_ivec3().as_vec3();
            positions.push(face.origin() + direction * 0.5 + offset);
            normals.push(direction);
        }

        for offset in [0, 1, 2, 2, 1, 3] {
            indices.push(current_index + offset);
        }
        current_index += 6;
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U16(indices))
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_tests() {
        todo!()
    }
}
