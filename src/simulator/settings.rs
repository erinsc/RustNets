use crate::interactionnet::{Nid, Pid};
use raylib::{RaylibHandle, consts::PI, ffi::Color, prelude::Vector2, text::WeakFont};

pub struct Ripple {
    pub pos: Vector2,
    pub vel: Vector2,
    pub age: f32
}
impl Ripple {
    pub fn new(pos: Vector2, vel: Vector2) -> Ripple { Self {
        pos, vel, age: 0.0
    }}
}

pub struct Environment {
    pub radius: f32,
    pub edge: f32,

    pub min_dist: f32,
    pub force: f32,

    pub paused: bool,
    pub reducing: bool,
    pub held: Option<Nid>,

    pub font: WeakFont,
    pub ripples: Vec<Ripple>
}
impl Environment {
    pub fn default(rl: &RaylibHandle) -> Environment { Self {
        radius: 12.0,
        edge: 4.0,
        min_dist: 64.0,
        force: 0.1,
        paused: false, reducing: false, held: None, font: rl.get_font_default(),
        ripples: Vec::new()
    }}
}

pub fn port_offset(port: Pid, count: Pid, angle: f32, s: &Environment) -> Vector2 {
    if port == 0 {
        Vector2::new(s.radius * 0.667, 0.0).rotate(angle)
    } else {
        let x = s.radius* 2.0 * (port as f32 / count as f32) - s.radius;
        Vector2::new(-s.radius/2.0, x * 1.5).rotate(angle)
    }
}

#[derive(Clone, Copy)]
pub struct NodeData {
    pub pos: Vector2,
    pub vel: Vector2,
    pub angle: f32,
    pub angle_velocity: f32
}
impl NodeData {
    #[allow(unused)]
    pub fn new(pos: Vector2) -> NodeData {
        Self { pos, vel: Vector2::zero(), angle: 0.0, angle_velocity: 0.0 }
    }
    pub fn random(rl: &RaylibHandle) -> NodeData { Self {
        pos: Vector2::new(rl.get_random_value::<i32>(-100..=100) as f32, 
                          rl.get_random_value::<i32>(-100..=100) as f32), 
        vel: Vector2::zero(), 
        angle: rl.get_random_value::<i32>(0..=359) as f32 * (PI as f32) / 180.0,
        angle_velocity: 0.01
    }}
}
pub struct SymbolData {
    pub color: Color,
    pub label: String
}
impl SymbolData {
    pub fn new(color: Color, label: &str) -> SymbolData { Self {
        color, label: label.to_string()
    }}
}