// And then... there was light 🔦

use chaos_framework::*;
use glfw::Key;

fn main() {
    let mut el = EventLoop::new(600, 600);
    let mut renderer = Renderer::new();
    
    unsafe {
        Enable(DEPTH_TEST);
    }
    
    let texture_handle = renderer.add_texture("assets/textures/3e.png").unwrap();
    let mut mesh = Sphere::new(1024, 1.5, Vec4::ONE).mesh();
    mesh.set_texture(texture_handle, &renderer);

    let mut floor = Quad::new(vec3(250.0, 250.0, 250.0), Vec4::ONE).mesh();
    floor.rotation = Quat::from_euler(EulerRot::XYZ, -3.1415 * 0.5, 0.0, 0.0);
    floor.position = vec3(-125.0, 0.0, 125.0);
    floor.color = vec3(0.1, 0.1, 0.1);
    renderer.add_mesh(floor).unwrap();
    
    let _handle = renderer.add_mesh(mesh).unwrap();

    let mut quad = Sphere::new(7, 0.25, Vec4::ONE).mesh();
    let mut quads = vec![];
    for _ in 0..25 {
        let pos = rand_vec3() * 25.0;
        let col = rand_vec3();
        quad.position = pos;
        quad.set_color(col);
        let quad_handle = renderer.add_mesh(quad.clone()).unwrap();
        quads.push(quad_handle);
        renderer.add_light(Light { position: pos, color: col }).unwrap();
    }

    let mut bart = Model::new("assets/models/untitled.obj");
    for mesh in bart.meshes.iter_mut() {
        mesh.set_texture(texture_handle, &renderer);
    }
    let bart_handle = renderer.add_model(bart).unwrap();
    
    el.window.set_cursor_mode(CursorMode::Disabled);
    // el.window.glfw.set_swap_interval(SwapInterval::Sync(0));
    
    let mut is_fullscreen = false;
    let mut zoom = 5.0;
    let mut sod = SecondOrderDynamics::new(2.0, 0.5, 1.0, Vec3::ZERO);

    while !el.window.should_close() {
        // el.glfw.poll_events();
        el.update();
        el.set_fullscreen(&mut is_fullscreen);
        
        renderer.camera.input(&el, &el.window.glfw);
        renderer.camera.mouse_callback(el.event_handler.mouse_pos, &el.window);
        renderer.camera.update(renderer.camera.pos);
        
        let model = &mut renderer.models[bart_handle];
        let mut goal = model.meshes[0].position;
        if el.event_handler.lmb {
            goal = renderer.camera.pos + renderer.camera.front * zoom;
            let result = sod.update(el.dt, goal);
            for mesh in model.meshes.iter_mut() {
                mesh.position = result;
            }
        } else {
            let result = sod.update(el.dt, goal);
            for mesh in model.meshes.iter_mut() {
                mesh.position = result;
            }
        }

        for ball in quads.iter_mut() {
            let mesh = &mut renderer.meshes[*ball];
            mesh.position += (rand_vec3() * 2.0 - 1.0) * el.dt * 8.0;
        }

        let mut counter = 0;
        for light in renderer.lights.values_mut() {
            let pos = renderer.meshes[quads[counter]].position;
            renderer.meshes[quads[counter]].color = light.color;
            light.color = vec3(
                (counter as f32 / 20.0) % 1.0,
                (counter as f32 * 7424.0 / 50.0) % 1.0, 
                (counter as f32 * 134.0 / 50.0) % 1.0,  
            ) + rand_vec3() * 0.1;
            light.position = pos;

            counter += 1;
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

        // mesh.color = vec3(0.5, 0.9, 1.0);

        unsafe {
            Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
            ClearColor(0.1, 0.2, 0.3, 1.0);
            
            renderer.draw();
            el.ui.draw();
        }

        // el.window.swap_buffers();
    }
}
