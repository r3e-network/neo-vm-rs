//! Shared NeoVM semantics for Neo N4 execution profiles.
//!
//! This crate is intentionally small and `no_std + alloc` compatible. Host
//! runtimes such as PolkaVM and proving runtimes such as SP1 can share these
//! definitions without inheriting each other's execution or proving stack.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod execution;
mod limits;
mod opcode;
mod stack_value;
mod syscall;

pub use execution::{BackendKind, ExecutionResult, VmState};
pub use limits::{
    DEFAULT_MAX_INVOCATION_DEPTH, DEFAULT_MAX_STACK_DEPTH, MAX_ITEM_SIZE, MAX_SCRIPT_SIZE,
};
pub use opcode::OpCode;
pub use stack_value::StackValue;
pub use syscall::{interop_hash, syscall_arg_count};
