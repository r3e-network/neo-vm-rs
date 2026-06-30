//! Free helper functions operating on canonical NeoVM stack values.

use alloc::{format, string::String, vec::Vec};
use core::convert::TryInto;
use num_bigint::BigInt;

use crate::semantics::numeric;

use super::{
    COMPACT_TAG_ARRAY, COMPACT_TAG_BIG_INTEGER, COMPACT_TAG_BOOLEAN, COMPACT_TAG_BUFFER,
    COMPACT_TAG_BYTESTRING, COMPACT_TAG_INTEGER, COMPACT_TAG_INTEROP, COMPACT_TAG_MAP,
    COMPACT_TAG_NULL, COMPACT_TAG_POINTER, COMPACT_TAG_STRUCT, NEOVM_STACK_ITEM_TYPE_ANY,
    NEOVM_STACK_ITEM_TYPE_ARRAY, NEOVM_STACK_ITEM_TYPE_BOOLEAN, NEOVM_STACK_ITEM_TYPE_BUFFER,
    NEOVM_STACK_ITEM_TYPE_BYTESTRING, NEOVM_STACK_ITEM_TYPE_INTEGER,
    NEOVM_STACK_ITEM_TYPE_INTEROP_INTERFACE, NEOVM_STACK_ITEM_TYPE_MAP,
    NEOVM_STACK_ITEM_TYPE_POINTER, NEOVM_STACK_ITEM_TYPE_STRUCT, StackValue,
};

/// Normalize NeoVM StackItemType tags and compact runtime tags into one compact tag space.
#[must_use]
pub fn normalize_stack_item_type_tag(type_tag: u8) -> u8 {
    match type_tag {
        NEOVM_STACK_ITEM_TYPE_BOOLEAN => COMPACT_TAG_BOOLEAN,
        NEOVM_STACK_ITEM_TYPE_INTEGER => COMPACT_TAG_INTEGER,
        NEOVM_STACK_ITEM_TYPE_BYTESTRING => COMPACT_TAG_BYTESTRING,
        NEOVM_STACK_ITEM_TYPE_BUFFER => COMPACT_TAG_BUFFER,
        NEOVM_STACK_ITEM_TYPE_ARRAY => COMPACT_TAG_ARRAY,
        NEOVM_STACK_ITEM_TYPE_STRUCT => COMPACT_TAG_STRUCT,
        NEOVM_STACK_ITEM_TYPE_MAP => COMPACT_TAG_MAP,
        NEOVM_STACK_ITEM_TYPE_POINTER => COMPACT_TAG_POINTER,
        NEOVM_STACK_ITEM_TYPE_INTEROP_INTERFACE => COMPACT_TAG_INTEROP,
        other => other,
    }
}

/// Return the shared default value for compact runtime tags and non-ambiguous
/// NeoVM StackItemType tags.
///
/// Compact integer is `0`, which is also NeoVM `StackItemType.Any`; use
/// [`new_array_default_value_for_neovm_type_tag`] for `NEWARRAY_T` operands.
#[must_use]
pub fn default_value_for_type_tag(type_tag: u8) -> StackValue {
    match normalize_stack_item_type_tag(type_tag) {
        COMPACT_TAG_BOOLEAN => StackValue::Boolean(false),
        COMPACT_TAG_INTEGER | COMPACT_TAG_BIG_INTEGER => StackValue::Integer(0),
        COMPACT_TAG_BYTESTRING => StackValue::ByteString(Vec::new()),
        COMPACT_TAG_BUFFER => StackValue::Buffer(crate::next_stack_item_id(), Vec::new()),
        COMPACT_TAG_ARRAY => StackValue::Array(crate::next_stack_item_id(), Vec::new()),
        COMPACT_TAG_STRUCT => StackValue::Struct(crate::next_stack_item_id(), Vec::new()),
        COMPACT_TAG_MAP => StackValue::Map(crate::next_stack_item_id(), Vec::new()),
        COMPACT_TAG_NULL => StackValue::Null,
        _ => StackValue::Null,
    }
}

/// Return the `NEWARRAY_T` item default for compact runtime tags and
/// non-ambiguous NeoVM StackItemType tags.
///
/// Compact integer is `0`, which is also NeoVM `StackItemType.Any`; use
/// [`new_array_default_value_for_neovm_type_tag`] for raw NeoVM opcode
/// operands.
#[must_use]
pub fn new_array_default_value_for_type_tag(type_tag: u8) -> StackValue {
    match normalize_stack_item_type_tag(type_tag) {
        COMPACT_TAG_INTEGER => StackValue::Integer(0),
        COMPACT_TAG_BYTESTRING => StackValue::ByteString(Vec::new()),
        _ => StackValue::Null,
    }
}

/// Return the NeoVM `NEWARRAY_T` item default for a NeoVM StackItemType operand.
#[must_use]
pub fn new_array_default_value_for_neovm_type_tag(type_tag: u8) -> StackValue {
    match type_tag {
        // Canonical NEWARRAY_T default per element type: Boolean=>false,
        // Integer=>0, ByteString=>empty, everything else (Any/Pointer/Buffer/
        // Array/Struct/Map/InteropInterface)=>Null.
        NEOVM_STACK_ITEM_TYPE_BOOLEAN => StackValue::Boolean(false),
        NEOVM_STACK_ITEM_TYPE_INTEGER => StackValue::Integer(0),
        NEOVM_STACK_ITEM_TYPE_BYTESTRING => StackValue::ByteString(Vec::new()),
        _ => StackValue::Null,
    }
}

/// Whether `type_tag` is a defined NeoVM `StackItemType` (canonical
/// `Enum.IsDefined`). NEWARRAY_T faults (uncatchably) on an undefined tag.
pub fn is_defined_neovm_type_tag(type_tag: u8) -> bool {
    matches!(
        type_tag,
        NEOVM_STACK_ITEM_TYPE_ANY
            | NEOVM_STACK_ITEM_TYPE_POINTER
            | NEOVM_STACK_ITEM_TYPE_BOOLEAN
            | NEOVM_STACK_ITEM_TYPE_INTEGER
            | NEOVM_STACK_ITEM_TYPE_BYTESTRING
            | NEOVM_STACK_ITEM_TYPE_BUFFER
            | NEOVM_STACK_ITEM_TYPE_ARRAY
            | NEOVM_STACK_ITEM_TYPE_STRUCT
            | NEOVM_STACK_ITEM_TYPE_MAP
            | NEOVM_STACK_ITEM_TYPE_INTEROP_INTERFACE
    )
}

/// Pop a syscall-style byte argument from the top of a stack.
///
/// NeoVM byte syscalls accept ByteString and Buffer values as raw bytes. This
/// deliberately does not accept BigInteger, even though it has a byte
/// representation, because syscall argument validation is type based.
pub fn pop_byte_arg(stack: &mut Vec<StackValue>, context: &str) -> Result<Vec<u8>, String> {
    match stack.pop() {
        Some(StackValue::ByteString(bytes)) | Some(StackValue::Buffer(_, bytes)) => Ok(bytes),
        Some(other) => Err(format!(
            "{context} expects ByteString or Buffer, got {other:?}"
        )),
        None => Err(format!("{context} expects one stack argument")),
    }
}

/// Borrow the content of NeoVM byte sequence values.
///
/// This is intentionally narrower than [`StackValue::as_bytes`]: NeoVM string
/// and byte-array opcodes accept only `ByteString` and `Buffer`, while integer
/// byte representations are valid only for conversion paths.
#[must_use]
pub fn byte_sequence_bytes(value: &StackValue) -> Option<&[u8]> {
    match value {
        StackValue::ByteString(bytes) | StackValue::Buffer(_, bytes) => Some(bytes),
        _ => None,
    }
}

/// Return the byte length of a NeoVM `ByteString` or `Buffer`.
#[must_use]
pub fn byte_sequence_len(value: &StackValue) -> Option<usize> {
    byte_sequence_bytes(value).map(<[u8]>::len)
}

/// Extract a strict native-contract boolean result.
///
/// Native contract wrappers should use this helper when decoding host results
/// where only Boolean, Integer, or BigInteger values are acceptable.
#[must_use]
pub fn stack_value_as_bool(value: &StackValue) -> Option<bool> {
    match value {
        StackValue::Boolean(value) => Some(*value),
        StackValue::Integer(_) | StackValue::BigInteger(_) => {
            stack_value_as_i64(value).map(|integer| integer != 0)
        }
        _ => None,
    }
}

/// Extract a strict 64-bit signed integer result.
#[must_use]
pub fn stack_value_as_i64(value: &StackValue) -> Option<i64> {
    match value {
        StackValue::Integer(_) | StackValue::BigInteger(_) => value.to_i128()?.try_into().ok(),
        _ => None,
    }
}

/// Convert an integer-compatible stack value to an arbitrary-precision integer.
///
/// This follows NeoVM's little-endian two's-complement integer conversion rules
/// and accepts Buffer values for `StackItem.ConvertTo(Integer)` parity.
pub fn stack_value_as_bigint(value: &StackValue) -> Result<BigInt, String> {
    match value {
        StackValue::Integer(value) => Ok(BigInt::from(*value)),
        StackValue::BigInteger(bytes)
        | StackValue::ByteString(bytes)
        | StackValue::Buffer(_, bytes) => numeric::decode_signed_le_bytes_bigint(bytes),
        StackValue::Boolean(value) => Ok(BigInt::from(if *value { 1 } else { 0 })),
        StackValue::Null
        | StackValue::Array(_, _)
        | StackValue::Struct(_, _)
        | StackValue::Map(_, _)
        | StackValue::Interop(_)
        | StackValue::Iterator(_)
        | StackValue::Pointer(_) => Err("expected integer-compatible value".to_string()),
    }
}

/// Extract a strict 32-bit unsigned integer result.
#[must_use]
pub fn stack_value_as_u32(value: &StackValue) -> Option<u32> {
    stack_value_as_i64(value)?.try_into().ok()
}

/// Extract a strict 8-bit unsigned integer result.
#[must_use]
pub fn stack_value_as_u8(value: &StackValue) -> Option<u8> {
    stack_value_as_i64(value)?.try_into().ok()
}

/// Extract a copy of a NeoVM byte-sequence value.
#[must_use]
pub fn stack_value_as_bytes(value: &StackValue) -> Option<Vec<u8>> {
    byte_sequence_bytes(value).map(<[u8]>::to_vec)
}

/// Return the bytes exposed by NeoVM `StackItem.GetSpan`.
///
/// Splice opcodes such as CAT, LEFT, RIGHT, SUBSTR, and MEMCPY use this path:
/// primitive values expose their memory representation, buffers expose their
/// mutable bytes, and non-span values such as Null or compound references fault.
#[must_use]
pub fn stack_value_span_bytes(value: &StackValue) -> Option<Vec<u8>> {
    match value {
        StackValue::Integer(value) => Some(encode_integer(*value)),
        StackValue::BigInteger(bytes)
        | StackValue::ByteString(bytes)
        | StackValue::Buffer(_, bytes) => Some(bytes.clone()),
        StackValue::Boolean(value) => Some(alloc::vec![u8::from(*value)]),
        StackValue::Null
        | StackValue::Array(_, _)
        | StackValue::Struct(_, _)
        | StackValue::Map(_, _)
        | StackValue::Interop(_)
        | StackValue::Iterator(_)
        | StackValue::Pointer(_) => None,
    }
}

/// Extract a fixed-width byte array from a NeoVM byte-sequence value.
#[must_use]
pub fn stack_value_as_fixed_bytes<const N: usize>(value: &StackValue) -> Option<[u8; N]> {
    stack_value_as_bytes(value)?.try_into().ok()
}

/// Extract a UTF-8 string from a NeoVM byte-sequence value.
#[must_use]
pub fn stack_value_as_string(value: &StackValue) -> Option<String> {
    String::from_utf8(stack_value_as_bytes(value)?).ok()
}

/// Extract the owned items from an Array or Struct value.
#[must_use]
pub fn stack_value_into_items(value: StackValue) -> Option<Vec<StackValue>> {
    match value {
        StackValue::Array(_, items) | StackValue::Struct(_, items) => Some(items),
        _ => None,
    }
}

/// Concatenate two NeoVM GetSpan-compatible values as the CAT opcode does.
#[must_use]
pub fn concat_splice_values(left: &StackValue, right: &StackValue) -> Option<StackValue> {
    let mut bytes = stack_value_span_bytes(left)?;
    bytes.extend_from_slice(&stack_value_span_bytes(right)?);
    Some(StackValue::Buffer(crate::next_stack_item_id(), bytes))
}

/// Slice one NeoVM GetSpan-compatible value as LEFT/RIGHT/SUBSTR do.
#[must_use]
pub fn slice_splice_value(value: &StackValue, index: usize, count: usize) -> Option<StackValue> {
    let bytes = stack_value_span_bytes(value)?;
    let end = index.checked_add(count)?;
    bytes
        .get(index..end)
        .map(|slice| StackValue::Buffer(crate::next_stack_item_id(), slice.to_vec()))
}

/// Encode an integer using NeoVM's minimal little-endian two's-complement form.
#[must_use]
pub fn encode_integer(value: i64) -> Vec<u8> {
    if value == 0 {
        return Vec::new();
    }

    let mut bytes = value.to_le_bytes().to_vec();
    if value > 0 {
        while bytes.len() > 1 && bytes.last() == Some(&0) {
            if bytes[bytes.len() - 2] & 0x80 != 0 {
                break;
            }
            bytes.pop();
        }
    } else {
        while bytes.len() > 1 && bytes.last() == Some(&0xff) {
            if bytes[bytes.len() - 2] & 0x80 == 0 {
                break;
            }
            bytes.pop();
        }
    }

    bytes
}

/// Decode NeoVM little-endian two's-complement integer bytes.
pub fn decode_integer_bytes(bytes: &[u8]) -> Result<BigInt, String> {
    numeric::decode_signed_le_bytes_bigint(bytes)
}
