use glam::{vec3, Vec2, Vec3, Vec4};

use crate::rand_vec3;

use super::{Mesh, MeshHandle, Renderer, Vertex};

pub struct Trail {
    pub handle: MeshHandle,
    positions: Vec<Vec3>,
    indices: Vec<u32>,
    vertices: Vec<Vertex>,
    resolution: u32,
    pub thickness: f32,

    pub position: Vec3,
}

impl Trail {
    pub fn new(renderer: &mut Renderer, resolution: u32) -> Self {
        let num_vertices = (resolution - 1) * 2;
        let vertices = vec![Vertex::new(vec3(0.0, 0.0, 0.0), Vec4::ONE, Vec2::ONE, Vec3::ONE); num_vertices as usize ];

        let indices = (0..(resolution - 2))
            .flat_map(|i| {
                let base = i * 2;
                vec![
                    base, base + 1, base + 2, // 1st triangle
                    base + 1, base + 3, base + 2  // 2nd triangle
                ]
            })
            .collect::<Vec<u32>>();

        let mesh = renderer.add_mesh(
            Mesh::new(&vertices, &indices)
        ).unwrap();

        Self {
            handle: mesh,
            positions: vec![],
            indices,
            vertices,
            resolution,
            thickness: 1.0,
            position: Vec3::ZERO
        }
    }

    pub fn update(&mut self, renderer: &mut Renderer) {
        let mesh = &mut renderer.meshes[self.handle];

        self.positions.push(self.position);
        
        if self.positions.len() > self.resolution as usize {
            self.positions.remove(0);
        }

        for i in 0..self.positions.len() {
            self.vertices[i].position = self.positions[i] + (rand_vec3() * 2.0 - 1.0) * self.thickness;
        }

        for i in self.positions.len()..self.vertices.len() {
            self.vertices[i].position = *self.positions.last().unwrap() + (rand_vec3() * 2.0 - 1.0) * self.thickness;
        }

        mesh.update_data(self.vertices.clone(), self.indices.clone())

    }
}