// And then... there was light 🔦

use chaos_framework::*;

fn main() {
    let mut el = EventLoop::new(600, 600);
    let mut renderer = Renderer::new();

    unsafe {
        Enable(DEPTH_TEST);
    }

    let mesh = Sphere::new(16, 1.5, Vec4::ONE).mesh();

    let handle = renderer.add_mesh(mesh).unwrap();
    let light_handle = renderer.add_light(Light { position: Vec3::ONE * 10.0, color: Vec3::ONE }).unwrap();

    el.window.set_cursor_mode(CursorMode::Disabled);

    while !el.window.should_close() {
        el.update();
        renderer.camera.input(&el.window, &el.window.glfw);
        renderer.camera.mouse_callback(el.event_handler.mouse_pos, &el.window);
        renderer.camera.update(renderer.camera.pos);

        let mesh = &mut renderer.meshes[handle];
        mesh.color = vec3(0.5, 0.9, 1.0);
        
        unsafe {
            Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
            ClearColor(0.1, 0.2, 0.3, 1.0);

            renderer.draw();
        }
    }
}
