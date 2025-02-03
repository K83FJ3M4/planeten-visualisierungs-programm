use std::collections::HashMap;
use super::Vertex;
use bytemuck::cast_slice;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{Buffer, BufferUsages, Device};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Edge(usize, usize);

#[derive(Clone, Debug)]
pub(super) struct Icosphere {
    vertices: Vec<[f32; 3]>,
    indices: Vec<[usize; 3]>,
}

impl Icosphere {
    pub(super) fn new(subdivisions: u32) -> Self {
        let mut vertices = Self::icosahedron_vertices();
        let mut indices = Self::icosahedron_faces();

        for _ in 0..subdivisions {
            let mut midpoints = HashMap::new();
            let mut new_indices = Vec::new();

            for &[v0, v1, v2] in &indices {
                let a = Self::get_midpoint(v0, v1, &mut vertices, &mut midpoints);
                let b = Self::get_midpoint(v1, v2, &mut vertices, &mut midpoints);
                let c = Self::get_midpoint(v2, v0, &mut vertices, &mut midpoints);
                
                new_indices.push([v0, a, c]);
                new_indices.push([v1, b, a]);
                new_indices.push([v2, c, b]);
                new_indices.push([a, b, c]);
            }

            indices = new_indices;
        }

        Self { vertices, indices }
    }

    fn random(seed: usize) -> bool {
        let mut state = seed;
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        state & 1 == 1
    }

    pub(super) fn vertex_buffer(&self, device: &Device) -> Buffer {
        let color = [0.3, 0.8, 0.5, 1.0];

        let vertices = self.vertices.iter().enumerate()
            .map(|(i, &[x, y, z])| Vertex { position: [x, y, z, 1.0], color: if Self::random(i) { color } else { [1.0; 4] } })
            .collect::<Vec<_>>();

        device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::VERTEX,
            contents: cast_slice(&vertices)
        })
    }

    pub(super) fn index_count(&self) -> u32 {
        self.indices.len() as u32 * 3
    }

    pub(super) fn index_buffer(&self, device: &Device) -> Buffer {
        let triangle_indices = self.indices.iter()
            .flat_map(|triangle| triangle.iter().copied())
            .map(|index| index as u32)
            .collect::<Vec<_>>();

        device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            usage: BufferUsages::INDEX,
            contents: cast_slice(&triangle_indices)
        })
    }

    fn icosahedron_vertices() -> Vec<[f32; 3]> {
        let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;
        let mut vertices = vec![
            [-1.0, phi, 0.0], [1.0, phi, 0.0], [-1.0, -phi, 0.0], [1.0, -phi, 0.0],
            [0.0, -1.0, phi], [0.0, 1.0, phi], [0.0, -1.0, -phi], [0.0, 1.0, -phi],
            [phi, 0.0, -1.0], [phi, 0.0, 1.0], [-phi, 0.0, -1.0], [-phi, 0.0, 1.0],
        ];

        for v in &mut vertices {
            let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            v[0] /= len;
            v[1] /= len;
            v[2] /= len;
        }
        vertices
    }

    fn icosahedron_faces() -> Vec<[usize; 3]> {
        vec![
            [0, 11, 5], [0, 5, 1], [0, 1, 7], [0, 7, 10], [0, 10, 11],
            [1, 5, 9], [5, 11, 4], [11, 10, 2], [10, 7, 6], [7, 1, 8],
            [3, 9, 4], [3, 4, 2], [3, 2, 6], [3, 6, 8], [3, 8, 9],
            [4, 9, 5], [2, 4, 11], [6, 2, 10], [8, 6, 7], [9, 8, 1],
        ]
    }

    fn get_midpoint(v0: usize, v1: usize, vertices: &mut Vec<[f32; 3]>, midpoints: &mut HashMap<Edge, usize>) -> usize {
        let edge = if v0 < v1 { Edge(v0, v1) } else { Edge(v1, v0) };
        if let Some(&index) = midpoints.get(&edge) {
            return index;
        }
        let mid = [
            (vertices[v0][0] + vertices[v1][0]) * 0.5,
            (vertices[v0][1] + vertices[v1][1]) * 0.5,
            (vertices[v0][2] + vertices[v1][2]) * 0.5,
        ];
        let len = (mid[0] * mid[0] + mid[1] * mid[1] + mid[2] * mid[2]).sqrt();
        let mid = [mid[0] / len, mid[1] / len, mid[2] / len];
        
        let index = vertices.len();
        vertices.push(mid);
        midpoints.insert(edge, index);
        index
    }
}
