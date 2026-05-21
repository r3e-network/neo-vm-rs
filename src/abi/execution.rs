//! Shared VM execution result types.

use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};

use super::stack_value::StackValue;

/// VM execution state after script completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VmState {
    /// Execution completed successfully.
    Halt,
    /// Execution failed with an error.
    Fault,
}

/// Execution backend identifier for result reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendKind {
    /// Direct interpreter execution.
    Interpreter,
}

/// Result of VM script execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Gas consumed in pico units.
    pub fee_consumed_pico: i64,
    /// Final execution state.
    pub state: VmState,
    /// Final evaluation stack contents.
    pub stack: Vec<StackValue>,
    /// Optional user-facing fault message.
    #[serde(default)]
    pub fault_message: Option<String>,
    /// Instruction pointer at the attributed fault location.
    #[serde(default)]
    pub fault_ip: Option<u32>,
    /// Serialized local variables snapshot at the attributed fault location.
    #[serde(default)]
    pub fault_locals: Option<Vec<u8>>,
}
