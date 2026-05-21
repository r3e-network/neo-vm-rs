//! Shared ABI and result types used at VM host/proof boundaries.

mod execution;
pub mod fast_codec;
mod stack_value;

pub use execution::{BackendKind, ExecutionResult, VmState};
pub use stack_value::{
    encode_integer, StackValue, TAG_ARRAY, TAG_BIG_INTEGER, TAG_BOOLEAN, TAG_BUFFER,
    TAG_BYTESTRING, TAG_INTEGER, TAG_INTEROP, TAG_ITERATOR, TAG_MAP, TAG_NULL, TAG_POINTER,
    TAG_STRUCT,
};
