use crate::sparsevec::SparseVec;
use crate::{Nid, Sid, Port, Symbol, Node, NetError};
use crate::{Rule, RuleBook, Action};

#[derive(Debug)]
pub struct InteractionNet<N, S> {
    pub nodes: SparseVec<Node<N>>,
    pub symbols: Vec<Symbol<S>>
}
impl<N: Clone, S> InteractionNet<N, S> {
    pub fn new() -> InteractionNet<N, S> { Self {
        nodes: SparseVec::new(),
        symbols: Vec::new()
    }}
    pub fn register_symbol(&mut self, name: &str, arity: usize, data: S) -> Sid {
        let symbol = Symbol { data, label: name.to_owned(), arity };
        self.symbols.push(symbol);
        (self.symbols.len() - 1) as Sid
    }
    pub fn create_node(&mut self, symbol: Sid, data: N) -> Result<Nid, NetError> {
        let arity = self.symbols.get(symbol as usize).ok_or(NetError::SymbolMissing)?.arity;

        let ports = (0..arity+1).map(|_| None).collect();    
    
        let node = Node { data, symbol, ports };
        Ok(self.nodes.insert(node) as Nid)
    }
    pub fn remove_node(&mut self, node: Nid) -> Option<N> {
        self.nodes.remove(node as usize).map(|n| n.data)
    }
    pub fn get_node(&self, id: Nid) -> Option<&Node<N>> {
        self.nodes.get(id as usize)
    }
    pub fn get_node_mut(&mut self, id: Nid) -> Option<&mut Node<N>> {
        self.nodes.get_mut(id as usize)
    }
    pub fn follow_port(&self, port: Port) -> Option<Port> {
		self.get_node(port.node)?
			.ports
			.get(port.port as usize)?
            .clone()
	}
    pub fn try_follow_port(&self, port: Option<Port>) -> Option<Port> {
		port.and_then(|port| self.follow_port(port))
	}
    pub fn set_port(&mut self, key: Port, value: Option<Port>) -> Result<(), NetError> {
        let node = self.get_node_mut(key.node).ok_or(NetError::NodeMissing)?;
        let port = node.ports.get_mut(key.port as usize).ok_or(NetError::PortMissing)?;

        *port = value;
        Ok(())
    }
    pub fn try_set_port(&mut self, key: Option<Port>, value: Option<Port>) -> Result<(), NetError> {
        key.map_or(Ok(()), |key| self.set_port(key, value))
    }
    pub fn link(&mut self, left: Port, right: Port) -> Result<(), NetError> {
        self.set_port(left, Some(right))?;
        self.set_port(right, Some(left))
    }
    pub fn try_link(&mut self, left: Option<Port>, right: Option<Port>) -> Result<(), NetError> {
        self.try_set_port(left, right)?;
        self.try_set_port(right, left)
    }

    pub fn interact(&mut self, left: Nid, right: Nid, rulebook: &RuleBook) -> Result<(), NetError> {
        let left_symbol = self.get_node(left).ok_or(NetError::NodeMissing)?.symbol;
        let right_symbol = self.get_node(right).ok_or(NetError::NodeMissing)?.symbol;

        if let Some(rule) = rulebook.get_rule(left_symbol, right_symbol) {
            self.rule_interact(left, right, rule)
        } else if let Some(rule) = rulebook.get_rule(right_symbol, left_symbol) {
            self.rule_interact(right, left, rule)
        } else {
            Err(NetError::RuleMissing)
        }
    }
    fn rule_interact(&mut self, left: Nid, right: Nid, rule: &Rule) -> Result<(), NetError> {
		let mut hard_pairs = Vec::new();

		let left_node = self.get_node(left).unwrap();
		let right_node = self.get_node(right).unwrap();

        let left_data = left_node.data.clone();
        let right_data = right_node.data.clone();

		let mut stack: Vec<_> = right_node.ports.iter().skip(1)
			.chain(left_node.ports.iter().skip(1)).copied().collect();

        //println!("Indirect: {:?}", stack);

		for port in stack.clone() {
			if let Some(port) = port && (port.node == left || port.node == right) {
                let other = self.follow_port(port).ok_or(NetError::LinkBroken)?;
                if port < other {
                    hard_pairs.push((port, other));
                }
            }
		}

        //println!("HardPairs: {:?}", hard_pairs);
        //println!("Stack: {:?}", stack);

        let mut lookup = Vec::new();
        for action in &rule.actions {
            let port = stack.pop().ok_or(NetError::RuleMalformed)?;
            
            //println!("STEP port: {:?}, action {:?}", port, action);

            match action {
                Action::AddLeftNode(sid) => {
                    let nid = self.create_node(*sid, left_data.clone())?;
                    self.add_new_node_to_stack(*sid, nid, port, &mut stack)?;
                }
                Action::AddRightNode(sid) => {
                    let nid = self.create_node(*sid, right_data.clone())?;
                    self.add_new_node_to_stack(*sid, nid, port, &mut stack)?;
                }
                Action::LinkStart() => {
                    lookup.push(port);
                }
                Action::LinkEnd(pid) => {
                    let prev = lookup.get(*pid as usize).ok_or(NetError::RuleMalformed)?.clone();
                    self.try_link(prev, port)?;
                }
            }
        }

        for (left, right) in hard_pairs {
            let left = self.follow_port(left);
            let right = self.follow_port(right);

            self.try_link(left, right)?;
        }

        self.remove_node(left);
        self.remove_node(right);

        Ok(())
    }
    fn add_new_node_to_stack(&mut self, sid: Sid, nid: Nid, port: Option<Port>, stack: &mut Vec<Option<Port>>) -> Result<(), NetError> {
        if let Some(port) = port {
            self.link(Port { node: nid, port: 0}, port)?;
        }
        let arity = self.symbols[sid as usize].arity;
        let ports = (0..arity)
            .map(|p| Some(Port { node: nid, port: p as u8 + 1}));
        stack.extend(ports);
        Ok(())
    }
}

impl<N, S> std::fmt::Display for InteractionNet<N, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "InteractionNet Count: {}", self.nodes.count())?;
        for (i, node) in self.nodes.iter() {
            let label = &self.symbols[node.symbol as usize].label;
            write!(f, "\nId: {: >3} Symbol: {: <6} Ports:", i, label)?;
            for port in &node.ports {
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

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn peano_rules() {
        let mut net = InteractionNet::<(), ()>::new();
        let add = net.register_symbol("Add", 2, ());
        let z = net.register_symbol("Z", 0, ());
        let s = net.register_symbol("S", 1, ());

        let mut book = RuleBook::new();

        let peano_zero = rule![
            [add, 0, 0],
            [z]
        ];
        let peano_add = rule![
            [add, [s, 0], 1], 
            [s, [add, 0, 1]]
        ];
        book.register_rule(peano_zero);
        book.register_rule(peano_add);
        
        let an = net.create_node(add, ()).unwrap();
        let sn = net.create_node(s, ()).unwrap();
        let zn = net.create_node(z, ()).unwrap();
        let zn2 = net.create_node(z, ()).unwrap();

        net.link(port![an, 0], port![sn, 0]).unwrap();
        net.link(port![sn, 1], port![zn, 0]).unwrap();
        net.link(port![an, 2], port![zn2, 0]).unwrap();

        for (_, node) in net.nodes.iter() {
            for port in node.ports.clone() {
                assert_eq!(port, net.try_follow_port(net.try_follow_port(port)));
            }
        }

        net.interact(an, sn, &book).unwrap();
        net.interact(2, 5, &book).unwrap();

        println!("{}", net);

        for (_, node) in net.nodes.iter() {
            for port in node.ports.clone() {
                assert_eq!(port, net.try_follow_port(net.try_follow_port(port)));
            }
        }
        println!("{}", net);
    }
    #[test]
    fn universal_rules() {
        let mut net = InteractionNet::<(), ()>::new();
        let gamma = net.register_symbol("Gamma", 2, ());
        let delta = net.register_symbol("Delta", 2, ());
        let epsilon = net.register_symbol("Epsiln", 0, ());

        let mut book = RuleBook::new();

        book.register_rule(rule![
            [epsilon],
            [epsilon]
        ]);
        book.register_rule(rule![
            [epsilon], 
            [gamma, [epsilon], [epsilon]]
        ]);
        book.register_rule(rule![
            [epsilon], 
            [delta, [epsilon], [epsilon]]
        ]);
        book.register_rule(rule![
            [gamma, 0, 1], 
            [gamma, 1, 0]
        ]);
        book.register_rule(rule![
            [delta, 0, 1], 
            [delta, 0, 1]
        ]);
        book.register_rule(rule![
            [gamma, [delta, 0, 1], [delta, 2, 3]], 
            [delta, [gamma, 0, 2], [gamma, 1, 3]]
        ]);

        net.create_node(epsilon, ()).unwrap();
        net.create_node(epsilon, ()).unwrap();
        net.create_node(delta, ()).unwrap();
        net.create_node(gamma, ()).unwrap();

        net.link(port![0, 0], port![2, 2]).unwrap();
        net.link(port![1, 0], port![3, 1]).unwrap();
        net.link(port![2, 1], port![3, 2]).unwrap();
        net.link(port![2, 0], port![3, 0]).unwrap();

        println!("{}", net);

        net.interact(2, 3, &book).unwrap();

        println!("{}", net);

        net.interact(0, 6, &book).unwrap();
        net.interact(1, 5, &book).unwrap();
        net.interact(3, 6, &book).unwrap();

        println!("{}", net);
    }
}