# Chaos Framework (WIP)

## What is it?
It's a framework which aims to have the capabilities of creating cool games while still easy to use.

It currently supports basic graphics with OpenGL and glfw for windowing. It also has Imgui built-in and uses tobj and assimp for the loading of models.

## Features
  * Basic 2D and 3D rendering (Textures, camera, simple lighning, etc.)
  * Model loading
  * Skeletal animation

## Plans for the future
Near future: instancing; particles; parallelization; late latching; skybox (atmosphere shader and cubemaps); sound; make it faster; improve the lightning systems; abstract OpenGL even more
Not-so-near future: Vulkan instead of OpenGL; support for Web (Might use WGPU); sound synthesizer engine; 

## Simple 2D example code (see in examples folder):
```rs
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
```

## Current issues
A couple of examples are missing, such as the ones which include rigged meshes, use of destroy_mesh(), model loading, custom shaders and custom meshes, textures, macroes, event handling.
There also seems to be a very slight input delay, although slight enough to be barely noticeable to the point of mild frustration.
There might be memory leaks in the model loading? There might also be missing implementations of Drop.
