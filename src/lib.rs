//! Shared NeoVM semantics for Neo N4 execution profiles.
//!
//! This crate is intentionally small and `no_std + alloc` compatible. Host
//! runtimes such as PolkaVM and proving runtimes such as SP1 can share these
//! definitions without inheriting each other's execution or proving stack.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod abi;
mod host;
mod vm;

pub use abi::{BackendKind, ExecutionResult, StackValue, VmState};
pub use host::{interop_hash, syscall_arg_count};
pub use vm::{
    OpCode, DEFAULT_MAX_INVOCATION_DEPTH, DEFAULT_MAX_STACK_DEPTH, MAX_ITEM_SIZE, MAX_SCRIPT_SIZE,
};
