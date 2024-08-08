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

    el.window.set_cursor_mode(CursorMode::Disabled);
    el.window.glfw.set_swap_interval(SwapInterval::Sync(1));

    let mut trail = Trail::new(&mut renderer, 64);
    trail.thickness = 0.05;
    
    let mut ofs = rand_vec3() * 3.0;

    while !el.window.should_close() {
        el.update();
        renderer.update();
        trail.update(&mut renderer);

        trail.position = (vec3(el.time.sin(), (el.time * 5.0).sin(), el.time.cos())) * 0.1 + lerp(trail.position, ofs, 0.05);
        
        if el.time as i32 % 3 == 0 {
            ofs = rand_vec3() * 3.0;
        }

        renderer.camera.input(&el);
        renderer.camera.mouse_callback(el.event_handler.mouse_pos, &el.window);
        renderer.camera.update(renderer.camera.pos, &el);

        let frame = el.ui.frame(&mut el.window);
        frame.text("hello, world!");

        unsafe {
            Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
            ClearColor(0.1, 0.2, 0.3, 1.0);
            
            renderer.draw();
            el.ui.draw();
        }
    }
}
