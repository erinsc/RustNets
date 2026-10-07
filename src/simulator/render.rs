use raylib::{prelude::*};
use crate::{
    interactionnet::{InteractionNet, Nid, Port}, simulator::{NodeData, Environment, SymbolData, port_offset}
};

pub fn pick_node(
    net: &InteractionNet<NodeData, SymbolData>,
    pos: Vector2,
    s: &Environment
) -> Option<Nid> {
    net.iter_nodes()
        .filter_map(|(i, node)| ((node.data.pos - pos).length() < s.radius).then(|| i))
        .next()
}

pub fn draw_graph<D: RaylibDraw>(
    d: &mut D,
    net: &InteractionNet<NodeData, SymbolData>,
    s: &mut Environment
) {
    for (nid, left_node) in net.iter_nodes() {
        for (pid, right) in left_node.active_ports() {
            let left = Port::new(nid, pid);
            if left > *right { continue; }

            let right_node = net.get_node_unchecked(right.node);

            let left_pos = left_node.data.pos + port_offset(
                    left.port,
                    left_node.ports.len() as u8,
                    left_node.data.angle,
                    s
            );
            let right_pos = right_node.data.pos + port_offset(
                right.port,
                right_node.ports.len() as u8,
                right_node.data.angle,
                s
            );

            let dist = (left_pos - right_pos).length() / 16.0;
            let color = if left.port == 0 && right.port == 0 && s.reducing
                { Color::RED } else { Color::BLACK };

            let mut left_dir = Vector2::new(s.radius * dist, 0.0).rotate(left_node.data.angle);
            if left.port != 0 { left_dir *= -1.0 }
            let left_offset = left_pos + left_dir;

            let mut right_dir = Vector2::new(s.radius * dist, 0.0).rotate(right_node.data.angle);
            if right.port != 0 { right_dir *= -1.0 }
            let right_offset = right_pos + right_dir;

            d.draw_spline_segment_bezier_cubic(
                left_pos, left_offset, right_offset, right_pos,
                s.edge, color
            );
        }
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
    for ripple in s.ripples.iter() {
        let radius = (1.0 - (ripple.age - 1.0) * (ripple.age - 1.0)) * s.radius*4.0;
        d.draw_ring(ripple.pos, radius, radius + s.edge,
            0.0, 360.0, 24, Color::BLACK.alpha(1.0 - ripple.age));
    }
}

pub fn draw_hud<D: RaylibDraw>(
    d: &mut D,
    net: &InteractionNet<NodeData, SymbolData>,
    s: &Environment
) {
    d.draw_fps(10, 10);
    d.draw_text(&format!("nodes: {}", net.count()), 10, 10 + 24, 20, Color::BLACK);
    d.draw_text(&format!("reducing: {}", s.reducing), 10, 10 + 2*24, 20, Color::BLACK);

    if s.paused {
        d.draw_text("PAUSED", 100, 10, 30, Color::RED);
    }
}