//! Shared ABI and result types used at VM host/proof boundaries.

pub mod callback_codec;
mod execution;
pub mod fast_codec;
pub mod result_codec;
mod stack_value;

pub use execution::{BackendKind, ExecutionResult, VmState};
pub use stack_value::{
    default_value_for_type_tag, encode_integer, normalize_stack_item_type_tag, StackValue,
    COMPACT_TAG_ARRAY, COMPACT_TAG_BIG_INTEGER, COMPACT_TAG_BOOLEAN, COMPACT_TAG_BUFFER,
    COMPACT_TAG_BYTESTRING, COMPACT_TAG_INTEGER, COMPACT_TAG_INTEROP, COMPACT_TAG_ITERATOR,
    COMPACT_TAG_MAP, COMPACT_TAG_NULL, COMPACT_TAG_POINTER, COMPACT_TAG_STRUCT,
    NEOVM_STACK_ITEM_TYPE_ARRAY, NEOVM_STACK_ITEM_TYPE_BOOLEAN, NEOVM_STACK_ITEM_TYPE_BUFFER,
    NEOVM_STACK_ITEM_TYPE_BYTESTRING, NEOVM_STACK_ITEM_TYPE_INTEGER, NEOVM_STACK_ITEM_TYPE_MAP,
    NEOVM_STACK_ITEM_TYPE_STRUCT,
};
