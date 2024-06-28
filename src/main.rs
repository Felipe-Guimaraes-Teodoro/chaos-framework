// And then... there was light 🔦

use chaos_framework::*;

fn main() {
    let mut el = EventLoop::new(600, 600);
    let mut renderer = Renderer::new();
    
    unsafe {
        Enable(DEPTH_TEST);
    }
    
    let texture_handle = renderer.add_texture("assets/textures/3e.png").unwrap();
    let mut mesh = Sphere::new(1024, 1.5, Vec4::ONE).mesh();
    mesh.set_texture(texture_handle, &renderer);

    let mut floor = Quad::new(vec3(25.0, 25.0, 25.0), Vec4::ONE).mesh();
    floor.rotation = Quat::from_euler(EulerRot::XYZ, 3.1415 * 0.5, 0.0, 0.0);
    floor.color = vec3(0.1, 0.4, 0.8);
    renderer.add_mesh(floor).unwrap();
    
    let handle = renderer.add_mesh(mesh).unwrap();

    let mut quad = Sphere::new(7, 0.25, Vec4::ONE).mesh();
    for _ in 0..50 {
        let pos = rand_vec3() * 25.0;
        let col = rand_vec3();
        quad.position = pos;
        quad.set_color(col);
        renderer.add_mesh(quad.clone()).unwrap();
        renderer.add_light(Light { position: pos, color: col }).unwrap();
    }
    
    el.window.set_cursor_mode(CursorMode::Disabled);
    //el.window.glfw.set_swap_interval(SwapInterval::Sync(0));
    
    let mut is_fullscreen = false;
    while !el.window.should_close() {
        // el.glfw.poll_events();
        el.update();
        el.set_fullscreen(&mut is_fullscreen);
        
        renderer.camera.input(&el, &el.window.glfw);
        renderer.camera.mouse_callback(el.event_handler.mouse_pos, &el.window);
        renderer.camera.update(renderer.camera.pos);
        
        let mesh = &mut renderer.meshes[handle];
        if el.event_handler.lmb {
            mesh.position = renderer.camera.pos + renderer.camera.front * 5.0;
        }
        // mesh.color = vec3(0.5, 0.9, 1.0);
        
        unsafe {
            Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
            ClearColor(0.1, 0.2, 0.3, 1.0);
            
            renderer.draw();
        }

        // el.window.swap_buffers();
    }
}
