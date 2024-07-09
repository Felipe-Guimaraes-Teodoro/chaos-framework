// And then... there was light 🔦

use chaos_framework::*;
use glfw::Key;

fn main() {
    let mut el = EventLoop::new(800, 600);
    let mut renderer = Renderer::new();
    
    unsafe {
        Enable(DEPTH_TEST);
        Enable(CULL_FACE);
        // CullFace(FRONT);
    }
    
    let texture_handle = renderer.add_texture("assets/scenes/textures/diffuse.png").unwrap();

    let mut floor = Quad::new(vec3(250.0, 250.0, 250.0), Vec4::ONE).mesh();
    floor.rotation = Quat::from_euler(EulerRot::XYZ, -3.1415 * 0.5, 0.0, 0.0);
    floor.position = vec3(-125.0, 0.0, 125.0);
    floor.color = vec3(0.3, 0.3, 0.6);
    // renderer.add_mesh(floor).unwrap();

    for _ in 0..260 {
        renderer.add_mesh({
            let mut mesh = Cuboid::new(Vec3::ONE, Vec4::ONE).mesh();
            mesh.position = (rand_vec3() * 2.0 - 1.0) * 20.0 + vec3(0.0, 20.0, 0.0);

            mesh
        }).unwrap();
    }

    el.window.set_cursor_mode(CursorMode::Disabled);
    // el.window.glfw.set_swap_interval(SwapInterval::Sync(0));

    
    let scene = load_scene("assets/scenes/fucker.dae");
    let dance = load_scene("assets/scenes/knight.dae");

    let great_mountains = Model::new("assets/models/m.obj").meshes[0].clone();
    let v = great_mountains.vertices.clone();
    
    let sample_y = |p: Vec3| -> Option<f32> {
        let mut nearest_vertex: Option<&Vec3> = None;
        let mut min_distance = f32::MAX;

        for vertex in &v {
            let distance = ((vertex.position.x - p.x).powi(2) + (vertex.position.y - p.y).powi(2) + (vertex.position.z - p.z).powi(2)).sqrt();
            if distance < min_distance {
                min_distance = distance;
                nearest_vertex = Some(&vertex.position);
            }
        }

        nearest_vertex.map(|vertex| vertex.y)
    };

    for _ in 0..20 { 
        let mut pos = (rand_vec3() * 2.0 - 1.0) * 20.0;
        pos.y = sample_y(pos).unwrap();
        renderer.add_light(Light { position: pos, color: rand_vec3() });
    }

    renderer.add_mesh(great_mountains).unwrap();


    let mut sk_mesh = Model::load_skeletal(&scene);
    sk_mesh.color = Vec3::ONE;
    sk_mesh.set_texture(texture_handle, &renderer);
    sk_mesh.scale = Vec3::ONE;

    let dance_anim = Animation::new(&dance);
    let walk_anim = Animation::new(&scene);
    let mut animator = Animator::new(walk_anim.clone());

    let sk = renderer.add_skeletal_mesh(sk_mesh).unwrap();
    
    let mut is_fullscreen = false;
    let mut speed = 0.0;
    let mut last_pos = Vec3::ZERO;
    let mut angle = 0.0;
    let mut zoom = 2.0;
    let mut new_sample_y = 0.0;

    while !el.window.should_close() {
        el.update();
        renderer.update();

        if el.is_key_down(Key::A) || el.is_key_down(Key::W) || el.is_key_down(Key::S) || el.is_key_down(Key::D) {
            if el.is_key_down(Key::LeftShift) {
                speed = lerp(speed, 10.0, 0.03);
            } else {
                speed = lerp(speed, 5.0, 0.03);
            }
        } else {
            speed = lerp(speed, 0.0, 0.1);
        }

        if el.event_handler.key_just_pressed(Key::F) {
            animator.start_transition(dance_anim.clone(), 0.2);
        }
        if el.is_key_down(Key::F) {
            speed = 1.0;
        }
        if el.event_handler.key_just_released(Key::F) {
            animator.start_transition(walk_anim.clone(), 0.05);
        }

        
        el.set_fullscreen(&mut is_fullscreen);
        
        renderer.camera.input(&el);
        renderer.camera.mouse_callback(el.event_handler.mouse_pos, &el.window);
        if el.event_handler.scroll.y < -0.5 {
            zoom *= 1.2; 
        } else if el.event_handler.scroll.y > 0.5 {
            zoom *= 0.8;
        }
        let goal = renderer.skeletal_meshes[&sk].position + vec3(1.0, 1.0, 1.0) * renderer.camera.front * -zoom + vec3(0.0, 1.2, 0.0);
        renderer.camera.update(goal, &el);
        
        let mesh = &mut renderer.skeletal_meshes.get_mut(&sk).unwrap();
        let mut front = renderer.camera.front;
        front.y = 0.0;
        let front = front.normalize();
        let right = front.cross(Vec3::Y);
        
        
        if el.is_key_down(Key::W) {
            mesh.position += front * el.dt * speed;
        }
        if el.is_key_down(Key::S) {
            mesh.position -= front * el.dt * speed;
        }
        if el.is_key_down(Key::A) {
            mesh.position -= right * el.dt * speed;
        }
        if el.is_key_down(Key::D) {
            mesh.position += right * el.dt * speed;
        }

        let mut clamped_pos = mesh.position;
        clamped_pos.y = 0.0;
        last_pos.y = 0.0;

        if last_pos - clamped_pos != Vec3::ZERO {
            let composite_velocity = last_pos - mesh.position;
            let goal = -composite_velocity.x.atan2(-composite_velocity.z);
            angle = lerp(angle, goal, 0.1);
            if angle - goal > 3.14 {
                angle = -3.14;
            }
            if angle - goal < -3.14 {
                angle = 3.14;
            }

            let rotation = Quat::from_rotation_y(angle);
            mesh.rotation = rotation;
        }

        last_pos = mesh.position;

        if (el.time * 100.0) as i32 % 5 == 0 {
            new_sample_y = sample_y(mesh.position).unwrap();
        }
        mesh.position.y = lerp(mesh.position.y, new_sample_y, speed * 0.05);

        animator.update(el.dt * (front * speed).length());
        
        let frame = el.ui.frame(&mut el.window);
        frame.text("hello, world!");
        frame.text(format!("f: {:?}", 1.0 / el.dt));
        
        unsafe {
            Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
            ClearColor(0.1, 0.2, 0.3, 1.0);
            
            RUSSIMP_SHADER.use_shader();
            animator.upload_uniforms(&RUSSIMP_SHADER);
            UseProgram(0);
            
            renderer.draw();
            el.ui.draw();
        }
    }
}
