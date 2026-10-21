use std::{collections::HashMap};

use raylib::{RaylibHandle, ffi::Vector2};

use crate::{interactionnet::{Cell, Cid, Net, NetError}, simulator::{Ripple, data::{CellData, Settings}, ports::port_offset}};

fn cross_product(left: Vector2, right: Vector2) -> f32 {
    left.x * right.y - left.y * right.x
}

struct CollisionGrid {
    grid: HashMap<(i32, i32), Vec<Cid>>,
}
impl CollisionGrid {
    fn new<S>(grid_size: f32, net: &Net<CellData, S>) -> CollisionGrid { 
        let mut grid = HashMap::<(i32, i32), Vec<Cid>>::new();

        net.iter().for_each(|(cell_id, cell)| {
            grid.entry(
                CollisionGrid::pos_to_grid(grid_size, cell.data.pos)
            ).or_default().push(cell_id);
        });
        CollisionGrid { grid: grid }
    }

    fn pos_to_grid(grid_size: f32, pos: Vector2) -> (i32, i32) {
        ((pos.x / grid_size).floor() as i32, (pos.y / grid_size).floor() as i32)
    }

    // Iterate through all pairs of registered cells that are within neigbouring cells, or the same cell
    // Each pair is only returned once, with no guarrantee of pair order, nor order within pairs
    fn iter(&self) -> impl Iterator<Item = (Cid, Cid)> {
        self.grid.iter().flat_map(move |(&(cx, cy), vec)| {

            // Cells from the same grid square
            // Only considers pairs where left_id < right_id

            let same = vec.iter().enumerate().flat_map(move |(i, &left)| {
                vec[i + 1..].iter().map(move |&right| (left, right))
            });

            // Cells from different squares
            // Only consides squares down and to the left

            let different = [(1, 0), (-1, 1), (0, 1), (1, 1)]
                .into_iter()
                .filter_map(move |(dx, dy)| self.grid.get(&(cx + dx, cy + dy)))
                .flat_map(move |other|
                    vec.iter().flat_map(move |&left|
                        other.iter().map(move |&right| (left, right))
                    )
                )
            ;

            same.chain(different)
        })
    }
}

pub fn interact_net<S>(net: &mut Net<CellData, S>, settings: &mut Settings) -> Result<(), NetError> {
    if !settings.reducing {
        return Ok(());
    }
    
    let main_links: Vec<_> = net.iter_main_links().copied().collect();

    for (left_id, right_id) in main_links {
        let [left, right] = net.get_disjoint_mut([&left_id, &right_id]);
        let (Some(left), Some(right)) = (left, right) else { continue; };

        let difference = left.data.pos - right.data.pos;
        let length = difference.length();
        if length > settings.reduction_distance { continue; }

        let velocity = (left.data.vel + right.data.vel) * 0.5; 
        let position = (left.data.pos + right.data.pos) * 0.5;

        let ripple = Ripple::new(position, velocity); 
        settings.ripples.push(ripple);

        let r = net.interact(left_id, right_id);
        match r {
            Ok(()) => Ok(()),
            Err(NetError::RuleMissing(_, _)) => Ok(()),
            Err(e) => Err(e)
        }?;
    }
    Ok(())
}

pub fn step_net<S>(net: &mut Net<CellData, S>, settings: &mut Settings) {
    let grid = CollisionGrid::new(settings.min_dist, net);
    
    // Push cells away from each other
    for (left_id, right_id) in grid.iter() {
        let [left, right] = net.get_disjoint_mut([&left_id, &right_id]);
        let (Some(left), Some(right)) = (left, right) else { continue; };

        let difference = left.data.pos - right.data.pos;
        let length = difference.length();            
        if length > settings.min_dist || length < 1e-4 { continue; }
        
        let direction = difference * (1.0 / length);
        let force = settings.force * (1.0 - length / settings.min_dist ) * 2.0;

        left.data.vel += direction * force;
        right.data.vel -= direction * force;
    }

    // Pull links together
    for (left_port, right_port) in net.iter_links().collect::<Vec<_>>() {
        let [left, right] = net.get_disjoint_mut([&left_port.cell_id, &right_port.cell_id]);
        let (Some(left), Some(right)) = (left, right) else { continue; };

        let left_offset = port_offset(left, left_port.port_id, settings);
        let right_offset = port_offset(right, right_port.port_id, settings);

        let difference = left.data.pos + left_offset - right_offset - right.data.pos;
        let length = difference.length();       
        if length < 1e-4 { continue; }
        
        let direction = difference * (1.0 / length);
        let mut force = settings.force;

        if left_port.port_id == 0 && right_port.port_id == 0 && settings.reducing {
            force *= 4.0;
        }

        let push = direction * force;

        left.data.vel -= push;
        right.data.vel += push;

        left.data.spin -= cross_product(left_offset, push) / 180.0;
        right.data.spin += cross_product(right_offset, push) / 180.0;
    }
    for (_, cell) in net.iter_mut() {
        cell.data.vel *= 0.95;
        //cell.data.vel = cell.data.vel.clamp_value(0.0, settings.max_vel);
        cell.data.pos += cell.data.vel;

        cell.data.spin *= 0.9;
        cell.data.angle += cell.data.spin;
    }
    for ripple in settings.ripples.iter_mut() {
        ripple.age += 0.01;
        ripple.pos += ripple.vel;
    }
}

pub fn jitter_net<S>(net: &mut Net<CellData, S>, rl: &RaylibHandle) {
    for (_, cell) in net.iter_mut() {
        let x: i32 = rl.get_random_value(-10..=10);
        let y: i32 = rl.get_random_value(-10..=10);
        let r = Vector2::new(x as f32 / 10.0, y as f32 / 10.0);

        cell.data.pos += r;
    }
}