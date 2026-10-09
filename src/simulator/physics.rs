use std::collections::HashMap;

use raylib::prelude::Vector2;
use crate::{
    interactionnet::{InteractionNet, NetError, Nid, RuleBook},
    simulator::{Settings, NodeData, Ripple, SymbolData, link_offsets_unchecked}
};

fn pos_to_cell(v: Vector2, s: &Settings) -> (i32, i32) {
    ((v.x / s.min_dist).floor() as i32, (v.y / s.min_dist).floor() as i32)
}

fn cross_product(left: Vector2, right: Vector2) -> f32 {
    left.x * right.y - left.y * right.x
}
pub fn interact(net: &mut InteractionNet<NodeData, SymbolData>, rulebook: &RuleBook, s: &mut Settings) -> Result<(), NetError> {
    if !s.reducing {
        return Ok(());
    }

    let pairs: Vec<_> = net.get_principal_pairs().cloned().collect();
    
    for (left, right) in pairs {
        let left_node = net.get_node_unchecked(left);
        let right_node = net.get_node_unchecked(right);

        let len = (left_node.data.pos - right_node.data.pos).length();
        
        if len < s.min_reducting_dist {
            let vel = (left_node.data.vel + right_node.data.vel) * 0.5;
            let pos = (left_node.data.pos + right_node.data.pos) * 0.5;
            let ripple = Ripple::new(pos, vel);
            s.ripples.push(ripple);

            match net.interact(left, right, rulebook) {
                Ok(()) => Ok(()),
                Err(NetError::RuleMissing) => Ok(()),
                Err(e) => Err(e)
            }?;
        }
    }
    Ok(())
}

pub fn step(net: &mut InteractionNet<NodeData, SymbolData>, s: &mut Settings, _dt: f32) {
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
    }
    let density = if s.paused {0.9} else {0.99};

    for (id, node) in net.iter_mut_nodes() {
        node.data.vel *= density;
        node.data.vel += *forces.get(&id).unwrap_or(&Vector2::zero());
        node.data.vel = node.data.vel.clamp_value(0.0, s.max_vel);
        node.data.pos += node.data.vel;
            
        node.data.angle_velocity *= 0.9;
        node.data.angle_velocity += *torques.get(&id).unwrap_or(&0.0);
        node.data.angle += node.data.angle_velocity;
    }
    for ripple in s.ripples.iter_mut() {
        ripple.age += 0.01;
        ripple.pos += ripple.vel;
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
                    
                    if len > s.min_dist {
                        continue;
                    }
                    let f = s.force * (1.0 - len / s.min_dist);
                    
                    let dir = if len > 1e-4 {
                        diff * (1.0/len)
                    } else { 
                        Vector2::one()
                    };

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
    for (left, right) in net.iter_links() {
        let [a, _, _, d] = link_offsets_unchecked(left, right, net, s);

        let diff = a - d;
        let len = diff.length();           
        if len < 1e-4 { continue; }
                    
        let dir = diff * (1.0/len);
        let mut f = s.force;

        if left.port == 0 && right.port == 0 && s.reducing {
            f /= 1.0;
        } else {
            f /= 4.0;
        }

        *forces.entry(left.node).or_default() -= dir * f;
        *forces.entry(right.node).or_default() += dir * f;

        let left_offset = net.get_node_unchecked(left.node).data.pos - a;
        let right_offset = net.get_node_unchecked(right.node).data.pos - d;

        let left_torque = cross_product(left_offset, dir * f) / 180.0;
        let right_torque = cross_product(right_offset, dir * f) / 180.0;

        *torques.entry(left.node).or_default() += left_torque;
        *torques.entry(right.node).or_default() -= right_torque;
    }
}