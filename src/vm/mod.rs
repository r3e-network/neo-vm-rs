//! Shared NeoVM bytecode metadata and execution constants.

mod limits;
mod opcode;

pub use limits::{
    DEFAULT_MAX_INVOCATION_DEPTH, DEFAULT_MAX_STACK_DEPTH, MAX_ITEM_SIZE, MAX_SCRIPT_SIZE,
};
pub use opcode::OpCode;
