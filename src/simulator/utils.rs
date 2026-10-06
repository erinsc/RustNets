use crate::interactionnet::Pid;
use raylib::{RaylibHandle, consts::PI, ffi::Color, prelude::Vector2};


pub const RADIUS: f32 = 12.0;
pub const MINDIST: f32 = RADIUS * 4.0;
pub const CUTOFF: f32 = RADIUS * 4.0;
pub const REPULSION: f32 = 0.05;


pub fn port_offset(port: Pid, count: Pid, angle: f32) -> Vector2 {
    if port == 0 {
        Vector2::new(RADIUS * 0.667, 0.0).rotate(angle)
    } else {
        let x = RADIUS * 2.0 * (port as f32 / count as f32) - RADIUS;
        Vector2::new(-RADIUS/2.0, x * 1.5).rotate(angle)
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
    pub color: Color
}
impl SymbolData {
    pub fn new(color: Color) -> SymbolData { Self {
        color
    }}
}