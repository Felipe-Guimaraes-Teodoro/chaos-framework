use glam::{Vec2, Vec3, Vec4};



#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub color: Vec4,
    pub tex_coords: Vec2,
    pub normal: Vec3,
}

impl Vertex {
    pub fn new(position: Vec3, color: Vec4, tex_coords: Vec2, normal: Vec3) -> Self {
        Self {
            position,
            color,
            tex_coords,
            normal,
        }
    }
}
