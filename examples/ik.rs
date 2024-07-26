/*
An example for a simple 2D game using an orthographic camera
*/
use chaos_framework::*;
use glfw::Key;

fn main() {
    let mut el = EventLoop::new(600, 600);
    let mut renderer = Renderer::new();

    unsafe {
        Enable(DEPTH_TEST);
    }

    /* default projection type is perspective */
    renderer.camera.set_projection(ProjectionType::Orthographic);
    renderer.add_light(Light { position: vec3(0.0, 0.0, 3.0), color: Vec3::ONE })
        .unwrap();

    /* we'll represent our player using a quad */
    let player_handle = renderer.add_mesh(Quad::new(Vec3::ONE * 0.1, Vec4::ONE).mesh())
        .unwrap();

    renderer.meshes[player_handle].position = Vec3::Z * 2.0; // so the player stays in front of everything

    let mut segments = vec![];
    for _ in 0..128 {
        let new_segment = Segment::new(vec3(0.0, 0.0, 0.0), 0.05, &mut renderer);

        segments.push(new_segment);
    }

    renderer.meshes[segments[0].handle].hidden = true;
    renderer.meshes[segments[segments.len()-1].handle].hidden = true;

    renderer.meshes[player_handle].hidden = true;
    

    segments[0].pos = Vec3::new(-2.0, -2.0, 0.0);

    let mut clamped_pos;
    let mut player_vel;
    let mut old_pos = Vec3::ZERO;

    while !el.window.should_close() {
        el.update();
        renderer.update();
        
        segments.iter_mut().for_each(|s| {
            s.update(&mut renderer);
        });
        
        let player = &mut renderer.meshes[player_handle];
        let mp = el.event_handler.mouse_pos / el.event_handler.width * 2.0;
        let clamped_player_pos = {
            // clamped_pos = lerp(clamped_pos, player.position, 0.1);
            clamped_pos = player.position;

            vec3(clamped_pos.x, clamped_pos.y, 0.0)
        };
        fabrik(&mut segments, {
            if el.event_handler.lmb {
                clamped_player_pos + vec3(mp.x, mp.y, 0.0)
            } else {
                clamped_player_pos
            }
        }, 0.0, 2);

        let first_segment_position = segments[0].pos;
        let seg_amm = segments.len() as f32;
        let distance = first_segment_position.distance(segments[segments.len()-1].pos);
        let len = segments[0].length;
        if distance > segments.len() as f32 * segments[0].length + segments[0].length {
            segments[0].pos += ((clamped_player_pos - first_segment_position) / seg_amm) * len * f32::powf(distance, 2.3);
        }

        player_vel = player.position - old_pos;
        old_pos = player.position;

        player.color = vec3(0.5, 0.0, el.time.sin());
        move_player(&el, &mut player.position);
    
        renderer.camera.update(lerp(renderer.camera.pos, player.position, 0.125), &el);
        renderer.camera.mouse_callback(el.event_handler.mouse_pos, &el.window);

        let frame = el.ui.frame(&mut el.window);
        frame.text("Hello, world! This is imgui.");
        frame.text(format!("p: {:.1}\nv: {:.3}", player.position, player_vel));

        unsafe {
            Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
            ClearColor(0.1, 0.2, 0.3, 1.0);
            
            renderer.draw();
            el.ui.draw();
        }
    }
}

struct Segment {
    pos: Vec3,
    length: f32,
    angle: f32,
    handle: MeshHandle,
}

impl Segment {
    fn new(a: Vec3, length: f32, renderer: &mut Renderer) -> Self {
        let handle = renderer.add_mesh(Quad::new(Vec3::ONE * length, Vec4::ONE).mesh())
            .unwrap();

        Self {
            pos: a,
            length: length / 2.0,
            angle: 0.0,
            handle: handle,
        }
    }

    fn get_end(&self) -> Vec3 {
        self.pos + Vec3::new(self.length * self.angle.cos(), self.length * self.angle.sin(), 0.0)
    }

    fn update_position(&mut self, new_pos: Vec3) {
        self.pos = new_pos;
    }

    fn update_angle(&mut self, direction: Vec3) {
        self.angle = direction.y.atan2(direction.x);
    }

    fn update(&mut self, renderer: &mut Renderer) {
        let rot = Quat::from_axis_angle(Vec3::Z, self.angle);

        renderer.meshes[self.handle].position = self.pos;
        renderer.meshes[self.handle].rotation = rot;
    }
}

fn fabrik(segments: &mut [Segment], target: Vec3, tolerance: f32, max_iterations: usize) {
    let b = segments[0].pos;
    let mut diff = (segments.last().unwrap().get_end() - target).length();
    let mut iterations = 0;

    while diff > tolerance && iterations < max_iterations {
        iterations += 1;

        segments.last_mut().unwrap().update_position(target);
        for i in (2..segments.len()).rev() {
            let parent_end = segments[i].get_end();
            let direction = (segments[i - 1].pos - parent_end).normalize();
            let new_pos = parent_end + direction * segments[i - 1].length;
            segments[i - 1].update_position(new_pos);
            segments[i - 1].update_angle(direction);
        }

        segments[0].update_position(b);
        for i in 1..segments.len()-1 {
            let parent_end = segments[i - 1].get_end();
            let direction = (segments[i].pos - parent_end).normalize();
            let new_pos = parent_end + direction * segments[i - 1].length;
            segments[i].update_position(new_pos);
            segments[i].update_angle(direction);
        }

        diff = (segments.last().unwrap().get_end() - target).length();
    }
}




fn move_player(el: &EventLoop, pos: &mut Vec3) {
    let mut velocity = Vec3::ZERO; 
    let mut speed = 1.5;

    if el.is_key_down(Key::LeftShift) {
        speed *= 1.5;
    }

    if el.is_key_down(Key::W) {
        velocity.y+=speed;
    }
    if el.is_key_down(Key::S) {
        velocity.y-=speed;
    }
    if el.is_key_down(Key::D) {
        velocity.x+=speed;
    }
    if el.is_key_down(Key::A) {
        velocity.x-=speed;
    }

    *pos += velocity * el.dt;
}
