use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::{util::{BufferInitDescriptor, DeviceExt}, vertex_attr_array, Buffer, BufferAddress, BufferUsages, Device, VertexAttribute, VertexBufferLayout, VertexStepMode};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ColoredVertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl ColoredVertex {
    const ATTRIBUTES: [VertexAttribute; 2] = vertex_attr_array![0 => Float32x3, 1 => Float32x3];
    pub fn buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<ColoredVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub const TRIANGLE_VERTICES: &[ColoredVertex] = &[
    ColoredVertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
    ColoredVertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
    ColoredVertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
];

pub const PENTAGON_VERTICES: &[ColoredVertex] = &[
    ColoredVertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.5, 0.0, 0.5] },
    ColoredVertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 0.0, 0.5] },
    ColoredVertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5] },
    ColoredVertex { position: [0.35966998, -0.3473291, 0.0], color: [0.5, 0.0, 0.5] },
    ColoredVertex { position: [0.44147372, 0.2347359, 0.0], color: [0.5, 0.0, 0.5] },
];

pub const PENTAGON_INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4
];

pub struct Model {
    pub vertices: Vec<ColoredVertex>,
    pub indices: Vec<u16>,
    pub vertex_count: u32,
    pub index_count: u32,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
}

impl Model {
    pub fn pentagon(gpu_device: &Device) -> Self {
        Self {
            vertices: PENTAGON_VERTICES.to_vec(),
            indices: PENTAGON_INDICES.to_vec(),
            vertex_count: PENTAGON_VERTICES.len() as u32,
            index_count: PENTAGON_INDICES.len() as u32,
            vertex_buffer: gpu_device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: cast_slice(PENTAGON_VERTICES),
                usage: BufferUsages::VERTEX,
            }),
            index_buffer: gpu_device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: cast_slice(PENTAGON_INDICES),
                usage: BufferUsages::INDEX,
            }),
        }
    }
}