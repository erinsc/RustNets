use std::collections::HashMap;

use crate::SparseVec;

type Nid = u32;
type Sid = u16;
type Pid = u8;

pub enum NetError {
    RuleMissing,
    RuleMalformed,
    NodeMissing,
    SymbolMissing,
    PortMissing
}

#[derive(Clone, Copy, Debug)]
pub struct Port {
    node: Nid,
    port: Pid
}

impl Ord for Port {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.node.cmp(&other.node).then(self.port.cmp(&other.port))
    }
}
impl PartialOrd for Port {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Port {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node && self.port == other.port
    }
}
impl Eq for Port {}


pub struct Symbol<T> {
    data: T,
    label: String,
    arity: usize // Not counting the principal port
}

pub struct Node<T> {
    data: T,
    symbol: Sid,
    ports: Vec<Option<Port>>
}

enum Action {
    AddLeftNode(Sid),
    AddRightNode(Sid),
    LinkStart(),
    LinkEnd(Pid)
}

struct Rule {
    actions: Vec<Action>
}

pub struct RuleBook {
    rules: HashMap<(Sid, Sid), Rule>
}
impl RuleBook {
    pub fn new() -> RuleBook { Self {
        rules: HashMap::new()
    }}
    // TODO: need to flip the rule around for when left > right. right now not doing that will produce a bug
    pub fn register_rule(&mut self, rule: Rule, left: Sid, right: Sid) {
        self.rules.insert((left, right), rule);
    }
    pub fn get_rule(&self, left: Sid, right: Sid) -> Option<&Rule> {
        self.rules.get(&(left, right))
    }
}

pub struct InteractionNet<N, S> {
    nodes: SparseVec<Node<N>>,
    symbols: Vec<Symbol<S>>,
    
}
impl<N: Clone, S> InteractionNet<N, S> {
    pub fn new() -> InteractionNet<N, S> { Self {
        nodes: SparseVec::new(),
        symbols: Vec::new()
    }}
    pub fn register_symbol(&mut self, name: &str, arity: usize, data: S) -> () {
        let symbol = Symbol { data, label: name.to_owned(), arity };
        self.symbols.push(symbol);
    }
    pub fn create_node(&mut self, symbol: Sid, data: N) -> Result<Nid, NetError> {
        let arity = self.symbols.get(symbol as usize).ok_or(NetError::SymbolMissing)?.arity;

        let ports = (0..arity).map(|_| None).collect();    
    
        let node = Node { data, symbol, ports };
        Ok(self.nodes.insert(node) as Nid)
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

    pub fn interact(&mut self, left: Nid, right: Nid, rulebook: RuleBook) -> Result<(), NetError> {
        let left_symbol = self.get_node(left).ok_or(NetError::NodeMissing)?.symbol;
        let right_symbol = self.get_node(right).ok_or(NetError::NodeMissing)?.symbol;

        let rule = rulebook.get_rule(left_symbol, right_symbol).ok_or(NetError::RuleMissing)?;

		let mut hard_pairs = Vec::new();
		let mut stack = Vec::new();

		let left_node = self.get_node(left).unwrap();
		let right_node = self.get_node(right).unwrap();

        let left_data = left_node.data.clone();
        let right_data = right_node.data.clone();

		let indirects = left_node.ports.iter().skip(1)
			.chain(right_node.ports.iter().skip(1)).copied();

		for indirect in indirects {
			let port = self.try_follow_port(indirect);

			if let (Some(port), Some(indirect)) = (port, indirect) {
				if (port.node == left || port.node == right) && port < indirect {
					hard_pairs.push((port, indirect));
				}
			}
			stack.push(port);
		}

        let mut lookup = Vec::new();
        for action in &rule.actions {
            let port = stack.pop().ok_or(NetError::RuleMalformed)?;
            
            match action {
                Action::AddLeftNode(sid) => {
                    let nid = self.create_node(*sid, left_data.clone())?;

                    if let Some(port) = port {
                        self.link(Port { node: nid, port: 0}, port)?;
                    }
                }
                Action::AddRightNode(sid) => {
                    let nid = self.create_node(*sid, right_data.clone())?;

                    if let Some(port) = port {
                        self.link(Port { node: nid, port: 0}, port)?;
                    }
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

        Ok(())
    }
}