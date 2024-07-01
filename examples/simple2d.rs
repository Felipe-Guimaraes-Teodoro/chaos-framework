/*
An example for a simple 2D game using an orthographic camera
*/

use chaos_framework::*;
use glfw::Key;

fn main() {
    let mut el = EventLoop::new(600, 600);
    let mut renderer = Renderer::new();

    /* default projection type is perspective */
    renderer.camera.set_projection(ProjectionType::Orthographic);
    renderer.add_light(Light { position: vec3(0.0, 0.0, 1.0), color: Vec3::ONE })
        .unwrap();

    /* we'll represent our player using a quad */
    let player_handle = renderer.add_mesh(Quad::new(Vec3::ONE * 0.1, Vec4::ONE).mesh())
        .unwrap();

    while !el.window.should_close() {
        el.update();

        /* we can modify the player by indexing into it in the renderer's meshes */
        let player = &mut renderer.meshes[player_handle];
        player.color = vec3(0.5, 0.0, el.time.sin());
        move_player(&el, &mut player.position);
    
        renderer.camera.update(Vec3::ZERO, &el);
    
        let frame = el.ui.frame(&mut el.window);
        frame.text("hello, world! this is imgui");

        unsafe {
            Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
            ClearColor(0.1, 0.2, 0.3, 1.0);
            
            renderer.draw();
            el.ui.draw();
        }
    }
}

fn move_player(el: &EventLoop, pos: &mut Vec3) {
    let mut velocity = Vec3::ZERO; 
    let mut speed = 1.5;

    if el.is_key_down(Key::LeftShift) {
        speed *= 1.5;
    }

    if el.is_key_down(Key::W) {
        velocity.y+=speed;
    }
    if el.is_key_down(Key::S) {
        velocity.y-=speed;
    }
    if el.is_key_down(Key::D) {
        velocity.x+=speed;
    }
    if el.is_key_down(Key::A) {
        velocity.x-=speed;
    }

    *pos += velocity * el.dt;
}
