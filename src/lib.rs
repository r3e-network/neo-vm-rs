//! Shared NeoVM semantics and interpreter for Neo N4 execution profiles.
//!
//! This crate is `no_std + alloc` compatible. Host runtimes such as PolkaVM and
//! proving runtimes such as SP1 can share the same VM-facing types and
//! interpreter behavior without inheriting each other's host or proving stack.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod abi;
mod host;
mod interpreter;
mod vm;

pub use abi::{callback_codec, fast_codec, result_codec};
pub use abi::{
    encode_integer, BackendKind, ExecutionResult, StackValue, VmState, COMPACT_TAG_ARRAY,
    COMPACT_TAG_BIG_INTEGER, COMPACT_TAG_BOOLEAN, COMPACT_TAG_BUFFER, COMPACT_TAG_BYTESTRING,
    COMPACT_TAG_INTEGER, COMPACT_TAG_INTEROP, COMPACT_TAG_ITERATOR, COMPACT_TAG_MAP,
    COMPACT_TAG_NULL, COMPACT_TAG_POINTER, COMPACT_TAG_STRUCT,
};
pub use host::{interop_hash, syscall_arg_count};
pub use interpreter::{
    interpret, interpret_with_stack_and_syscalls, interpret_with_stack_and_syscalls_at,
    interpret_with_stack_and_syscalls_at_with_initializer,
    interpret_with_stack_and_syscalls_at_with_initializer_and_result_limit,
    interpret_with_stack_and_syscalls_at_with_result_limit, interpret_with_syscalls,
    last_interpreter_ip, last_result_limit, last_result_stack_len, last_result_stage,
    SyscallProvider, CALLT_MARKER, CALLT_MARKER_HI, INITIALIZER_COMPLETE_MARKER,
};
pub use vm::{
    OpCode, DEFAULT_MAX_INVOCATION_DEPTH, DEFAULT_MAX_STACK_DEPTH, MAX_ITEM_SIZE, MAX_SCRIPT_SIZE,
};
