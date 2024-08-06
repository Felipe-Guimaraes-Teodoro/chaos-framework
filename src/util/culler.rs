use glam::{vec3, vec4, Mat4, Vec3, Vec3A, Vec4};

use crate::{Camera, MeshHandle, Renderer};

/*
A set of utilities to help with culling meshes that are not in the view frustrum of the camera
*/

pub struct Culler {
    pub mesh_handles: Vec<MeshHandle>,
}

impl Culler {
    pub fn new(renderer: &Renderer) -> Self {
        Self {
            mesh_handles: vec![],
        }
    }

    pub fn add_mesh(&mut self, handle: MeshHandle) {
        self.mesh_handles.push(handle);
    }

    pub fn update(&mut self, renderer: &mut Renderer) {
        let frustum = Frustum::sample_from_camera(
            &renderer.camera, 
            1.0, 
            90.0_f32.to_radians(), 
            0.1, 
            100.0
        );

        for handle in &self.mesh_handles {
            let mesh = renderer.meshes.get_mut(handle).unwrap();

            let biggest_radius = |mesh: &crate::Mesh| {
                let mut size = 0.0;
                let mut biggest = 0.0;

                for v in &mesh.vertices {
                    size = v.position.length();
                    if size > biggest {
                        biggest = size;
                    }
                }

                size
            };

            let volume = Sphere {
                center: mesh.position,
                // radius: biggest_radius(&mesh),
                radius: 1.0
            };

            let model_matrix = 
                Mat4::from_translation(mesh.position) *
                Mat4::from_quat(mesh.rotation) *
                Mat4::from_scale(mesh.scale);

            if volume.is_on_frustrum(&frustum, model_matrix) {
                mesh.hidden = false;
            } else {
                mesh.hidden = true;
            }
            
        }
    }
}

struct Plane {
    normal: Vec3,
    point: Vec3,
}

impl Plane {
    fn get_signed_distance_to_plane(&self, point: Vec3) -> f32 {
        return Vec3::dot(self.normal, point) - self.point.length();
    }
}

pub struct Frustum {
    top_face: Plane,
    bottom_face: Plane,

    right_face: Plane,
    left_face: Plane,

    far_face: Plane,
    near_face: Plane,
}

impl Frustum {
    pub fn sample_from_camera(cam: &Camera, aspect: f32, fov_y: f32, z_near: f32, z_far: f32) -> Self {
        let half_v_side = z_far * (fov_y * 0.5).tan();
        let half_h_side = half_v_side * aspect;
        let front_mult_far = z_far * cam.front;

        Self {
            near_face: Plane {
                point: cam.pos + z_near * cam.front,
                normal: cam.front,
            },
            far_face: Plane {
                point: cam.pos + front_mult_far,
                normal: -cam.front,
            },
            right_face: Plane {
                point: cam.pos,
                normal: (front_mult_far - cam.right * half_h_side).cross(cam.up).normalize(),
            },
            left_face: Plane {
                point: cam.pos,
                normal: cam.up.cross(front_mult_far + cam.right * half_h_side).normalize(),
            },
            top_face: Plane {
                point: cam.pos,
                normal: cam.right.cross(front_mult_far - cam.up * half_v_side).normalize(),
            },
            bottom_face: Plane {
                point: cam.pos,
                normal: (front_mult_far + cam.up * half_v_side).cross(cam.right).normalize(),
            },
        }

    }

}

pub trait Volume {
    fn is_on_frustrum(&self, frustum: &Frustum, model: Mat4) -> bool;
}

struct Sphere {
    center: Vec3,
    radius: f32,
}

impl Sphere {
    pub fn is_on_or_forward_plane(&self, plane: &Plane) -> bool {
        let distance = plane.normal.dot(self.center) - plane.point.length();
        distance <= self.radius
    }
}

impl Volume for Sphere {
    fn is_on_frustrum(&self, frustum: &Frustum, model: Mat4) -> bool {
        let scale_x = model.col(0).truncate().length();
        let scale_y = model.col(1).truncate().length();
        let scale_z = model.col(2).truncate().length();
        let global_scale = Vec3A::new(scale_x, scale_y, scale_z);

        let global_center = (model * Vec4::new(self.center.x, self.center.y, self.center.z, 1.0)).truncate();

        let max_scale = global_scale.max_element();

        let global_sphere = Sphere {
            center: global_center,
            radius: self.radius * (max_scale * 0.5),
        };

        global_sphere.is_on_or_forward_plane(&frustum.left_face)
            && global_sphere.is_on_or_forward_plane(&frustum.right_face)
            && global_sphere.is_on_or_forward_plane(&frustum.far_face)
            && global_sphere.is_on_or_forward_plane(&frustum.near_face)
            && global_sphere.is_on_or_forward_plane(&frustum.top_face)
            && global_sphere.is_on_or_forward_plane(&frustum.bottom_face)
    }
}

struct AABB {
    center: Vec3,
    extents: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        let center = (max + min) * 0.5;

        Self {
            center,
            extents: vec3(max.x - center.x, max.y - center.y, max.z - center.z),
        }
    }

    fn is_on_or_forward_plane(&self, plane: &Plane) -> bool {
        let r = self.extents.x * plane.normal.x.abs()
            + self.extents.y * plane.normal.y.abs() 
            + self.extents.z * plane.normal.z.abs();

        return -r <= plane.get_signed_distance_to_plane(self.center);
    }
}

impl Volume for AABB {
    fn is_on_frustrum(&self, frustum: &Frustum, model: Mat4) -> bool {
        let global_center = (model * vec4(self.center.x, self.center.y, self.center.z, 1.0)).truncate();

        let right = model.col(0).truncate() * self.extents.x;
        let up = model.col(1).truncate() * self.extents.y;
        let forward = model.col(2).truncate() * self.extents.z;

        let new_ii = right.abs().dot(vec3(1.0, 0.0, 0.0))
            + up.abs().dot(vec3(1.0, 0.0, 0.0))
            + forward.abs().dot(vec3(1.0, 0.0, 0.0));

        let new_ij = right.abs().dot(vec3(0.0, 1.0, 0.0))
            + up.abs().dot(vec3(0.0, 1.0, 0.0))
            + forward.abs().dot(vec3(0.0, 1.0, 0.0));

        let new_ik = right.abs().dot(vec3(0.0, 0.0, 1.0))
            + up.abs().dot(vec3(0.0, 0.0, 1.0))
            + forward.abs().dot(vec3(0.0, 0.0, 1.0));

        let global_aabb = AABB {
            center: global_center,
            extents: vec3(new_ii, new_ij, new_ik),
        };

        global_aabb.is_on_or_forward_plane(&frustum.left_face)
            && global_aabb.is_on_or_forward_plane(&frustum.right_face)
            && global_aabb.is_on_or_forward_plane(&frustum.top_face)
            && global_aabb.is_on_or_forward_plane(&frustum.bottom_face)
            && global_aabb.is_on_or_forward_plane(&frustum.near_face)
            && global_aabb.is_on_or_forward_plane(&frustum.far_face)
    }
}