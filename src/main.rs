mod interactionnet;
mod simulator;
//mod rulebooks;

use interactionnet::*;
use raylib::{prelude::*};

use crate::simulator::{CellData, Settings, SymbolData};

//use crate::{rulebooks::create_peano, simulator::{Settings}};

fn main() -> Result<(), NetError> {
    let (mut rl, thread) = raylib::init()
        .size(1820, 1080)
        .title("Rustnets")
        .resizable()
        .build();

    rl.set_target_fps(60);

    let mut settings = Settings::default();

    let mut book = Rulebook::new("Lafont", "lafonts universal rules");
    let gamma = book.add_symbol("Gamma", 2, SymbolData::new(Color::YELLOW, "g"));
    let delta = book.add_symbol("Delta", 2, SymbolData::new(Color::RED, "d"));
    let epsilon = book.add_symbol("Epsiln", 0, SymbolData::new(Color::CYAN, "e"));

    book.add(rule![epsilon],
                rule![epsilon]).unwrap();
    book.add(rule![epsilon], 
                rule![gamma, [epsilon], [epsilon]]).unwrap();
    book.add(rule![epsilon], 
                rule![delta, [epsilon], [epsilon]]).unwrap();
    book.add(rule![gamma, 0, 1], 
                rule![gamma, 1, 0]).unwrap();
    book.add(rule![delta, 0, 1], 
                rule![delta, 0, 1]).unwrap();
    book.add(rule![gamma, [delta, 0, 1], [delta, 2, 3]], 
                rule![delta, [gamma, 0, 2], [gamma, 1, 3]]).unwrap();

    let mut net = Net::new(book);

    net.insert(epsilon, CellData::new(Vector2::new(0.0, 0.0))).unwrap();
    net.insert(epsilon, CellData::new(Vector2::new(0.0, 100.0))).unwrap();
    net.insert(gamma, CellData::new(Vector2::new(100.0, 0.0))).unwrap();
    net.insert(delta, CellData::new(Vector2::new(100.0, 100.0))).unwrap();

    //net.link(Port::new(0, 0), Port::new(1, 0)).unwrap();

    net.link(Port::new(0, 0), Port::new(2, 2)).unwrap();
    net.link(Port::new(1, 0), Port::new(3, 1)).unwrap();
    net.link(Port::new(2, 1), Port::new(3, 2)).unwrap();
    net.link(Port::new(2, 0), Port::new(3, 0)).unwrap();


    while !rl.window_should_close() {
        simulator::process_input(&mut net, &mut rl, &mut settings);
        simulator::step_net(&mut net, &mut settings);
        
        simulator::interact_net(&mut net, &mut settings)?;

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        { // Block to drop d2 automatically
            let mut d2 = d.begin_mode2D(settings.camera);
            simulator::draw_effects(&mut d2, &settings);
            simulator::draw_links(&mut d2, &net, &settings);
            simulator::draw_cells(&mut d2, &net, &settings);
        }
        simulator::draw_hud(&mut d, &net, &settings);
    }

    Ok(())
}
