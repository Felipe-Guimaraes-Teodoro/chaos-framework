// And then... there was light 🔦

use chaos_framework::*;
use culler::Culler;
use glfw::Key;

fn main() {
    let mut el = EventLoop::new(800, 600);
    let mut renderer = Renderer::new();

    el.window.glfw.set_swap_interval(SwapInterval::Sync(0));
    
    unsafe {
        Enable(DEPTH_TEST);
        Enable(CULL_FACE);
        // CullFace(FRONT);
    }

    let mut floor = Quad::new(vec3(250.0, 250.0, 250.0), Vec4::ONE).mesh();
    floor.rotation = Quat::from_euler(EulerRot::XYZ, -3.1415 * 0.5, 0.0, 0.0);
    floor.position = vec3(-125.0, 0.0, 125.0);
    floor.color = vec3(0.3, 0.3, 0.6);
    // renderer.add_mesh(floor).unwrap();
    let mut culler = Culler::new(&renderer);

    let size = 30.0;
    for _ in 0..1024 {
        let handle = renderer.add_mesh({
            let mut mesh = Cuboid::new(Vec3::ONE, Vec4::ONE).mesh();
            mesh.position = (rand_vec3() * 2.0 - 1.0) * size + vec3(0.0, size, 0.0);

            mesh
        }).unwrap();

        culler.add_mesh(handle);
    }

    el.window.glfw.set_swap_interval(SwapInterval::Sync(0));

    let mut v = 0.1;
    let mut b = true;
    let mut button_pressed = true;

    while !el.window.should_close() {
        el.update();
        renderer.update();
        if b {
            culler.update(&mut renderer, &el);
        }

        renderer.camera.input(&el);
        renderer.camera.mouse_callback(el.event_handler.mouse_pos, &el.window);
        renderer.camera.update(renderer.camera.pos, &el);

        let frame = el.ui.frame(&mut el.window);
        frame.text("hello, world!\nTIP: hold alt to toggle mouse mode");
        frame.text(
            format!("sent to gpu: {:?}\ntotal: {:?}", count_meshes_sent_to_the_gpu(&renderer), renderer.meshes.len())
        );
        frame.text(format!("frametime: {:.1}ms", el.dt*1000.0));

        let button_clicked = frame.button("pause culling");
        
        if button_clicked && !button_pressed {
            b = !b;
            button_pressed = true;
        } else if !button_clicked {
            button_pressed = false;
        }

        if el.is_key_down(Key::LeftAlt) {
            el.window.set_cursor_mode(CursorMode::Normal);
        } else {
            el.window.set_cursor_mode(CursorMode::Disabled);
        }


        unsafe {
            Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
            ClearColor(0.1, 0.2, 0.3, 1.0);
            
            renderer.draw();
            el.ui.draw();
        }

    }
}

fn count_meshes_sent_to_the_gpu(renderer: &Renderer) -> i32 {
    let mut count = 0;

    for mesh in renderer.meshes.values() {
        if mesh.hidden == false {
            count += 1;
        }
    }

    count
}
