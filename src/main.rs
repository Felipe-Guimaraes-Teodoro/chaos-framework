// And then... there was light 🔦

use chaos_framework::*;
use glfw::Key;

fn main() {
    let mut el = EventLoop::new(600, 600);
    let mut renderer = Renderer::new();
    
    unsafe {
        Enable(DEPTH_TEST);
        // Enable(CULL_FACE);
        // CullFace(FRONT);
    }
    
    let texture_handle = renderer.add_texture("assets/textures/3e.png").unwrap();

    for _ in 0..100 {
        renderer.add_light(Light { position: vec3(10.0, 10.0, 10.0), color: Vec3::ONE });
    }

    let mut floor = Quad::new(vec3(250.0, 250.0, 250.0), Vec4::ONE).mesh();
    floor.rotation = Quat::from_euler(EulerRot::XYZ, -3.1415 * 0.5, 0.0, 0.0);
    floor.position = vec3(-125.0, 0.0, 125.0);
    floor.color = vec3(0.01, 0.01, 0.02);
    renderer.add_mesh(floor).unwrap();
    
    let result = test();
    let mut smesh = result.0;
    let knight_texture_handle = renderer.add_texture("assets/scenes/textures/diffuse.png")
        .unwrap();
    // smesh.set_texture(knight_texture_handle, &renderer);
    smesh.color = vec3(1.0, 1.0, 1.0);
    let smesh_handle = renderer.add_animated_mesh(smesh).unwrap();

    let mut animator = result.1;

    el.window.set_cursor_mode(CursorMode::Disabled);
    // el.window.glfw.set_swap_interval(SwapInterval::Sync(0));
    
    let mut is_fullscreen = false;
    let mut zoom = 5.0;
    let mut sod = SecondOrderDynamics::new(2.0, 0.5, 1.0, Vec3::ZERO);

    let mut valor_arbitrario = 0.0;

    while !el.window.should_close() {
        // el.glfw.poll_events();
        el.update();
        animator.update_animation(el.dt);
        el.set_fullscreen(&mut is_fullscreen);
        
        renderer.camera.input(&el);
        renderer.camera.mouse_callback(el.event_handler.mouse_pos, &el.window);
        renderer.camera.update(renderer.camera.pos, &el);
        
        let mesh2 = &mut renderer.animated_meshes[smesh_handle];

        let mut goal = mesh2.position;
        if el.event_handler.lmb {
            goal = renderer.camera.pos + renderer.camera.front * zoom;
            let result = sod.update(el.dt, goal);
            mesh2.position = result;
        } else {
            let result = sod.update(el.dt, goal);
            mesh2.position = result;
        }

        if el.event_handler.scroll.y >= 1.0 {
            zoom += 0.25;
        } else if el.event_handler.scroll.y <= -1.0  {
            zoom -= 0.25;
        } 

        if el.is_key_down(Key::LeftAlt) {
            el.window.set_cursor_mode(CursorMode::Normal);
        } else {
            el.window.set_cursor_mode(CursorMode::Disabled);
        }

        let frame = el.ui.frame(&mut el.window);
        frame.text("hello, world!");
        frame.slider("valor", 0.0, 2.0, &mut valor_arbitrario);

        unsafe {
            Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
            ClearColor(0.1, 0.2, 0.3, 1.0);
            
            RUSSIMP_SHADER.use_shader();
            animator.upload_uniforms(&RUSSIMP_SHADER);
            UseProgram(0);

            renderer.draw();
            el.ui.draw();
        }

        // el.window.swap_buffers();
    }
}
