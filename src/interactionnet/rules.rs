use std::collections::{HashMap};
use crate::interactionnet::{ Pid, Sid };

pub struct Symbol<T> {
    pub label: String,
    pub arity: usize,
    pub data: T
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    AddCell(Sid, bool),
    LinkStart(),
    LinkEnd(Pid)
}

pub struct Rule {
    pub left: Sid,
    pub right: Sid,
    pub actions: Vec<Action>
}
impl Rule {

}

#[derive(Debug)]
pub enum RuleNode {
    Port(Pid),
    Symbol(Sid, Vec<RuleNode>) 
}
impl RuleNode {
    fn new(symbol_id: Sid) -> RuleNode {
        RuleNode::Symbol(symbol_id, Vec::new())
    }
    fn port(self, port_id: Pid) -> RuleNode {
        self.node(RuleNode::Port(port_id))
    }
    fn node(mut self, node: RuleNode) -> RuleNode {
        match &mut self {
            RuleNode::Symbol(_, kids) => kids.push(node),
            RuleNode::Port(_) => unreachable!("Node::new only creates symbols"),
        }
        self
    }
}

struct RuleBuilder<'a, T> {
    symbols: &'a Vec<Symbol<T>>,
    ports: HashMap<Pid, Option<Pid>>,
    actions: Vec<Action>,
    stack_len: usize 
}
impl<'a, T> RuleBuilder<'a, T> {
    fn new(symbols: &'a Vec<Symbol<T>>) -> RuleBuilder<'a, T> { RuleBuilder {
        symbols, 
        ports: HashMap::new(), 
        actions: Vec::new(),
        stack_len: 0
    }}
    fn root(&mut self, sid: Sid) -> Result<(), RuleError> {
        let arity = self.symbols.get(sid as usize)
            .ok_or(RuleError::SymbolUnknown)?.arity;
        self.stack_len = arity;
        Ok(())
    }
    fn visit(&mut self, kids: Vec<RuleNode>, is_left: bool) -> Result<(), RuleError> {
        for node in kids {
            println!("{node:?}");

            if self.stack_len == 0 {
                return Err(RuleError::ArityViolated);
            }
            match node {
                RuleNode::Port(pid) => {
                    if let Some(time) = self.ports.get(&pid) {
                        if let Some(time) = time {
                            self.actions.push(Action::LinkEnd(*time));
                            self.stack_len -= 1;
                            self.ports.insert(pid, None);
                        } else {
                            return Err(RuleError::PortMismatch);
                        }
                    } else {
                        self.actions.push(Action::LinkStart());
                        self.stack_len -= 1;
                        self.ports.insert(pid, Some(self.ports.len() as Pid));
                    }
                }
                RuleNode::Symbol(sid, grandkids) => {
                    let arity = self.symbols.get(sid as usize)
                        .ok_or(RuleError::SymbolUnknown)?.arity;

                    self.stack_len += arity;
                    self.stack_len -= 1;

                    self.actions.push(Action::AddCell(sid, is_left));
                    self.visit(grandkids, is_left)?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum RuleError {
    ArityViolated,
    PortMismatch,
    SymbolUnknown
}

pub struct Rulebook<T> {
    name: String,
    description: String,
    symbols: Vec<Symbol<T>>,
    rules: HashMap<(Sid, Sid), Rule>
}
impl<T> Rulebook<T> {
    pub fn new(name: &str, description: &str) -> Rulebook<T> { Rulebook {
        name: name.to_owned(),
        description: description.to_owned(),
        symbols: Vec::new(),
        rules: HashMap::new()
    }}
    pub fn add_symbol(&mut self, label: &str, arity: usize, data: T) -> Sid {
        let symbol = Symbol { label: label.to_owned(), arity, data };
        self.symbols.push(symbol);
        (self.symbols.len() - 1) as Sid
    }
    pub fn get_symbol(&self, id: Sid) -> Option<&Symbol<T>> {
        self.symbols.get(id as usize)
    }
    pub fn add(&mut self, left: RuleNode, right: RuleNode) -> Result<(), RuleError> {
        let mut rb = RuleBuilder::new(&self.symbols);

        let (ls, ld) = match left {
            RuleNode::Symbol(ss, sd) => (ss, sd),
            RuleNode::Port(_) => unreachable!()
        };
        let (rs, rd) = match right {
            RuleNode::Symbol(ss, sd) => (ss, sd),
            RuleNode::Port(_) => unreachable!()
        };
        rb.root(ls)?;
        rb.visit(ld, true)?;
        rb.root(rs)?;
        rb.visit(rd, false)?;

        let rule = Rule {
            left: ls,
            right: rs,
            actions: rb.actions
        };

        self.rules.insert((ls, rs), rule);

        Ok(())
    }
    pub fn get(&self, left: Sid, right: Sid) -> Option<&Rule> {
        self.rules.get(&(left, right))
    }
}

#[macro_export]
macro_rules! rule {
    // Internal: a bracketed child is a nested symbol, e.g. [s, 0]
    (@child [ $($inner:tt)+ ]) => {
        $crate::rule![$($inner)+]
    };
    // Internal: anything else is a port id, e.g. 0, 1
    (@child $port:expr) => {
        $crate::interactionnet::RuleNode::Port($port)
    };

    // Entry point: rule![symbol, child, child, ...]
    ($sym:expr $(, $child:tt)* $(,)?) => {
        $crate::interactionnet::RuleNode::Symbol(
            $sym,
            vec![ $( $crate::rule!(@child $child) ),* ]
        )
    };
}