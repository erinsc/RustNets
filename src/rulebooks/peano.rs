use raylib::ffi::{Color, Vector2};
use crate::{interactionnet::{InteractionNet, Port, RuleBook}, rule, simulator::{NodeData, SymbolData}};

pub fn create_peano() -> (InteractionNet<NodeData, SymbolData>, RuleBook) {
    let mut net = InteractionNet::<NodeData, SymbolData>::new();
    
    let z = net.register_symbol("Z", 0, SymbolData::new(Color::WHITE, "0"));
    let s = net.register_symbol("S", 1, SymbolData::new(Color::YELLOW, "S"));
    let add = net.register_symbol("Add", 2, SymbolData::new(Color::ORANGE, "+"));
    let mult = net.register_symbol("Add", 2, SymbolData::new(Color::ORANGE, "*"));
    let delta = net.register_symbol("Delta", 2, SymbolData::new(Color::RED, "d"));
    let epsilon = net.register_symbol("Epsln", 0, SymbolData::new(Color::CYAN, "e"));
    
    let mut book = RuleBook::new("", "");

    book.register_rule(rule![
        [add, 0, 0],
        [z]
    ]);
    book.register_rule(rule![
        [add, [s, 0], 1], 
        [s, [add, 0, 1]]
    ]);
    book.register_rule(rule![
        [mult, 0, [delta, 1, 2]], 
        [s, [mult, [add, 0, 2], 1]]
    ]);
    book.register_rule(rule![
        [mult, [z], [epsilon]],
        [z]
    ]);
    book.register_rule(rule![
        [delta, [s, 0], [s, 1]],
        [s, [delta, 0, 1]]
    ]);
    book.register_rule(rule![
        [delta, [z], [z]],
        [z]
    ]);
    book.register_rule(rule![
        [epsilon],
        [s, [epsilon]]
    ]);
    book.register_rule(rule![
        [epsilon],
        [z]
    ]);

    let numeral = |net: &mut InteractionNet<NodeData, SymbolData>, n: usize| {
        let mut top = net.create_node(z, NodeData::new(Vector2::zero())).unwrap();
        for _ in 0..n {
            let succ = net.create_node(s, NodeData::new(Vector2::zero())).unwrap();
            net.link(Port::new(succ, 1), Port::new(top, 0)).unwrap(); // S's argument = previous numeral
            top = succ;
        }
        top // its port 0 is the numeral's output
    };

    let four  = numeral(&mut net, 4); // S(S(S(S(Z))))
    let three = numeral(&mut net, 3); // S(S(S(Z)))
    let m = net.create_node(mult, NodeData::new(Vector2::zero())).unwrap();

    net.link(Port::new(m, 0), Port::new(four, 0)).unwrap();  // first operand on the principal port
    net.link(Port::new(m, 1), Port::new(three, 0)).unwrap();

    (net, book)
}