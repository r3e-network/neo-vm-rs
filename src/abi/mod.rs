//! Shared ABI and result types used at VM host/proof boundaries.

mod execution;
pub mod fast_codec;
mod stack_value;

pub use execution::{BackendKind, ExecutionResult, VmState};
pub use stack_value::{encode_integer, StackValue};
