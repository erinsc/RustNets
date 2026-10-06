use raylib::{prelude::*};
use crate::{
    interactionnet::{InteractionNet, Port}, simulator::{NodeData, RADIUS, SymbolData, port_offset}
};

pub fn update_camera(rl: &RaylibHandle, camera: &mut Camera2D) {
    if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
        camera.target -= rl.get_mouse_delta() * (1.0/camera.zoom);
    }
    let wheel = rl.get_mouse_wheel_move();
    if wheel != 0.0 {
        let mouse = rl.get_mouse_position();
        camera.target = rl.get_screen_to_world2D(mouse, *camera);
        camera.offset = mouse;
        camera.zoom = (camera.zoom * (1.0 + wheel * 0.1)).clamp(0.05, 20.0);
    }
}

pub fn draw_graph<D: RaylibDraw>(
    d: &mut D,
    net: &InteractionNet<NodeData, SymbolData>
) {
    for (nid, left_node) in net.iter_nodes() {
        for (pid, right) in left_node.active_ports() {
            let left = Port::new(nid, pid);
            if left > *right { continue; }

            let right_node = net.get_node_unchecked(right.node);

            let left_pos = left_node.data.pos + port_offset(
                    left.port,
                    left_node.ports.len() as u8,
                    left_node.data.angle
            );
            let right_pos = right_node.data.pos + port_offset(
                right.port,
                right_node.ports.len() as u8,
                right_node.data.angle
            );

            let dist = (left_pos - right_pos).length() / 16.0;

            let mut left_dir = Vector2::new(-RADIUS * dist, 0.0).rotate(left_node.data.angle);
            if left.port == 0 { left_dir *= -1.0 }
            let left_offset = left_pos + left_dir;

            let mut right_dir = Vector2::new(-RADIUS * dist, 0.0).rotate(right_node.data.angle);
            if right.port == 0 { right_dir *= -1.0 }
            let right_offset = right_pos + right_dir;

            d.draw_spline_segment_bezier_cubic(
                left_pos, left_offset, right_offset, right_pos,
                4.0, Color::BLACK
            );
        }
    }
    for (_nid, node) in net.iter_nodes() {
        let angle = node.data.angle * 180.0 / PI as f32;
        
        let symbol = net.get_symbol_unchecked(node.symbol);
        if symbol.arity == 0 {
            let rad = RADIUS * 0.667;
            d.draw_circle_v(node.data.pos, rad * 2.0 / 3.0 + 5.0, Color::DARKGRAY);
            d.draw_circle_v(node.data.pos, rad, symbol.data.color);
        } else {
            d.draw_poly(node.data.pos, 3, RADIUS + 4.0, angle, Color::DARKGRAY);
            d.draw_poly(node.data.pos, 3, RADIUS, angle, symbol.data.color);
        }
    }
}
