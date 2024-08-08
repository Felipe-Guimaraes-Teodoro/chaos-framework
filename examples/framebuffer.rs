// And then... there was light 🔦

use chaos_framework::*;
use framebuffer::Framebuffer;
use glfw::Key;

fn main() {
    let mut el = EventLoop::new(800, 600);
    let mut renderer = Renderer::new();
    
    unsafe {
        Enable(DEPTH_TEST);
        // Enable(CULL_FACE);
        // CullFace(FRONT);
    }

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

    let mut framebuffer = Framebuffer::new_texture(800, 600);

    while !el.window.should_close() {
        el.update();
        renderer.update();

        renderer.camera.update(renderer.camera.pos, &el);
        renderer.camera.input(&el);
        renderer.camera.mouse_callback(el.event_handler.mouse_pos, &el.window);

        let frame = el.ui.frame(&mut el.window);
        frame.text("hello, world!");
        frame.text(format!("f: {:?}", 1.0 / el.dt));
        
        unsafe {
            framebuffer.draw_first_pass(&mut renderer);
            Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
            ClearColor(0.1, 0.2, 0.3, 1.0);

            renderer.draw();
            framebuffer.draw_second_pass();

            
            el.ui.draw();
        }
    }
}
