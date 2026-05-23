//! Runtime-level opcode adapters shared by compiled VM runtimes.
//!
//! The pure modules in `semantics::{arithmetic, comparison, conversion,
//! collections}` own the VM rules. This module owns the common stack operation
//! shape around those rules: pop operands, call the shared rule, push results,
//! and report faults through a host runtime adapter.

pub mod arithmetic;
pub mod bytes;
pub mod collections;
pub mod comparison;
pub mod conversion;
pub mod stack;
pub(crate) mod value_stack;
