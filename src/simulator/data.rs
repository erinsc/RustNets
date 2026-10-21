use raylib::{ffi::Camera2D, prelude::{Color, Vector2}};

use crate::interactionnet::Cid;

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

#[derive(Clone, Copy)]
pub struct CellData {
    pub pos: Vector2,
    pub vel: Vector2,
    pub angle: f32,
    pub spin: f32
}
impl CellData {
    pub fn new(pos: Vector2) -> CellData { CellData {
        pos, vel: Vector2::zero(), angle: 0.0, spin: 0.0
    }}
}

#[derive(Clone)]
pub struct SymbolData {
    pub color: Color,
    pub label: String
}
impl SymbolData {
    pub fn new(color: Color, label: &str) -> SymbolData { Self {
        color, label: label.to_string()
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
    pub reduction_distance: f32,

    pub camera: Camera2D,
    pub held: Option<Cid>,
    pub ripples: Vec<Ripple>
}
impl Settings {
    pub fn default() -> Settings { Self {
        radius: 12.0,
        edge: 4.0,

        min_dist: 128.0,
        max_vel: 4.0,
        force: 0.5,

        paused: false,
        reducing: false,
        reduction_distance: 24.0,
        
        camera: Camera2D {
            offset: Vector2::new(500.0, 350.0),
            target: Vector2::zero(),
            rotation: 0.0,
            zoom: 1.0,
        },
        held: None,
        ripples: Vec::new()
    }}
}
