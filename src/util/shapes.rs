use glam::{vec2, vec3, Vec3, Vec4};

use crate::graphics::{Mesh, Vertex};

pub struct Quad{
    pub size: Vec3,
    pub color: Vec4,
}

impl Quad{
    pub fn new(size: Vec3, color: Vec4) -> Self{
        Self{
            size,
            color,
        }
    }

    pub fn mesh(&self) -> Mesh {
        let vertices = vec![
            Vertex::new(vec3(0.0, 0.0, 0.0), self.color, vec2(0.0, 0.0), vec3(0., 0., 1.)),                    // Bottom-left
            Vertex::new(vec3(0.0, self.size.y, 0.0), self.color, vec2(0.0, 1.0), vec3(0., 0., 1.)),             // Top-left
            Vertex::new(vec3(self.size.x, 0.0, 0.0), self.color, vec2(1.0, 0.0), vec3(0., 0., 1.)),             // Bottom-right
            Vertex::new(vec3(self.size.x, self.size.y, 0.0), self.color, vec2(1.0, 1.0), vec3(0., 0., 1.)),      // Top-right
        ];


        let indices = vec![0, 2, 1, 2, 3, 1];
        
        Mesh::new(&vertices, &indices)
    }
}

pub struct Circle{
    pub iterations: i32,
    pub radius: f32,
    pub color: Vec4,
}

impl Circle {
    pub fn new(iterations: i32, radius: f32, color: Vec4) -> Self{
        let mut fixed_iterations = iterations;
        if iterations <= 3{
            fixed_iterations = 4;
        }

        Self {
            iterations: fixed_iterations,
            radius,
            color,
        }
    }

    pub fn mesh(&self) -> Mesh {
        let mut vertices = vec![];
        
        for i in 0..self.iterations {
            let angle = 2.0 * std::f32::consts::PI * (i as f32 / self.iterations as f32);
            let tex_coord = vec2(f32::cos(angle) * 0.5 + 0.5, f32::sin(angle) * 0.5 + 0.5); // Normalize to [0, 1] range
        
            vertices.push(Vertex::new(
                vec3(f32::sin(angle), f32::cos(angle), 0.0) * self.radius,
                self.color,
                tex_coord,
                vec3(0., 0., 1.),
            ));
        }
        
        let mut indices = vec![];
        for i in 1..=self.iterations-2 {
            indices.push(0); 
            indices.push(i as u32); 
            indices.push((i % self.iterations + 1) as u32);
        }

        Mesh::new(&vertices, &indices)
    }
}
 
pub struct Triangle{
    pub size: f32,
    pub color: Vec4,
}

impl Triangle{
    pub fn new(size: f32, color: Vec4) -> Self{
        Self {
            size,
            color,
        }
    }

    pub fn mesh(&self) -> Mesh {
        let mut vertices = vec![];
        for i in 0..3 {
            let angle = 2.0 * std::f32::consts::PI * (i as f32 / 3.0);
            let tex_coord = match i {
                0 => vec2(0.5, 1.0),   // Bottom vertex
                1 => vec2(0.0, 0.0),   // Left vertex
                2 => vec2(1.0, 0.0),   // Right vertex
                _ => panic!("Unexpected index"),
            };
        
            vertices.push(Vertex::new(
                vec3(f32::sin(angle), f32::cos(angle), 0.0) * self.size,
                self.color,
                tex_coord,
                vec3(0., 0., 1.),
            ));
        }

        let mut indices = vec![];
            indices.push(0); 
            indices.push(1 as u32); 
            indices.push(2 as u32);
            // Shamelessly (ok theres a bit of shame) stole my own circle rendering code so I just set it to three vertices

        Mesh::new(&vertices, &indices)
    }
}

pub struct Cuboid{
    pub size: Vec3,
    pub color: Vec4,
}

impl Cuboid{
    pub fn new(size: Vec3, color: Vec4) -> Self{
        Self{
            size,
            color,
        }
    }

    pub fn mesh(&self) -> Mesh {
        let half_size = self.size/2.;
        let x = half_size.x;
        let y = half_size.y;
        let z = half_size.z;

        let vertices = vec![
            // Front face
            Vertex::new(vec3(-x, -y, z), self.color, vec2(0.0, 0.0), vec3(0.0, 0.0, 1.0)),    // 0
            Vertex::new(vec3(x, -y, z), self.color, vec2(1.0, 0.0), vec3(0.0, 0.0, 1.0)),     // 1
            Vertex::new(vec3(x, y, z), self.color, vec2(1.0, 1.0), vec3(0.0, 0.0, 1.0)),      // 2
            Vertex::new(vec3(-x, y, z), self.color, vec2(0.0, 1.0), vec3(0.0, 0.0, 1.0)),     // 3

            // Back face
            Vertex::new(vec3(-x, -y, -z), self.color, vec2(0.0, 0.0), vec3(0.0, 0.0, -1.0)),   // 4
            Vertex::new(vec3(x, -y, -z), self.color, vec2(1.0, 0.0), vec3(0.0, 0.0, -1.0)),    // 5
            Vertex::new(vec3(x, y, -z), self.color, vec2(1.0, 1.0), vec3(0.0, 0.0, -1.0)),     // 6
            Vertex::new(vec3(-x, y, -z), self.color, vec2(0.0, 1.0), vec3(0.0, 0.0, -1.0)),    // 7

            // Left face
            Vertex::new(vec3(-x, -y, -z), self.color, vec2(0.0, 0.0), vec3(-1.0, 0.0, 0.0)),   // 8
            Vertex::new(vec3(-x, -y, z), self.color, vec2(1.0, 0.0), vec3(-1.0, 0.0, 0.0)),    // 9
            Vertex::new(vec3(-x, y, z), self.color, vec2(1.0, 1.0), vec3(-1.0, 0.0, 0.0)),     // 10
            Vertex::new(vec3(-x, y, -z), self.color, vec2(0.0, 1.0), vec3(-1.0, 0.0, 0.0)),    // 11

            // Right face
            Vertex::new(vec3(x, -y, -z), self.color, vec2(0.0, 0.0), vec3(1.0, 0.0, 0.0)),    // 12
            Vertex::new(vec3(x, -y, z), self.color, vec2(1.0, 0.0), vec3(1.0, 0.0, 0.0)),     // 13
            Vertex::new(vec3(x, y, z), self.color, vec2(1.0, 1.0), vec3(1.0, 0.0, 0.0)),      // 14
            Vertex::new(vec3(x, y, -z), self.color, vec2(0.0, 1.0), vec3(1.0, 0.0, 0.0)),     // 15

            // Top face
            Vertex::new(vec3(-x, y, -z), self.color, vec2(0.0, 0.0), vec3(0.0, 1.0, 0.0)),    // 16
            Vertex::new(vec3(x, y, -z), self.color, vec2(1.0, 0.0), vec3(0.0, 1.0, 0.0)),     // 17
            Vertex::new(vec3(x, y, z), self.color, vec2(1.0, 1.0), vec3(0.0, 1.0, 0.0)),      // 18
            Vertex::new(vec3(-x, y, z), self.color, vec2(0.0, 1.0), vec3(0.0, 1.0, 0.0)),     // 19

            // Bottom face
            Vertex::new(vec3(-x, -y, -z), self.color, vec2(0.0, 0.0), vec3(0.0, -1.0, 0.0)),   // 20
            Vertex::new(vec3(x, -y, -z), self.color, vec2(1.0, 0.0), vec3(0.0, -1.0, 0.0)),    // 21
            Vertex::new(vec3(x, -y, z), self.color, vec2(1.0, 1.0), vec3(0.0, -1.0, 0.0)),     // 22
            Vertex::new(vec3(-x, -y, z), self.color, vec2(0.0, 1.0), vec3(0.0, -1.0, 0.0)),    // 23
        ];

        let indices = vec![
            // Front face
            0, 3, 2, 2, 1, 0,
            // Back face
            4, 5, 6, 6, 7, 4,
            // Left face
            8, 11, 10, 10, 9, 8,
            // Right face
            12, 13, 14, 14, 15, 12,
            // Top face
            16, 17, 18, 18, 19, 16,
            // Bottom face
            20, 23, 22, 22, 21, 20
        ];
        
        Mesh::new(&vertices, &indices)
    }
}

pub struct Sphere{
    pub iterations: i32,
    pub radius: f32,
    pub color: Vec4,
}

impl Sphere {
    pub fn new(iterations: i32, radius: f32, color: Vec4) -> Self{
        let mut fixed_iterations = iterations;
        if iterations <= 3{
            fixed_iterations = 4;
        }

        Self {
            iterations: fixed_iterations,
            radius,
            color,
        }
    }

    pub fn mesh(&self) -> Mesh {
        let mut vertices = vec![];
        let pi = std::f32::consts::PI;

        for lat in 0..=self.iterations {
            let theta = pi * lat as f32 / self.iterations as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            for lon in 0..=self.iterations {
                let phi = 2.0 * pi * lon as f32 / self.iterations as f32;
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();

                let x = cos_phi * sin_theta * self.radius;
                let y = cos_theta * self.radius;
                let z = sin_phi * sin_theta * self.radius;

                // Calculate texture coordinates (cylindrical projection)
                let s = lon as f32 / self.iterations as f32;
                let t = 1.0 - (lat as f32 / self.iterations as f32);

                let normal = vec3(x, y, z).normalize();

                vertices.push(Vertex::new(vec3(x, y, z), self.color, vec2(s, t), normal));
            }
        }

        let mut indices = vec![];
        for lat in 0..self.iterations {
            for lon in 0..self.iterations {
                let first = lat * (self.iterations + 1) + lon;
                let second = first + self.iterations + 1;

                indices.push(first as u32);
                indices.push(second as u32);
                indices.push((first + 1) as u32);

                indices.push(second as u32);
                indices.push((second + 1) as u32);
                indices.push((first + 1) as u32);
            }
        }

        Mesh::new(&vertices, &indices)
    }
}