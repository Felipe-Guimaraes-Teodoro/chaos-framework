use std::{collections::HashMap, rc::Rc, ffi::CString};
use glam::{vec3, Mat4, Quat, Vec3};
use russimp::{node::Node, property::PropertyStore, scene::{PostProcess, Scene}};

use crate::{convert_russimp_mat_to_glam_mat, cstr, BoneInfo, Model, Shader};

#[derive(Copy, Clone)]
struct KeyPosition {
    position: Vec3,
    timestamp: f32,
}

#[derive(Copy, Clone)]
struct KeyRotation {
    rotation: Quat,
    timestamp: f32,
}

#[derive(Copy, Clone)]
struct KeyScale {
    scale: Vec3,
    timestamp: f32,
}

#[derive(Clone)]
struct aBone {
    positions: Vec<KeyPosition>,
    rotations: Vec<KeyRotation>,
    scalings: Vec<KeyScale>,
    num_positions: usize,
    num_rotations: usize,
    num_scalings: usize,

    local_transform: Mat4,
    name: String,
    id: i32,
}

impl aBone {
    pub fn new(name: &str, id: i32, channel: &russimp::animation::NodeAnim) -> Self {
        let mut positions = vec![];
        let num_positions = channel.position_keys.len();
        let mut rotations = vec![];
        let num_rotations = channel.rotation_keys.len();
        let mut scalings = vec![];
        let num_scalings = channel.scaling_keys.len();

        for pos_idx in 0..num_positions {
            let channel_pos = channel.position_keys[pos_idx].value;
            let pos = vec3(channel_pos.x, channel_pos.y, channel_pos.z);
            let timestamp = channel.position_keys[pos_idx].time as f32;

            positions.push(KeyPosition { position: pos, timestamp });
        }

        for rot_idx in 0..num_rotations {
            let channel_rot = channel.rotation_keys[rot_idx].value;
            let rot = Quat::from_xyzw(channel_rot.x, channel_rot.y, channel_rot.z, channel_rot.w);
            let timestamp = channel.rotation_keys[rot_idx].time as f32;

            rotations.push(KeyRotation { rotation: rot, timestamp });
        }

        for sca_idx in 0..num_scalings {
            let channel_sca = channel.scaling_keys[sca_idx].value;
            let sca = vec3(channel_sca.x, channel_sca.y, channel_sca.z);
            let timestamp = channel.scaling_keys[sca_idx].time as f32;

            scalings.push(KeyScale { scale: sca, timestamp });
        }

        aBone {
            positions,
            rotations,
            scalings,
            num_positions,
            num_rotations,
            num_scalings,
            local_transform: Mat4::IDENTITY,
            name: name.to_string(),
            id,
        }
    }

    pub fn update(&mut self, animation_time: f32) {
        let translation = self.interpolate_position(animation_time);
        let rotation = self.interpolate_rotation(animation_time);
        let scale = self.interpolate_scale(animation_time);

        self.local_transform = translation * rotation * scale;
    }

    fn get_scale_factor(last_timestamp: f32, next_timestamp: f32, animation_time: f32) -> f32 {
        let midway_length = animation_time - last_timestamp;
        let frames_difference = next_timestamp - last_timestamp;
        midway_length / frames_difference
    }

    fn get_position_index(&self, animation_time: f32) -> usize {
        for i in 0..self.num_positions - 1 {
            if animation_time < self.positions[i + 1].timestamp {
                return i;
            }
        }
        0 // things got fucked up
    }

    fn get_rotation_index(&self, animation_time: f32) -> usize {
        for i in 0..self.num_rotations - 1 {
            if animation_time < self.rotations[i + 1].timestamp {
                return i;
            }
        }
        0 // things got fucked up (rotation edition)
    }

    fn get_scale_index(&self, animation_time: f32) -> usize {
        for i in 0..self.num_scalings - 1 {
            if animation_time < self.scalings[i + 1].timestamp {
                return i;
            }
        }
        0 // yeah
    }

    fn interpolate_position(&self, animation_time: f32) -> Mat4 {
        if self.num_positions == 1 {
            return Mat4::from_translation(self.positions[0].position);
        }

        let p0idx = self.get_position_index(animation_time);
        let p1idx = p0idx + 1;

        let scale_factor = Self::get_scale_factor(self.positions[p0idx].timestamp, self.positions[p1idx].timestamp, animation_time);

        let final_position = self.positions[p0idx].position.lerp(self.positions[p1idx].position, scale_factor);

        Mat4::from_translation(final_position)
    }

    fn interpolate_rotation(&self, animation_time: f32) -> Mat4 {
        if self.num_rotations == 1 {
            return Mat4::from_rotation_translation(self.rotations[0].rotation, Vec3::ZERO);
        }

        let p0idx = self.get_rotation_index(animation_time);
        let p1idx = p0idx + 1;

        let scale_factor = Self::get_scale_factor(self.rotations[p0idx].timestamp, self.rotations[p1idx].timestamp, animation_time);

        let final_rotation = self.rotations[p0idx].rotation.slerp(self.rotations[p1idx].rotation, scale_factor);

        Mat4::from_rotation_translation(final_rotation, Vec3::ZERO)
    }

    fn interpolate_scale(&self, animation_time: f32) -> Mat4 {
        if self.num_scalings == 1 {
            return Mat4::from_scale(self.scalings[0].scale);
        }

        let p0idx = self.get_scale_index(animation_time);
        let p1idx = p0idx + 1;

        let scale_factor = Self::get_scale_factor(self.scalings[p0idx].timestamp, self.scalings[p1idx].timestamp, animation_time);

        let final_scale = self.scalings[p0idx].scale.lerp(self.scalings[p1idx].scale, scale_factor);

        Mat4::from_scale(final_scale)
    }
}

#[derive(Default, Clone)]
struct RussimpNodeData {
    transformation: Mat4,
    name: String,
    children_count: i32,
    children: Vec<Self>,
}

pub struct Animation {
    duration: f32,
    ticks_per_second: i32,
    bones: Vec<aBone>,
    root_node: RussimpNodeData,
    bone_info_map: HashMap<String, BoneInfo>,
}

impl Animation {
    pub fn new(path: &str, model: &mut crate::Model) -> Self {
        let scene = load_scene(path);

        let russimp_animation = &scene.animations[0];

        let mut root_node = RussimpNodeData::default();
        Self::read_hierarchy_data(&mut root_node, scene.root.expect("Scene has no root node"));

        let mut animation = Self {
            duration: russimp_animation.duration as f32,
            ticks_per_second: russimp_animation.ticks_per_second as i32,
            bones: vec![],
            root_node,
            bone_info_map: HashMap::new(),
        };

        animation.read_missing_bones(russimp_animation, model);

        animation
    }

    fn find_bone(&mut self, name: &str) -> Option<&mut aBone> {
        self.bones.iter_mut().find(|bone| bone.name == name)
    }

    fn read_missing_bones(&mut self, animation: &russimp::animation::Animation, model: &mut Model) {
        let size = animation.channels.len();

        let bone_count = &mut model.bone_info_map.len();
        let bone_info_map = &mut model.bone_info_map;

        for i in 0..size {
            let channel = &animation.channels[i];
            let bone_name = channel.name.clone();

            if !bone_info_map.contains_key(&bone_name) {
                bone_info_map.insert(bone_name.clone(), BoneInfo {
                    id: *bone_count as i32,
                    ofs: Mat4::IDENTITY, // Initialize offset, replace with actual if needed
                });
                *bone_count += 1;
            }

            self.bones.push(aBone::new(&bone_name, bone_info_map[&bone_name].id, channel));
        }

        self.bone_info_map = bone_info_map.clone();
    }

    fn read_hierarchy_data(dest: &mut RussimpNodeData, src: Rc<Node>) {
        dest.name = src.name.clone();
        dest.transformation = convert_russimp_mat_to_glam_mat(src.transformation);
        dest.children_count = src.children.borrow().len() as i32;

        for child in src.children.borrow().iter() {
            let mut new_data = RussimpNodeData::default();
            Self::read_hierarchy_data(&mut new_data, child.clone());
            dest.children.push(new_data);
        }
    }
}

pub fn load_scene(path: &str) -> Scene {
    let props = PropertyStore::default();

    Scene::from_file_with_props(
        path,
        vec![
            // PostProcess::OptimizeMeshes,
            PostProcess::Triangulate,
            // PostProcess::GenerateSmoothNormals,
            // PostProcess::FlipWindingOrder,
            // PostProcess::JoinIdenticalVertices,
            // PostProcess::OptimizeGraph,
        ],
        &props,
    )
    .unwrap()
}

pub struct Animator {
    final_bone_matrices: Vec<Mat4>,
    current_animation: Animation,
    current_time: f32,
    delta_time: f32,
}

impl Animator {
    pub fn new(animation: Animation) -> Self {
        let final_bone_matrices = vec![Mat4::IDENTITY; 100];

        Self {
            current_time: 0.0,
            current_animation: animation,
            final_bone_matrices,
            delta_time: 0.0,
        }
    }

    pub fn update_animation(&mut self, dt: f32) {
        self.delta_time = dt;

        self.current_time += self.current_animation.ticks_per_second as f32 * dt;
        self.current_time = self.current_time - (self.current_time / self.current_animation.duration).floor() * self.current_animation.duration;

        let root = self.current_animation.root_node.clone();
        self.calculate_bone_transform(&root, Mat4::IDENTITY);
    }

    /* 
    fn calculate_bone_transform(&mut self, node: &RussimpNodeData, parent_transform: Mat4) {
        let node_name = &node.name;
        let mut node_transform = node.transformation;

        if let Some(bone) = self.current_animation.find_bone(node_name) {
            bone.update(self.current_time);
            node_transform = bone.local_transform;
        }

        let global_transformation = parent_transform * node_transform;

        if let Some(bone_info) = self.current_animation.bone_info_map.get(node_name) {
            let index = bone_info.id;
            let offset = bone_info.ofs;
            self.final_bone_matrices[index as usize] = global_transformation * offset;
        }

        for child in &node.children {
            self.calculate_bone_transform(child, global_transformation);
        }
    }
    */ 
    
    fn calculate_bone_transform(&mut self, node: &RussimpNodeData, parent_transform: glam::Mat4) {
        let mut node_transform = node.transformation;
    
        if let Some(bone) = self.current_animation.find_bone(&node.name) {
            bone.update(self.current_time);
            node_transform = bone.local_transform;
        }
    
        let global_transformation = parent_transform * node_transform;
    
        if let Some(bone_info) = self.current_animation.find_bone(&node.name) {
            let index = bone_info.id as usize;
            if index < 100 {
                self.final_bone_matrices[index] = global_transformation * node_transform;
            }
        }
    
        for child in &node.children {
            self.calculate_bone_transform(child, global_transformation);
        }
    }

    pub fn upload_uniforms(&self, shader: &Shader) {
        for (i, matrix) in self.final_bone_matrices.iter().enumerate() {
            unsafe {
                shader.uniform_mat4fv(
                    cstr!(format!("finalBonesMatrices[{}]", i)),
                    &matrix.to_cols_array(),
                );
            }
        }
    }
}