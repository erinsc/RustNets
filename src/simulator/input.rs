use raylib::prelude::*;
use crate::{interactionnet::{Cid, Net}, simulator::{CellData, Settings}};

pub fn pick_closest_cell<S>(net: &Net<CellData, S>, position: Vector2, distance: f32) -> Option<Cid> {
    net.iter().filter_map(|(cid, cell)|
        ((cell.data.pos - position).length_sqr() < distance * distance).then(|| cid)
    ).next()
}

pub fn process_input<S>(net: &mut Net<CellData, S>, rl: &mut RaylibHandle, s: &mut Settings) {
    let mouse_screen = rl.get_mouse_position();
    let mouse = rl.get_screen_to_world2D(mouse_screen, s.camera);
    let wheel = rl.get_mouse_wheel_move();

    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
        s.held = pick_closest_cell(net, mouse, s.radius);
    }

    if let Some(held) = s.held && let Some(cell) = net.get_mut(held) {
        let diff = mouse - cell.data.pos;
        cell.data.vel = diff * 0.1;
        cell.data.pos += diff * 0.1;
        cell.data.spin += wheel / 8.0;
    } else {
        s.camera.target = mouse;
        s.camera.offset = mouse_screen;
        s.camera.zoom = (s.camera.zoom * (1.0 + wheel * 0.1)).clamp(0.05, 20.0);

        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            s.camera.target -= rl.get_mouse_delta() * (1.0/s.camera.zoom);
        }
    }
    if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
        s.held = None;
    }
    if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
        s.paused = !s.paused;
    }
    if rl.is_key_pressed(KeyboardKey::KEY_TAB) {
        s.reducing = !s.reducing;
    }
}
