// And then... there was light 🔦

use chaos_framework::*;
use glfw::Key;
use russimp::camera;

fn main() {
    let mut el = EventLoop::new(800, 600);
    let mut renderer = Renderer::new();
    
    unsafe {
        Enable(DEPTH_TEST);
        // Enable(CULL_FACE);
        // CullFace(FRONT);
    }
    
    let texture_handle = renderer.add_texture("assets/textures/3e.png").unwrap();

    // dirty hack to make lights more intense
    for _ in 0..1 { 
        renderer.add_light(Light { position: vec3(1.0, 2.0, 1.0), color: Vec3::ONE });
    }

    let mut floor = Quad::new(vec3(250.0, 250.0, 250.0), Vec4::ONE).mesh();
    floor.rotation = Quat::from_euler(EulerRot::XYZ, -3.1415 * 0.5, 0.0, 0.0);
    floor.position = vec3(-125.0, 0.0, 125.0);
    floor.color = vec3(0.3, 0.3, 0.6);
    renderer.add_mesh(floor).unwrap();


    el.window.set_cursor_mode(CursorMode::Disabled);
    // el.window.glfw.set_swap_interval(SwapInterval::Sync(0));

    let (mut sk_mesh, scene) = Model::load_russimp("assets/scenes/fucker.dae");
    sk_mesh.set_texture(texture_handle, &renderer);
    sk_mesh.scale = Vec3::ONE;
    let animation = Animation::new(scene);
    let mut animator = Animator::new(animation);

    let sk = renderer.add_skeletal_mesh(sk_mesh).unwrap();
    
    let mut is_fullscreen = false;

    while !el.window.should_close() {
        el.update();
        if el.is_key_down(Key::A) || el.is_key_down(Key::W) || el.is_key_down(Key::S) || el.is_key_down(Key::D) {
            animator.update_animation(el.dt * 5.0);
        }

        
        el.set_fullscreen(&mut is_fullscreen);
        
        renderer.camera.input(&el);
        renderer.camera.mouse_callback(el.event_handler.mouse_pos, &el.window);
        let goal = renderer.skeletal_meshes[&sk].position + vec3(1.0, 1.0, 1.0) * renderer.camera.front * -2.0 + vec3(0.0, 1.2, 0.0);
        renderer.camera.update(goal, &el);
        
        let mesh = &mut renderer.skeletal_meshes.get_mut(&sk).unwrap();
        let mut front = renderer.camera.front;
        front.y = 0.0;
        let front = front.normalize();
        let angle = front.x.atan2(front.z);
        let right = front.cross(Vec3::Y);

        let speed = 5.0;

        let rotation = Quat::from_rotation_y(angle);
        if el.is_key_down(Key::W) {
            mesh.position += front * el.dt * speed;
            mesh.rotation = rotation;
        }
        if el.is_key_down(Key::S) {
            mesh.position -= front * el.dt * speed;
            mesh.rotation = rotation;
        }
        if el.is_key_down(Key::A) {
            mesh.position -= right * el.dt * speed;
            mesh.rotation = rotation;
        }
        if el.is_key_down(Key::D) {
            mesh.position += right * el.dt * speed;
            mesh.rotation = rotation;
        }
        
        let frame = el.ui.frame(&mut el.window);
        frame.text("hello, world!");

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
