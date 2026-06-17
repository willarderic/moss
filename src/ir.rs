use std::vec;

use crate::ast::Node;

#[derive(Clone, Debug, Eq, PartialEq)]
enum Op {
    ADD,
    LOAD,
    STORE,
}

// Representation of SSA.
// Arguments will references to Values stored in the basic block
#[derive(Clone, Debug, Eq, PartialEq)]
struct Value<'a> {
    id: u64,
    op: Op,
    arg1: &'a Value<'a>,
    arg2: &'a Value<'a>,
}

// Basic block is a contiguos sequence of SSA instructions
// the basic block will own the Value structs.
#[derive(Clone, Debug, Eq, PartialEq)]
struct BasicBlock<'a> {
    id: u64,
    vals: Vec<Value<'a>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Fn<'a> {
    id: u64,
    basic_blocks: Vec<BasicBlock<'a>>,
}

struct IR<'a> {
    fns: Vec<Fn<'a>>,
}

impl<'a> IR<'a> {
    pub fn new() -> Self {
        Self { fns: Vec::new() }
    }

    pub fn genssa(&mut self, node: Node) -> Vec<Fn<'a>> {

        self.fns.clone()
    }
}
