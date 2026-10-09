use crate::interactionnet::{InteractionNet, Nid, Pid, Port};
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

pub struct Settings {
    pub radius: f32,
    pub edge: f32,

    pub min_dist: f32,
    pub max_vel: f32,
    pub force: f32,

    pub paused: bool,
    pub reducing: bool,
    pub min_reducting_dist: f32,
    pub held: Option<Nid>,

    pub font: WeakFont,
    pub ripples: Vec<Ripple>
}
impl Settings {
    pub fn default(rl: &RaylibHandle) -> Settings { Self {
        radius: 12.0,
        edge: 4.0,
        min_dist: 96.0,
        max_vel: 1.0,
        force: 0.5,
        paused: false, reducing: false, held: None, font: rl.get_font_default(),
        min_reducting_dist: 24.0,
        ripples: Vec::new()
    }}
}

pub fn link_offsets_unchecked(
    left: Port, right: Port,
    net: &InteractionNet<NodeData, SymbolData>,
    s: &Settings
) -> [Vector2; 4] {
    let left_node = net.get_node_unchecked(left.node);
    let right_node = net.get_node_unchecked(right.node);
    
    let dist = (left_node.data.pos - right_node.data.pos).length() * 0.5;
    let left_angle = left_node.data.angle;
    let right_angle = right_node.data.angle;

    let left_pos = port_position(left.port, left_node.ports.len(), s)
        .rotate(left_angle) + left_node.data.pos;
    let right_pos = port_position(right.port, right_node.ports.len(), s)
        .rotate(right_angle) + right_node.data.pos;

    let left_offset = port_offset(left.port, dist)
        .rotate(left_angle) + left_pos;
    let right_offset = port_offset(right.port, dist)
        .rotate(right_angle) + right_pos;

    [left_pos, left_offset, right_offset, right_pos]
}

fn port_offset(port: Pid, length: f32) -> Vector2 {
    if port == 0 {
        Vector2::new(length, 0.0)
    } else {
        Vector2::new(-length, 0.0)
    }
}
fn port_position(port: Pid, count: usize, s: &Settings) -> Vector2 {
    if port == 0 {
        Vector2::new(s.radius * 2.0/3.0, 0.0)
    } else {
        let x = s.radius* 2.0 * (port as f32 / count as f32) - s.radius;
        Vector2::new(-s.radius/2.0, x * 1.5)
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
    pub fn new_random(rl: &RaylibHandle) -> NodeData { Self {
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