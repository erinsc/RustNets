use std::collections::HashMap;

use crate::{Sid, Pid};

#[macro_export]
macro_rules! tree {
    ($sym:expr $(, $arg:tt)*) => {{
        let node = $crate::BuildRuleNode::new($sym);
        $( let node = node.add($crate::tree!(@arg $arg)); )*
        node
    }};
    (@arg [$($inner:tt)*]) => { $crate::tree!($($inner)*) };
    (@arg $var:expr) => { $var };
}

#[macro_export]
macro_rules! rule {
    ([$($left:tt)*], [$($right:tt)*]) => {
        $crate::Rule::new(
            $crate::tree!($($left)*),
            $crate::tree!($($right)*),
        )
    };
}

pub enum Arg {
    Port(Pid),
    Node(BuildRuleNode)
}

impl From<Pid> for Arg { 
    fn from(v: Pid) -> Self { Arg::Port(v) }
}
impl From<BuildRuleNode> for Arg { 
    fn from(t: BuildRuleNode) -> Self { Arg::Node(t) }
}

pub struct BuildRuleNode {
    symbol: Sid,
    args: Vec<Arg>
}

impl BuildRuleNode {
    pub fn new(symbol: Sid) -> Self { BuildRuleNode {
        symbol, args: Vec::new()
    }}
    pub fn add(mut self, arg: impl Into<Arg>) -> Self {
        self.args.push(arg.into());
        self
    }
}

#[derive(Debug)]
pub enum Action {
    AddLeftNode(Sid),
    AddRightNode(Sid),
    LinkStart(),
    LinkEnd(Pid)
}
struct RuleBuilder {
    actions: Vec<Action>,
    next_pid: Pid,
    ports: Vec<Option<Pid>>
}
impl RuleBuilder {
    pub fn new() -> RuleBuilder { Self {
        actions: Vec::new(),
        next_pid: 0,
        ports: Vec::new()
    }}
    fn visit(&mut self, arg: Arg, side: fn(Sid) -> Action) {
        match arg {
            Arg::Port(pid) => {
                let id = pid as usize;

                if self.ports.len() < id + 1 {
                    self.ports.resize(id + 1, None);
                }
                if let Some(pid) = self.ports[id] {
                    self.actions.push(Action::LinkEnd(pid));
                } else {
                    self.actions.push(Action::LinkStart());
                    self.ports[id] = Some(self.next_pid);
                    self.next_pid += 1;
                }
            }
            Arg::Node(term) => {
                self.actions.push(side(term.symbol));

                for arg in term.args {
                    self.visit(arg, side);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Rule {
    pub left: Sid,
    pub right: Sid,
    pub actions: Vec<Action>
}
impl Rule {
    pub fn new(mut left: BuildRuleNode, mut right: BuildRuleNode) -> Rule {
        if left.symbol > right.symbol {
            (left, right) = (right, left);
        }

        let mut builder = RuleBuilder::new();

        for arg in left.args {
            builder.visit(arg, Action::AddRightNode);
        }
        for arg in right.args {
            builder.visit(arg, Action::AddLeftNode);
        }

        Rule { 
            left: left.symbol,
            right: right.symbol,
            actions: builder.actions
        }
    }
}
pub struct RuleBook {
    rules: HashMap<(Sid, Sid), Rule>
}
impl RuleBook {
    pub fn new() -> RuleBook { Self {
        rules: HashMap::new()
    }}
    
    pub fn register_rule(&mut self, rule: Rule) {
        self.rules.insert((rule.left, rule.right), rule);
    }
    pub fn get_rule(&self, left: Sid, right: Sid) -> Option<&Rule> {
        self.rules.get(&(left, right))
    }
}
