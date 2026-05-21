//! Shared ABI and result types used at VM host/proof boundaries.

mod execution;
mod stack_value;

pub use execution::{BackendKind, ExecutionResult, VmState};
pub use stack_value::StackValue;
