//! Shared ABI and result types used at VM host/proof boundaries.

mod execution;
pub mod fast_codec;
mod stack_value;

pub use execution::{BackendKind, ExecutionResult, VmState};
pub use stack_value::{
    encode_integer, StackValue, COMPACT_TAG_ARRAY, COMPACT_TAG_BIG_INTEGER, COMPACT_TAG_BOOLEAN,
    COMPACT_TAG_BUFFER, COMPACT_TAG_BYTESTRING, COMPACT_TAG_INTEGER, COMPACT_TAG_INTEROP,
    COMPACT_TAG_ITERATOR, COMPACT_TAG_MAP, COMPACT_TAG_NULL, COMPACT_TAG_POINTER,
    COMPACT_TAG_STRUCT,
};
