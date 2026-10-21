use std::collections::{HashMap, HashSet};

use crate::interactionnet::{Action, Rulebook, rules::Symbol};

pub type Cid = usize;
pub type Sid = u16;
pub type Pid = u8;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Port {
    pub cell_id: Cid,
    pub port_id: Pid
}
impl Port {
    pub fn new(cell_id: Cid, port_id: Pid) -> Port { Port {
        cell_id, port_id
    }}
}
impl Ord for Port {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cell_id.cmp(&other.cell_id).then(self.port_id.cmp(&other.port_id))
    }
}
impl PartialOrd for Port {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for Port {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{: ^3} {: ^2}]", self.cell_id, self.port_id)
    }
}

#[derive(Debug)]
pub struct Cell<T> {
    pub symbol_id: Sid,
    pub ports: Vec<Option<Port>>,
    pub data: T
}

#[derive(Debug)]
pub enum NetError {
    RuleMissing(Sid, Sid),
    CellMissing(Cid),
    SymbolMissing(Sid),
    PortMissing(Port),
    LinkBroken(Port)
}

#[derive(Debug)]
pub struct NetStore<T> {
    cells: HashMap<Cid, Cell<T>>,
    next_free_cell: Cid,
    main_links: HashSet<(Cid, Cid)>
}
impl<T> NetStore<T> {
    pub fn new() -> NetStore<T> { NetStore {
        cells: HashMap::new(),
        next_free_cell: 0,
        main_links: HashSet::new()
    }}

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn insert(&mut self, symbol_id: Sid, arity: usize, data: T) -> Cid {
        let ports = (0..arity+1).map(|_| None).collect();    
    
        let cell = Cell { data, symbol_id, ports };
        self.cells.insert(self.next_free_cell, cell);
        self.next_free_cell += 1;

        self.next_free_cell - 1
    }

    pub fn remove(&mut self, cell_id: Cid) -> Option<Cell<T>> {
        self.cells.remove(&cell_id)
    }

    pub fn get(&self, cell_id: Cid) -> Option<&Cell<T>> {
        self.cells.get(&cell_id)
    }

    pub fn get_mut(&mut self, cell_id: Cid) -> Option<&mut Cell<T>> {
        self.cells.get_mut(&cell_id)
    }

    pub fn get_disjoint_mut<const N: usize>(&mut self, cell_ids: [&Cid; N]) -> [Option<&mut Cell<T>>; N] {
        self.cells.get_disjoint_mut(cell_ids)
    }

    pub fn iter(&self) -> impl Iterator<Item = (Cid, &Cell<T>)> {
        self.cells.iter().map(|(&id, cell)| (id, cell))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Cid, &mut Cell<T>)> {
        self.cells.iter_mut().map(|(&id, cell)| (id, cell))
    }

    pub fn follow(&self, key: Port) -> Option<Port> {
        self.cells.get(&key.cell_id)?
            .ports.get(key.port_id as usize)
            .copied()
            .flatten()
    }
    fn set_port(&mut self, key: Port, value: Option<Port>) -> Result<(), NetError> {
        *self.get_mut(key.cell_id)
            .ok_or(NetError::CellMissing(key.cell_id))?
            .ports.get_mut(key.port_id as usize)
            .ok_or(NetError::PortMissing(key))? = value;
        Ok(())
    }
    pub fn link(&mut self, left: Option<Port>, right: Option<Port>) -> Result<(), NetError> {
        println!("{left:?} {right:?}");

        if let Some(left) = left {
            self.set_port(left, right)?;
        }
        if let Some(right) = right {
            self.set_port(right, left)?;
        }
        if let (Some(left), Some(right)) = (left, right) {
            if left.port_id == 0 && right.port_id == 0 {
                if left < right {
                    self.main_links.insert((left.cell_id, right.cell_id));
                } else {
                    self.main_links.insert((right.cell_id, left.cell_id));
                }
            }
        }
        Ok(())
    }
}



impl<C, S> std::fmt::Display for Net<C, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "InteractionNet Count: {}", self.net.cells.len())?;
        for (i, cell) in self.net.cells.iter() {
            let label = &self.rulebook.get_symbol(cell.symbol_id)
                .ok_or(std::fmt::Error)?.label;
            write!(f, "\nId: {: >3} Symbol: {: <6} Ports:", i, label)?;
            for port in &cell.ports {
                if let Some(port) = port {
                    write!(f, " {}", port)?;
                } else {
                    write!(f, " [ ---- ]")?;
                }
            }
        }
        Ok(())     
    }
}

pub struct Net<C, S> {
    net: NetStore<C>,
    rulebook: Rulebook<S>
}
impl<C: Clone, S> Net<C, S> {
    pub fn new(rulebook: Rulebook<S>) -> Net<C, S> { Net {
        net: NetStore::new(),
        rulebook
    }}
    pub fn len(&self) -> usize {
        self.net.len()
    }

    pub fn insert(&mut self, symbol_id: Sid, data: C) -> Result<Cid, NetError> {
        let arity = self.rulebook.get_symbol(symbol_id)
            .ok_or(NetError::SymbolMissing(symbol_id))?.arity;
        Ok(self.net.insert(symbol_id, arity, data))
    }

    pub fn get(&self, cell_id: Cid) -> Option<&Cell<C>> {
        self.net.get(cell_id)
    }

    pub fn get_mut(&mut self, cell_id: Cid) -> Option<&mut Cell<C>> {
        self.net.get_mut(cell_id)
    }

    pub fn get_disjoint_mut<const N: usize>(&mut self, cell_ids: [&Cid; N]) -> [Option<&mut Cell<C>>; N] {
        self.net.get_disjoint_mut(cell_ids)
    }

    pub fn get_symbol(&self, symbol_id: Sid) -> Option<&Symbol<S>> {
        self.rulebook.get_symbol(symbol_id)
    }

    pub fn link(&mut self, left: Port, right: Port) -> Result<(), NetError> {
        self.net.link(Some(left), Some(right))
    }

    pub fn iter(&self) -> impl Iterator<Item = (Cid, &Cell<C>)> {
        self.net.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Cid, &mut Cell<C>)> {
        self.net.iter_mut()
    }
    pub fn iter_main_links(&self) -> impl Iterator<Item = &(Cid, Cid)> {
        self.net.main_links.iter()
    }

    pub fn iter_links(&self) -> impl Iterator<Item = (Port, Port)> {
        self.iter()
            .flat_map(|(left_cell_id, left_cell)|
            left_cell.ports.iter().enumerate().filter_map(move |(left_port_id, left_port)| {
                let right_port = Port::new(left_cell_id, left_port_id as Pid);
                left_port.filter(|&p| p < right_port).map(|p| (p, right_port))
            })
        )
    }
    pub fn follow(&self, key: Port) -> Option<Port> {
        self.net.follow(key)
    }
    pub fn try_follow(&self, key: Option<Port>) -> Option<Port> {
        self.follow(key?)
    }

    pub fn interact(&mut self, left_id: Cid, right_id: Cid) -> Result<(), NetError> {
        let ls_id = self.net.get(left_id)
            .ok_or(NetError::CellMissing(left_id))?.symbol_id;
        let rs_id = self.net.get(right_id)
            .ok_or(NetError::CellMissing(right_id))?.symbol_id;

        let rule = self.rulebook.get(ls_id, rs_id)
            .ok_or(NetError::RuleMissing(ls_id, rs_id))?;

        let mut hard_pairs = Vec::new();

		let left_cell = self.net.get(left_id).unwrap();
		let right_cell = self.net.get(right_id).unwrap();

        let left_data = left_cell.data.clone();
        let right_data = right_cell.data.clone();

        let mut stack = Vec::new();
        
        // The right cell stores the left port, vice-versa
        // Assumes the link is valid, which lets us create the other port in the link without following it
        // Is it actually more efficient ? who knows
		for (rpid, lport) in right_cell.ports.iter().enumerate().skip(1).rev() {
            stack.push(*lport);

            if let Some(lport) = lport && (lport.cell_id == left_id || lport.cell_id == right_id) {
                let rport = Port::new(right_id, rpid as Pid);
                if *lport < rport {
                    hard_pairs.push((*lport, rport));
                }
            }
		}

        for (lpid, rport) in left_cell.ports.iter().enumerate().skip(1).rev() {
            stack.push(*rport);

            if let Some(rport) = rport && (rport.cell_id == left_id || rport.cell_id == right_id) {
                let lport = Port::new(left_id, lpid as Pid);
                if lport < *rport {
                    hard_pairs.push((lport, *rport));
                }
            }
		}

        let mut lookup = Vec::new();
        for action in rule.actions.clone() {
            let port = stack.pop().unwrap();

            match action {
                Action::AddCell(sid, left) => {
                    let data = if left { left_data.clone() } else { right_data.clone() };
                    let arity = self.rulebook.get_symbol(sid).unwrap().arity;

                    let cid = self.net.insert(sid, arity, data);
                    if port.is_some() {
                        let new = Port::new(cid, 0);
                        self.net.link(Some(new), port)?;
                    }
                    let ports = (0..arity)
                        .map(|p| Some(Port::new(cid, p as u8 + 1))).rev();
                    stack.extend(ports);
                }
                Action::LinkStart() => {
                    lookup.push(port);
                }
                Action::LinkEnd(pid) => {
                    let prev = lookup.get(pid as usize).unwrap().clone();
                    self.net.link(prev, port)?;
                }
            }
        }

        for (left, right) in hard_pairs {
            let left_port = self.net.follow(left);
            let right_port = self.net.follow(right);

            self.net.link(left_port, right_port)?;
        }

        self.net.remove(left_id);
        self.net.remove(right_id);
        if left_id < right_id {
            self.net.main_links.remove(&(left_id, right_id));
        } else {
            self.net.main_links.remove(&(right_id, left_id));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{interactionnet::{Net, Port, Rulebook}, *};

    fn check_port_validity<C: Clone, S>(net: &Net<C, S>) {
        for (_, cell) in net.iter() {
            for port in cell.ports.clone() {
                assert_eq!(port, net.try_follow(net.try_follow(port)));
            }
        }
    }

    #[test]
    fn peano_rules() {
        let mut book = Rulebook::new("Peano", "Simple Addition");
        let add = book.add_symbol("Add", 2, ());
        let z = book.add_symbol("Z", 0, ());
        let s = book.add_symbol("S", 1, ());

        book.add(rule![add, 0, 0],
                 rule![z]).unwrap();
        
        
        book.add(rule![add, [s, 0], 1], 
                 rule![s, [add, 0, 1]]).unwrap();

        let mut net = Net::new(book);
        
        let an = net.insert(add, ()).unwrap();
        let sn = net.insert(s, ()).unwrap();
        let zn = net.insert(z, ()).unwrap();
        let zn2 = net.insert(z, ()).unwrap();

        net.link(Port::new(an, 0), Port::new(sn, 0)).unwrap();
        net.link(Port::new(sn, 1), Port::new(zn, 0)).unwrap();
        net.link(Port::new(an, 2), Port::new(zn2, 0)).unwrap();

        
        println!("{:?}", rule![add, 0, 0]);
        println!("{:?}", rule![z]);
        println!("{net}");

        check_port_validity(&net);

        net.interact(an, sn).unwrap();

        println!("{net}");
        net.interact(2, 5).unwrap();

        check_port_validity(&net);
    }
    #[test]
    fn universal_rules() {
        let mut book = Rulebook::new("Lafont", "lafonts universal rules");
        let gamma = book.add_symbol("Gamma", 2, ());
        let delta = book.add_symbol("Delta", 2, ());
        let epsilon = book.add_symbol("Epsiln", 0, ());

        book.add(rule![epsilon],
                 rule![epsilon]).unwrap();
        book.add(rule![epsilon], 
                 rule![gamma, [epsilon], [epsilon]]).unwrap();
        book.add(rule![epsilon], 
                 rule![delta, [epsilon], [epsilon]]).unwrap();
        book.add(rule![gamma, 0, 1], 
                 rule![gamma, 1, 0]).unwrap();
        book.add(rule![delta, 0, 1], 
                 rule![delta, 0, 1]).unwrap();
        book.add(rule![gamma, [delta, 0, 1], [delta, 2, 3]], 
                 rule![delta, [gamma, 0, 2], [gamma, 1, 3]]).unwrap();

        let mut net = Net::new(book);

        net.insert(epsilon, ()).unwrap();
        net.insert(epsilon, ()).unwrap();
        net.insert(delta, ()).unwrap();
        net.insert(gamma, ()).unwrap();

        net.link(Port::new(0, 0), Port::new(2, 2)).unwrap();
        net.link(Port::new(1, 0), Port::new(3, 1)).unwrap();
        net.link(Port::new(2, 1), Port::new(3, 2)).unwrap();
        net.link(Port::new(2, 0), Port::new(3, 0)).unwrap();

        net.interact(2, 3).unwrap();
        
        check_port_validity(&net);

        net.interact(1, 6).unwrap();
        net.interact(4, 7).unwrap();
        net.interact(5, 0).unwrap();

        println!("{net}");

        check_port_validity(&net);

    }
}