mod sparsevec;
mod interactionnet;
mod simulator;

use interactionnet::*;
use raylib::{prelude::*};

use crate::simulator::{NodeData, SymbolData};


fn main() -> Result<(), NetError> {
    let (mut rl, thread) = raylib::init()
        .size(800, 450)
        .title("Hello raylib from Rust")
        .build();

    rl.set_target_fps(60);

    let mut net = InteractionNet::<NodeData, SymbolData>::new();
    let epsilon = net.register_symbol("Epsiln", 0, SymbolData::new(Color::CYAN));
    let delta = net.register_symbol("Delta", 2, SymbolData::new(Color::ORANGE));

    net.create_node(epsilon, NodeData::random(&mut rl)).unwrap();
    net.create_node(epsilon, NodeData::random(&mut rl)).unwrap();
    net.create_node(epsilon, NodeData::random(&mut rl)).unwrap();
    net.create_node(delta, NodeData::random(&mut rl)).unwrap();

    net.link(port![0, 0], port![3, 0]).unwrap();
    net.link(port![1, 0], port![3, 1]).unwrap();
    net.link(port![2, 0], port![3, 2]).unwrap();

    let mut camera = Camera2D {
        offset: Vector2::new(500.0, 350.0),
        target: Vector2::zero(),
        rotation: 0.0,
        zoom: 1.0,
    };

    while !rl.window_should_close() {
        simulator::update_camera(&rl, &mut camera);

        simulator::step(&mut net, 0.0);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);
        {
            let mut d2 = d.begin_mode2D(camera);
            simulator::draw_graph(&mut d2, &net);
        }
        //render::draw_hud(&mut d, bodies.len(), edges.len(), paused);
    }

    Ok(())
}

