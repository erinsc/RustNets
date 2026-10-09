use raylib::ffi::Color;
use crate::{interactionnet::{InteractionNet, RuleBook}, rule, simulator::{NodeData, SymbolData}};

pub fn create_rules_lafont() -> (InteractionNet<NodeData, SymbolData>, RuleBook) {
    let mut net = InteractionNet::<NodeData, SymbolData>::new();

    let epsilon = net.register_symbol("Epsiln", 0, SymbolData::new(Color::CYAN, "e"));
    let delta = net.register_symbol("Delta", 2, SymbolData::new(Color::RED, "d"));
    let gamma = net.register_symbol("Gamma", 2, SymbolData::new(Color::YELLOW, "g"));

    let mut book = RuleBook::new("", "");

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

    (net, book)
}