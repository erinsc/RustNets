use raylib::prelude::*;
use crate::{
    interactionnet::{InteractionNet, Nid},
    simulator::{Settings, NodeData, SymbolData, link_offsets_unchecked}
};

/// Given a position, find and return the ID of a node present there.
/// If there's more than one option, return the smallest.
pub fn pick_node(
    net: &InteractionNet<NodeData, SymbolData>,
    pos: Vector2,
    s: &Settings
) -> Option<Nid> {
    net.iter_nodes()
        .filter_map(|(i, node)| ((node.data.pos - pos).length() < s.radius).then(|| i))
        .next()
}

/// Draw the graph and other visual elements
pub fn draw_graph<D: RaylibDraw>(
    d: &mut D,
    net: &InteractionNet<NodeData, SymbolData>,
    s: &mut Settings
) {
    for ripple in s.ripples.iter() {
        let radius = (1.0 - (ripple.age - 1.0) * (ripple.age - 1.0)) * s.radius*4.0;
        d.draw_ring(ripple.pos, radius, radius + s.edge * 0.5,
            0.0, 360.0, 24, Color::BLACK.alpha(1.0 - ripple.age));
    }
    for (left, right) in net.iter_links() {
        let positions = link_offsets_unchecked(left, right, net, s);

        let color = if left.port == 0 && right.port == 0 && s.reducing
            { Color::RED } else { Color::BLACK };

        d.draw_spline_bezier_cubic(&positions, s.edge, color);
    }
    for (nid, node) in net.iter_nodes() {
        let angle = node.data.angle * 180.0 / PI as f32;

        let edge = if let Some(dragged) = s.held && dragged == nid { Color::BLUE } else { Color::BLACK };
        
        let symbol = net.get_symbol_unchecked(node.symbol);
        if symbol.arity == 0 {
            let rad = s.radius * 0.667;
            d.draw_circle_v(node.data.pos, rad * 2.0 / 3.0 + s.edge*1.6, edge);
            d.draw_circle_v(node.data.pos, rad, symbol.data.color);
        } else {
            d.draw_poly(node.data.pos, 3, s.radius + s.edge*2.0, angle, edge);
            d.draw_poly(node.data.pos, 3, s.radius, angle, symbol.data.color);
        }
        let text = symbol.data.label.as_ref();
        let size = 14.0;
        let spacing = size / 10.0;
        let dim = s.font.measure_text(text, size, spacing);
        d.draw_text_ex(&s.font, text, node.data.pos - dim * 0.5, size, spacing, Color::BLACK);
    }
}

pub fn draw_hud<D: RaylibDraw>(
    d: &mut D,
    net: &InteractionNet<NodeData, SymbolData>,
    s: &Settings
) {
    d.draw_fps(10, 10);
    d.draw_text(&format!("nodes: {}", net.count()), 10, 10 + 24, 20, Color::BLACK);
    d.draw_text(&format!("reducing: {}", s.reducing), 10, 10 + 2*24, 20, Color::BLACK);
    d.draw_text(&format!("max velocity: {}", s.max_vel), 10, 10 + 3*24, 20, Color::BLACK);

    if s.paused {
        d.draw_text("PAUSED", 100, 10, 30, Color::RED);
    }
}