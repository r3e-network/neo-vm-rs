//! Shared NeoVM bytecode metadata and execution constants.

mod exception_handling;
mod graph;
mod instruction;
mod limits;
mod opcode;
mod ordered_dictionary;
mod script_validation;

pub use exception_handling::{ExceptionHandlingContext, ExceptionHandlingState};
pub use graph::{Tarjan, next_stack_item_id};
// Only the `interpreter` reads this tag directly (to distinguish global ids);
// the const stays defined in `graph` for `next_stack_item_id` on every build.
#[cfg(feature = "interpreter")]
pub(crate) use graph::GLOBAL_ID_TAG;
pub use instruction::{
    FromOperand, Instruction, InstructionError, InstructionErrorKind, InstructionResult,
};
pub use limits::{
    DEFAULT_MAX_INVOCATION_DEPTH, DEFAULT_MAX_STACK_DEPTH, ExecutionEngineLimits, MAX_ITEM_SIZE,
    MAX_SCRIPT_SIZE,
};
pub use opcode::OpCode;
pub use ordered_dictionary::VmOrderedDictionary;
pub use script_validation::{
    ScriptInstruction, ValidatedScript, ValidationResult, instruction_jump_target,
    instruction_try_targets, parse_script_instructions, validate_script, validate_strict_script,
};
