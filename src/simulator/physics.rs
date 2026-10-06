use std::collections::HashMap;

use raylib::prelude::Vector2;
use crate::{
    interactionnet::{InteractionNet, Nid, Pid, Port}, simulator::{CUTOFF, MINDIST, NodeData, REPULSION, SymbolData, utils::port_offset}
};

fn pos_to_cell(v: Vector2) -> (i32, i32) {
    ((v.x / CUTOFF).floor() as i32, (v.y / CUTOFF).floor() as i32)
}

fn cross_product(left: Vector2, right: Vector2) -> f32 {
    left.x * right.y - left.y * right.x
}

pub fn step(net: &mut InteractionNet<NodeData, SymbolData>, _dt: f32) {
    let n = net.count();
    let mut forces = vec![Vector2::zero(); n];
    let mut torques = vec![0.0; n];

    let mut grid: HashMap<(i32, i32), Vec<Nid>> = HashMap::with_capacity(n);
    for (i, node) in net.iter_nodes() {
        grid.entry(pos_to_cell(node.data.pos)).or_default().push(i);
    }

    for (id, node) in net.iter_nodes() {
        let left = node.data;

        let (cx, cy) = pos_to_cell(node.data.pos);
        for dx in -1..=1 {
            for dy in -1..=1 {
                let Some(list) = grid.get(&(cx + dx, cy + dy)) else { continue };
                for jd in list.clone() {
                    if jd <= id { continue };
                    let right = net.get_node_unchecked(jd).data;
                    
                    let diff = left.pos - right.pos;
                    let len = diff.length();
                    
                    if len > CUTOFF {
                        continue;
                    }
                    
                    let dir = if len > 1e-4 {
                        diff * (1.0/len)
                    } else { 
                        Vector2::one()
                    };
                    
                    let f = REPULSION * (1.0 - len / MINDIST);

                    println!("- {}", f);

                    forces[id as usize] += dir * f;
                    forces[jd as usize] -= dir * f;
                }
            }
        }
    }
    for (nid, left_node) in net.iter_nodes() {
        let lp_count = left_node.ports.len();
        for (pid, right_port) in left_node.active_ports() {
            if right_port.node >= nid { continue; }

            let left_port = Port::new(nid, pid as Pid);
            let right_node = net.get_node_unchecked(right_port.node);

            let left = port_offset(left_port.port, lp_count as Pid, left_node.data.angle);
            let right = port_offset(right_port.port, right_node.ports.len() as Pid, right_node.data.angle);
        
            let diff = (left_node.data.pos + left) - (right_node.data.pos + right);
            let len = diff.length();
                    
            if len < 1e-4 {
                continue;
            }
                    
            let dir = diff * (1.0/len);
            let f = REPULSION / 8.0;

            println!("+ {}", f);

            forces[left_port.node as usize] -= dir * f;
            forces[right_port.node as usize] += dir * f;

            let left_t = cross_product(left, dir * f);
            let right_t = cross_product(right, dir * f);

            torques[left_port.node as usize] -= left_t;
            torques[right_port.node as usize] += right_t;
        }
    }
    for (id, node) in net.iter_mut_nodes() {
        let f = forces[id as usize];
        node.data.vel *= 0.99;
        node.data.vel += f;
        node.data.pos += node.data.vel;
        
        node.data.angle_velocity *= 0.9;
        node.data.angle_velocity += torques[id as usize];
        node.data.angle += node.data.angle_velocity;
    }
}
