use raylib::{drawing::{RaylibDraw, RaylibDrawHandle}, prelude::RaylibFont, ffi::{Color, PI}};

use crate::{interactionnet::Net, simulator::{data::{CellData, Settings, SymbolData}, ports::link_positions}};



pub fn draw_effects(d: &mut RaylibDrawHandle, settings: &Settings) {
    for ripple in settings.ripples.iter() {
        let radius = (1.0 - (ripple.age - 1.0) * (ripple.age - 1.0)) * settings.radius*4.0;

        d.draw_ring(ripple.pos, radius, radius + settings.edge * 0.5,
            0.0, 360.0, 24, Color::BLACK.alpha(1.0 - ripple.age));
    }
}

pub fn draw_links<S>(d: &mut RaylibDrawHandle, net: &Net<CellData, S>, settings: &Settings) {
    for (left, right) in net.iter_links() {
        let positions = link_positions(net, left, right, settings);

        let color = if left.port_id == 0 && right.port_id == 0 && settings.reducing
            { Color::RED } else { Color::BLACK };

        d.draw_spline_bezier_cubic(&positions, settings.edge, color);
    }   
}
pub fn draw_cells(d: &mut RaylibDrawHandle, net: &Net<CellData, SymbolData>, settings: &Settings) {
    for (cid, cell) in net.iter() {
        let angle = cell.data.angle * 180.0 / PI as f32;

        let border_color = if let Some(dragged) = settings.held && dragged == cid
            { Color::RED } else { Color::BLACK };
        
        let Some(symbol) = net.get_symbol(cell.symbol_id) else { continue; };

        if symbol.arity == 0 {
            let rad = settings.radius * 2.0/3.0;
            d.draw_circle_v(cell.data.pos, rad + settings.edge, border_color);
            d.draw_circle_v(cell.data.pos, rad, symbol.data.color);
        } else {
            d.draw_poly(cell.data.pos, 3, settings.radius + settings.edge*2.0, angle, border_color);
            d.draw_poly(cell.data.pos, 3, settings.radius, angle, symbol.data.color);
        }
        let text = &symbol.data.label;
        let size = 14.0;
        let spacing = size / 10.0;
        let font = d.get_font_default();
        let dim = font.measure_text(text, size, spacing);
        d.draw_text_ex(&font, text, cell.data.pos - dim * 0.5, size, spacing, Color::BLACK);
    }
}

pub fn draw_hud<C: Clone, S>(
    d: &mut RaylibDrawHandle,
    net: &Net<C, S>,
    s: &Settings
) {
    d.draw_fps(10, 10);
    d.draw_text(&format!("cells: {}", net.len()), 10, 10 + 24, 20, Color::BLACK);
    d.draw_text(&format!("reducing: {}", s.reducing), 10, 10 + 2*24, 20, Color::BLACK);
    d.draw_text(&format!("max velocity: {}", s.max_vel), 10, 10 + 3*24, 20, Color::BLACK);

    if s.paused {
        d.draw_text("PAUSED", 100, 10, 30, Color::RED);
    }
}