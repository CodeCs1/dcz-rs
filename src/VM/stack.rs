use crate::{codegen::ir_opcode::Opcode, Value::Value};

#[derive(Clone,Debug)]
pub enum Stack {
    Value(Value),
    MemoryAddr(usize),
    CompressedFunc(Vec<Opcode>)
}

impl Stack {
    pub fn as_value(self) -> Value {
        if let Self::Value(v) = self {
            return v;
        }
        Value::Null
    }
    pub fn as_memory_addr(self) -> usize {
        if let Self::MemoryAddr(addr) = self {
            return addr;
        }
        0
    }
    pub fn as_compressed_func(self) -> Vec<Opcode> {
        if let Self::CompressedFunc(f) = self {
            return f;
        }
        Vec::new()
    }
}
