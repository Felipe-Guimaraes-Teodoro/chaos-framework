// And then... there was light 🔦

use chaos_framework::*;
use glfw::Key;

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
    for _ in 0..50 { 
        renderer.add_light(Light { position: vec3(1.0, 2.0, 1.0), color: Vec3::ONE });
    }

    let mut floor = Quad::new(vec3(250.0, 250.0, 250.0), Vec4::ONE).mesh();
    floor.rotation = Quat::from_euler(EulerRot::XYZ, -3.1415 * 0.5, 0.0, 0.0);
    floor.position = vec3(-125.0, 0.0, 125.0);
    floor.color = vec3(0.01, 0.01, 0.02);
    renderer.add_mesh(floor).unwrap();


    el.window.set_cursor_mode(CursorMode::Disabled);
    // el.window.glfw.set_swap_interval(SwapInterval::Sync(0));

    let (mut sk_mesh, scene) = Model::load_russimp("assets/scenes/knight.dae");
    sk_mesh.set_texture(texture_handle, &renderer);
    sk_mesh.scale = Vec3::ONE * 0.01;
    let animation = Animation::new("assets/scenes/knight.dae", scene);
    let mut animator = Animator::new(animation);

    renderer.add_skeletal_mesh(sk_mesh).unwrap();
    
    let mut is_fullscreen = false;

    while !el.window.should_close() {
        el.update();
        animator.update_animation(el.dt);
        el.set_fullscreen(&mut is_fullscreen);
        
        renderer.camera.input(&el);
        renderer.camera.mouse_callback(el.event_handler.mouse_pos, &el.window);
        renderer.camera.update(renderer.camera.pos, &el);

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
