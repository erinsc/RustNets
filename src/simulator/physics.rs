use std::collections::HashMap;

use raylib::prelude::Vector2;
use crate::{
    interactionnet::{InteractionNet, NetError, Nid, Pid, Port, RuleBook}, simulator::{NodeData, Settings, SymbolData, settings::port_offset}
};

fn pos_to_cell(v: Vector2, s: &Settings) -> (i32, i32) {
    ((v.x / s.cutoff).floor() as i32, (v.y / s.cutoff).floor() as i32)
}

fn cross_product(left: Vector2, right: Vector2) -> f32 {
    left.x * right.y - left.y * right.x
}
pub fn interact(net: &mut InteractionNet<NodeData, SymbolData>, rulebook: &RuleBook, s: &Settings) -> Result<(), NetError> {
    if !s.reducing {
        return Ok(());
    }

    let pairs: Vec<_> = net.get_principal_pairs().cloned().collect();
    
    for (left, right) in pairs {
        let left_node = net.get_node_unchecked(left);
        let right_node = net.get_node_unchecked(right);

        let len = (left_node.data.pos - right_node.data.pos).length();
        
        println!("{} < {}", len, s.radius * 2.0);

        if len < s.radius * 2.0 {
            net.interact(left, right, rulebook)?;
        }
    }
    Ok(())
}

pub fn step(net: &mut InteractionNet<NodeData, SymbolData>, s: &Settings, _dt: f32) {
    let n = net.count();
    let mut forces: HashMap<Nid, Vector2> = HashMap::with_capacity(n);
    let mut torques: HashMap<Nid, f32> = HashMap::with_capacity(n);
    let mut grid: HashMap<(i32, i32), Vec<Nid>> = HashMap::with_capacity(n);
    
    for (i, node) in net.iter_nodes() {
        grid.entry(pos_to_cell(node.data.pos, s)).or_default().push(i);
    }
    if !s.paused {
        calculate_node_forces(net, s, &grid, &mut forces);
        calculate_edge_forces(net, s, &mut forces, &mut torques);

        for (id, node) in net.iter_mut_nodes() {
            let f = *forces.get(&id).unwrap_or(&Vector2::zero());
            node.data.vel *= 0.99;
            node.data.vel += f;
            node.data.pos += node.data.vel;
            
            node.data.angle_velocity *= 0.9;
            node.data.angle_velocity += torques.get(&id).unwrap_or(&0.0); 
            node.data.angle += node.data.angle_velocity;
        }
    } else {
        for (_, node) in net.iter_mut_nodes() {
            node.data.vel *= 0.9;
            node.data.pos += node.data.vel;
            
            node.data.angle_velocity *= 0.9;
            node.data.angle += node.data.angle_velocity;
        }
    }
}

fn calculate_node_forces(
    net: &InteractionNet<NodeData, SymbolData>,
    s: &Settings,
    grid: &HashMap<(i32, i32), Vec<Nid>>,
    forces: &mut HashMap<Nid, Vector2>
) {
    for (id, node) in net.iter_nodes() {
        let left = node.data;

        let (cx, cy) = pos_to_cell(node.data.pos, s);
        for dx in -1..=1 {
            for dy in -1..=1 {
                let Some(list) = grid.get(&(cx + dx, cy + dy)) else { continue };
                for jd in list.clone() {
                    if jd <= id { continue };
                    let right = net.get_node_unchecked(jd).data;
                    
                    let diff = left.pos - right.pos;
                    let len = diff.length();
                    
                    if len > s.cutoff {
                        continue;
                    }
                    
                    let dir = if len > 1e-4 {
                        diff * (1.0/len)
                    } else { 
                        Vector2::one()
                    };
                    
                    let f = s.force * (1.0 - len / s.min_dist);

                    *forces.entry(id).or_default() += dir * f;
                    *forces.entry(jd).or_default() -= dir * f;
                }
            }
        }
    }
}

fn calculate_edge_forces(
    net: &InteractionNet<NodeData, SymbolData>,
    s: &Settings,
    forces: &mut HashMap<Nid, Vector2>,
    torques: &mut HashMap<Nid, f32>
) {
    for (nid, left_node) in net.iter_nodes() {
        let lp_count = left_node.ports.len();
        for (pid, right_port) in left_node.active_ports() {
            if right_port.node >= nid { continue; }

            let left_port = Port::new(nid, pid as Pid);
            let right_node = net.get_node_unchecked(right_port.node);

            let left = port_offset(
                left_port.port,
                lp_count as Pid,
                left_node.data.angle,
                s
            );
            let right = port_offset(
                right_port.port,
                right_node.ports.len() as Pid,
                right_node.data.angle,
                s
            );
        
            let diff = (left_node.data.pos + left) - (right_node.data.pos + right);
            let len = diff.length();
                    
            if len < 1e-4 {
                continue;
            }
                    
            let dir = diff * (1.0/len);
            let mut f = s.force / 8.0;

            if left_port.port == 0 && right_port.port == 0 && s.reducing {
                f *= 4.0;
            }

            *forces.entry(left_port.node).or_default() -= dir * f;
            *forces.entry(right_port.node).or_default() += dir * f;

            let left_t = cross_product(left, dir * f);
            let right_t = cross_product(right, dir * f);

            *torques.entry(left_port.node).or_default() -= left_t;
            *torques.entry(right_port.node).or_default() += right_t;
        }
    }
}