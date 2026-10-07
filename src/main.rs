mod sparsevec;
mod interactionnet;
mod simulator;

use interactionnet::*;
use raylib::{prelude::*};

use crate::simulator::{NodeData, Settings, SymbolData};


fn main() -> Result<(), NetError> {
    let (mut rl, thread) = raylib::init()
        .size(800, 450)
        .title("Hello raylib from Rust")
        .build();

    rl.set_target_fps(60);

    let mut net = InteractionNet::<NodeData, SymbolData>::new();
    let epsilon = net.register_symbol("Epsiln", 0, SymbolData::new(Color::CYAN));
    let delta = net.register_symbol("Delta", 2, SymbolData::new(Color::ORANGE));
    let gamma = net.register_symbol("Gamma", 2, SymbolData::new(Color::ORANGE));

    net.create_node(epsilon, NodeData::random(&mut rl)).unwrap();
    net.create_node(epsilon, NodeData::random(&mut rl)).unwrap();
    net.create_node(epsilon, NodeData::random(&mut rl)).unwrap();
    net.create_node(delta, NodeData::random(&mut rl)).unwrap();

    net.link(port![0, 0], port![3, 0]).unwrap();
    net.link(port![1, 0], port![3, 1]).unwrap();
    net.link(port![2, 0], port![3, 2]).unwrap();

    let mut book = RuleBook::new();

        book.register_rule(rule![
            [epsilon],
            [epsilon]
        ]);
        book.register_rule(rule![
            [epsilon], 
            [gamma, [epsilon], [epsilon]]
        ]);
        book.register_rule(rule![
            [epsilon], 
            [delta, [epsilon], [epsilon]]
        ]);
        book.register_rule(rule![
            [gamma, 0, 1], 
            [gamma, 1, 0]
        ]);
        book.register_rule(rule![
            [delta, 0, 1], 
            [delta, 0, 1]
        ]);
        book.register_rule(rule![
            [gamma, [delta, 0, 1], [delta, 2, 3]], 
            [delta, [gamma, 0, 2], [gamma, 1, 3]]
        ]);

    let mut camera = Camera2D {
        offset: Vector2::new(500.0, 350.0),
        target: Vector2::zero(),
        rotation: 0.0,
        zoom: 1.0,
    };

    let mut settings = Settings::default();

    while !rl.window_should_close() {
        simulator::update_camera(&rl, &mut camera);

        let mouse = rl.get_screen_to_world2D(rl.get_mouse_position(), camera);
        
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            settings.held = simulator::pick_node(&net, mouse, &settings);
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
        
        simulator::step(&mut net, &settings, 0.0);
        simulator::interact(&mut net, &book, &settings)?;
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);
        {
            let mut d2 = d.begin_mode2D(camera);
            simulator::draw_graph(&mut d2, &net, &settings);
        }
        simulator::draw_hud(&mut d, &net, &settings);
    }

    Ok(())
}

