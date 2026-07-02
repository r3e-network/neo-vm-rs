//! Shared NeoVM semantics + execution engines for Neo N4 execution profiles.
//!
//! This is the single, canonical NeoVM implementation shared by three projects,
//! so the opcode logic is never re-implemented per project. It is
//! `no_std + alloc` compatible. The common core (opcode **semantics**, the
//! **value model**, opcodes/limits) is always compiled; the two execution
//! engines are cargo features so each consumer pulls only what it needs:
//!
//! - `interpreter` (default) — the direct `match opcode` dispatch loop
//!   ([`interpret`]). Used by the standalone **neo-vm** and by **riscvm**'s
//!   in-process (PolkaVM guest) execution.
//! - `runtime` (default) — the [`VmContext`] + [`RuntimeStack`] ABI for
//!   straight-line / compiled execution. Used by **riscvm**'s NeoVM→Rust→RISC-V
//!   compiler and by **neo-zkvm**'s proving/trace backend.
//!
//! Both engines are built on the shared [`semantics`] rule layer (single source
//! of truth for opcode behavior), so they can never diverge. Selecting features:
//! neo-vm → `interpreter`; neo-zkvm → `runtime`; riscvm → both (the default).

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod abi;
mod host;
// Shared exception carrier used by BOTH execution engines; lives at the crate
// root so neither engine depends on the other and the `interpreter` / `runtime`
// features can be selected independently.
mod pending_exception;
#[cfg(feature = "interpreter")]
mod interpreter;
#[cfg(feature = "runtime")]
pub mod runtime;
pub mod script_builder;
pub mod semantics;
mod vm;

pub(crate) use pending_exception::PendingException;

pub use abi::interoperable::{Interoperable, InteroperableError};
pub use abi::{
    BackendKind, COMPACT_TAG_ARRAY, COMPACT_TAG_BIG_INTEGER, COMPACT_TAG_BOOLEAN,
    COMPACT_TAG_BUFFER, COMPACT_TAG_BYTESTRING, COMPACT_TAG_INTEGER, COMPACT_TAG_INTEROP,
    COMPACT_TAG_ITERATOR, COMPACT_TAG_MAP, COMPACT_TAG_NULL, COMPACT_TAG_POINTER,
    COMPACT_TAG_STRUCT, ExecutionResult, NEOVM_STACK_ITEM_TYPE_ANY, NEOVM_STACK_ITEM_TYPE_ARRAY,
    NEOVM_STACK_ITEM_TYPE_BOOLEAN, NEOVM_STACK_ITEM_TYPE_BUFFER, NEOVM_STACK_ITEM_TYPE_BYTESTRING,
    NEOVM_STACK_ITEM_TYPE_INTEGER, NEOVM_STACK_ITEM_TYPE_INTEROP_INTERFACE,
    NEOVM_STACK_ITEM_TYPE_MAP, NEOVM_STACK_ITEM_TYPE_POINTER, NEOVM_STACK_ITEM_TYPE_STRUCT,
    STACK_VALUE_CODEC_TAG_ARRAY, STACK_VALUE_CODEC_TAG_BIG_INTEGER, STACK_VALUE_CODEC_TAG_BOOLEAN,
    STACK_VALUE_CODEC_TAG_BUFFER, STACK_VALUE_CODEC_TAG_BYTESTRING, STACK_VALUE_CODEC_TAG_INTEGER,
    STACK_VALUE_CODEC_TAG_INTEROP, STACK_VALUE_CODEC_TAG_ITERATOR, STACK_VALUE_CODEC_TAG_MAP,
    STACK_VALUE_CODEC_TAG_NULL, STACK_VALUE_CODEC_TAG_POINTER, STACK_VALUE_CODEC_TAG_STRUCT,
    StackItemType, StackValue, VmState, byte_sequence_bytes, byte_sequence_len,
    concat_splice_values, decode_integer_bytes, default_value_for_type_tag, encode_integer,
    is_defined_neovm_type_tag, new_array_default_value_for_neovm_type_tag,
    new_array_default_value_for_type_tag,
    normalize_stack_item_type_tag, pop_byte_arg, slice_splice_value, stack_value_as_bigint,
    stack_value_as_bool, stack_value_as_bytes, stack_value_as_fixed_bytes, stack_value_as_i64,
    stack_value_as_string, stack_value_as_u8, stack_value_as_u32, stack_value_into_items,
    stack_value_span_bytes,
};
pub use abi::{callback_codec, fast_codec, result_codec};
pub use host::{interop_hash, syscall_arg_count};
#[cfg(feature = "interpreter")]
pub use interpreter::{
    CALLT_MARKER, CALLT_MARKER_HI, INITIALIZER_COMPLETE_MARKER, SyscallProvider, interpret,
    interpret_with_stack_and_syscalls, interpret_with_stack_and_syscalls_at,
    interpret_with_stack_and_syscalls_at_with_initializer,
    interpret_with_stack_and_syscalls_at_with_initializer_and_result_limit,
    interpret_with_stack_and_syscalls_at_with_result_limit, interpret_with_syscalls,
    last_interpreter_ip, last_result_limit, last_result_stack_len, last_result_stage,
};
#[cfg(feature = "runtime")]
pub use runtime::{RuntimeStack, VmContext};
pub use vm::{
    DEFAULT_MAX_INVOCATION_DEPTH, DEFAULT_MAX_STACK_DEPTH, ExceptionHandlingContext,
    ExceptionHandlingState, ExecutionEngineLimits, FromOperand, Instruction, InstructionError,
    InstructionErrorKind, InstructionResult, MAX_ITEM_SIZE, MAX_SCRIPT_SIZE, OpCode,
    ScriptInstruction, Tarjan, ValidatedScript, ValidationResult, VmOrderedDictionary,
};
pub use vm::{
    instruction_jump_target, instruction_try_targets, next_stack_item_id,
    parse_script_instructions, validate_script, validate_strict_script,
};
