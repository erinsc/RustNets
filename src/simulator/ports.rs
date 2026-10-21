use raylib::prelude::Vector2;

use crate::{interactionnet::{Cell, Net, Pid, Port}, simulator::data::{CellData, Settings}};

pub fn link_positions<S>(net: &Net<CellData, S>, left: Port, right: Port, settings: &Settings) -> [Vector2; 4] {
    let left_cell = net.get(left.cell_id).unwrap();
    let right_cell = net.get(right.cell_id).unwrap();

    let distance = (left_cell.data.pos - right_cell.data.pos).length() * 0.5;
    let left = port_positions(left_cell, left.port_id, distance, settings);
    let right = port_positions(right_cell, right.port_id, distance, settings);

    [left.0, left.1, right.1, right.0]
}

fn port_positions(cell: &Cell<CellData>, port: Pid, distance: f32, settings: &Settings) -> (Vector2, Vector2) {
    let pos = port_offset(cell, port, settings) + cell.data.pos;
    let offset = port_control(port, distance)
        .rotate(cell.data.angle) + pos;

    (pos, offset)
}

fn port_control(port: Pid, distance: f32) -> Vector2 {
    if port == 0 {
        Vector2::new(distance, 0.0)
    } else {
        Vector2::new(-distance, 0.0)
    }
}
pub fn port_offset(cell: &Cell<CellData>, port: Pid, s: &Settings) -> Vector2 {
    if port == 0 {
        Vector2::new(s.radius * 2.0/3.0, 0.0).rotate(cell.data.angle)
    } else {
        let x = s.radius* 2.0 * (port as f32 / cell.ports.len() as f32) - s.radius;
        Vector2::new(-s.radius/2.0, x * 1.5).rotate(cell.data.angle)
    }
}