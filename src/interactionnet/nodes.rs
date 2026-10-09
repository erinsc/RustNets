pub type Nid = u32;
pub type Sid = u16;
pub type Pid = u8;

#[derive(Clone, Copy, Debug)]
pub struct Port {
    pub node: Nid,
    pub port: Pid
}
impl Port {
    pub fn new(node: Nid, port: Pid) -> Port { Self {
        node, port 
    }}
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

impl std::fmt::Display for Port {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{: ^3} {: ^2}]", self.node, self.port)
    }
}

#[macro_export]
macro_rules! port {
    ($node:expr, $port:expr) => {
        $crate::Port::new($node, $port)
    };
}

#[derive(Debug)]
pub struct Symbol<T> {
    pub data: T,
    pub label: String,
    pub arity: usize // Not counting the principal port
}

#[derive(Debug)]
pub struct Node<T> {
    pub data: T,
    pub symbol: Sid,
    pub ports: Vec<Option<Port>>
}

#[derive(Debug)]
pub enum NetError {
    RuleMissing,
    RuleMalformed,
    NodeMissing,
    SymbolMissing,
    PortMissing,
    LinkBroken
}