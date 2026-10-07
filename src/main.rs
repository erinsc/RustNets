mod sparsevec;
mod interactionnet;
mod simulator;
mod rulebooks;

use interactionnet::*;
use raylib::{prelude::*};

use crate::{rulebooks::create_peano, simulator::{Environment}};

fn main() -> Result<(), NetError> {
    let (mut net, book) = create_peano();

    let (mut rl, thread) = raylib::init()
        .size(1820, 1080)
        .title("Hello raylib from Rust")
        .build();

    rl.set_target_fps(60);

    let mut camera = Camera2D {
        offset: Vector2::new(500.0, 350.0),
        target: Vector2::zero(),
        rotation: 0.0,
        zoom: 1.0,
    };

    let mut settings = Environment::default(&rl);

    while !rl.window_should_close() {
        let mouse = rl.get_screen_to_world2D(rl.get_mouse_position(), camera);
        
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            settings.held = simulator::pick_node(&net, mouse, &settings);
        }
        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) && settings.held.is_none() {
            camera.target -= rl.get_mouse_delta() * (1.0/camera.zoom);
        }
        let wheel = rl.get_mouse_wheel_move();
        if wheel != 0.0 {
            let mouse = rl.get_mouse_position();
            camera.target = rl.get_screen_to_world2D(mouse, camera);
            camera.offset = mouse;
            camera.zoom = (camera.zoom * (1.0 + wheel * 0.1)).clamp(0.05, 20.0);
        }
        if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
            settings.held = None;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            settings.paused = !settings.paused;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_TAB) {
            settings.reducing = !settings.reducing;
        }

        if let Some(n) = settings.held {
            if let Some(node) = net.get_node_mut(n) {
                let diff = mouse - node.data.pos;
                node.data.vel = diff * 0.1;
            }
        }        
        
        simulator::step(&mut net, &mut settings, 0.0);
        simulator::interact(&mut net, &book, &mut settings)?;
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);
        {
            let mut d2 = d.begin_mode2D(camera);
            simulator::draw_graph(&mut d2, &net, &mut settings);
        }
        simulator::draw_hud(&mut d, &net, &settings);
    }

    Ok(())
}

