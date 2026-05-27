use std::vec;

enum Op {
    Add,
    Sub,
    Mult,
    Divide,
    Mov,
}

struct Value {
    id: u64,
    op: Op,
    args: Vec<Value>,
}
