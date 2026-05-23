use super::*;
use crate::{
    semantics::{
        collections as collection_rules, comparison as comparison_rules,
        conversion as conversion_rules, numeric,
    },
    NEOVM_STACK_ITEM_TYPE_ARRAY, NEOVM_STACK_ITEM_TYPE_BOOLEAN, NEOVM_STACK_ITEM_TYPE_BUFFER,
    NEOVM_STACK_ITEM_TYPE_BYTESTRING, NEOVM_STACK_ITEM_TYPE_INTEGER,
    NEOVM_STACK_ITEM_TYPE_INTEROP_INTERFACE, NEOVM_STACK_ITEM_TYPE_MAP,
    NEOVM_STACK_ITEM_TYPE_STRUCT,
};

#[inline]
pub(crate) fn peek_item(stack: &[StackValue]) -> Result<StackValue, String> {
    stack
        .last()
        .cloned()
        .ok_or_else(|| "stack underflow".to_string())
}

#[inline]
pub(crate) fn pop_item(stack: &mut Vec<StackValue>) -> Result<StackValue, String> {
    stack.pop().ok_or_else(|| "stack underflow".to_string())
}

#[inline]
pub(crate) fn pop_integer(stack: &mut Vec<StackValue>) -> Result<i64, String> {
    match stack.pop() {
        Some(StackValue::Integer(value)) => Ok(value),
        Some(StackValue::Boolean(value)) => Ok(if value { 1 } else { 0 }),
        Some(StackValue::ByteString(value)) => decode_signed_le_bytes(&value),
        Some(StackValue::BigInteger(value)) => decode_signed_le_bytes(&value),
        Some(StackValue::Buffer(_, bytes)) => decode_signed_le_bytes(&bytes),
        Some(_) => Err("expected integer on stack".to_string()),
        None => Err("stack underflow".to_string()),
    }
}

pub(crate) fn pop_shift_count(stack: &mut Vec<StackValue>) -> Result<i64, String> {
    match stack.pop() {
        Some(StackValue::Integer(value)) => Ok(value),
        Some(StackValue::Boolean(value)) => Ok(if value { 1 } else { 0 }),
        Some(StackValue::ByteString(value)) => decode_signed_le_bytes(&value),
        Some(StackValue::BigInteger(value)) => decode_signed_le_bytes(&value),
        Some(StackValue::Null) => Err("expected integer-compatible value".to_string()),
        Some(StackValue::Buffer(_, _)) => Err("expected integer-compatible value".to_string()),
        Some(_) => Err("expected integer-compatible shift count".to_string()),
        None => Err("stack underflow".to_string()),
    }
}

#[inline]
pub(crate) fn integer_value_for_collection_index(value: &StackValue) -> Result<i64, String> {
    collection_rules::collection_index_value(&to_abi_value(value))
}

#[inline]
pub(crate) fn validate_map_key(key: &StackValue) -> Result<(), String> {
    collection_rules::validate_map_key_value(&to_abi_value(key))
}

pub(crate) fn primitive_key_equals(left: &StackValue, right: &StackValue) -> bool {
    collection_rules::primitive_key_equal(&to_abi_value(left), &to_abi_value(right))
}

pub(crate) fn vm_equal(left: &StackValue, right: &StackValue) -> bool {
    match (left, right) {
        (StackValue::Integer(l), StackValue::Integer(r)) => l == r,
        (StackValue::Integer(l), StackValue::BigInteger(r))
        | (StackValue::BigInteger(r), StackValue::Integer(l)) => encode_integer(*l) == *r,
        (StackValue::BigInteger(l), StackValue::BigInteger(r)) => l == r,
        (StackValue::ByteString(l), StackValue::ByteString(r)) => l == r,
        (StackValue::Boolean(l), StackValue::Boolean(r)) => l == r,
        (StackValue::Pointer(l), StackValue::Pointer(r)) => l == r,
        (StackValue::Null, StackValue::Null) => true,
        (StackValue::Interop(l), StackValue::Interop(r)) => l == r,
        (StackValue::Iterator(l), StackValue::Iterator(r)) => l == r,
        (StackValue::Array(left_id, _), StackValue::Array(right_id, _))
        | (StackValue::Map(left_id, _), StackValue::Map(right_id, _))
        | (StackValue::Buffer(left_id, _), StackValue::Buffer(right_id, _)) => left_id == right_id,
        (StackValue::Struct(left_id, _), StackValue::Struct(right_id, _))
            if left_id == right_id =>
        {
            true
        }
        (StackValue::Struct(_, _), StackValue::Struct(_, _)) => struct_equal(left, right),
        _ => false,
    }
}

fn struct_equal(left: &StackValue, right: &StackValue) -> bool {
    let mut pending = vec![(left, right)];
    while let Some((left, right)) = pending.pop() {
        match (left, right) {
            (StackValue::Integer(l), StackValue::Integer(r)) => {
                if l != r {
                    return false;
                }
            }
            (StackValue::Integer(l), StackValue::BigInteger(r))
            | (StackValue::BigInteger(r), StackValue::Integer(l)) => {
                if encode_integer(*l) != *r {
                    return false;
                }
            }
            (StackValue::BigInteger(l), StackValue::BigInteger(r)) => {
                if l != r {
                    return false;
                }
            }
            (StackValue::ByteString(l), StackValue::ByteString(r)) => {
                if l != r {
                    return false;
                }
            }
            (StackValue::Boolean(l), StackValue::Boolean(r)) => {
                if l != r {
                    return false;
                }
            }
            (StackValue::Pointer(l), StackValue::Pointer(r)) => {
                if l != r {
                    return false;
                }
            }
            (StackValue::Null, StackValue::Null) => {}
            (StackValue::Interop(l), StackValue::Interop(r)) => {
                if l != r {
                    return false;
                }
            }
            (StackValue::Iterator(l), StackValue::Iterator(r)) => {
                if l != r {
                    return false;
                }
            }
            (StackValue::Array(left_id, _), StackValue::Array(right_id, _))
            | (StackValue::Map(left_id, _), StackValue::Map(right_id, _))
            | (StackValue::Buffer(left_id, _), StackValue::Buffer(right_id, _)) => {
                if left_id != right_id {
                    return false;
                }
            }
            (StackValue::Struct(left_id, _), StackValue::Struct(right_id, _))
                if left_id == right_id => {}
            (StackValue::Struct(_, left_items), StackValue::Struct(_, right_items)) => {
                if left_items.len() != right_items.len() {
                    return false;
                }
                pending.extend(left_items.iter().zip(right_items.iter()));
            }
            _ => return false,
        }
    }
    true
}

pub(crate) fn convert_value(
    kind: u8,
    value: StackValue,
    ids: &mut CompoundIds,
) -> Result<StackValue, String> {
    // Validate target type first, even for Null
    match kind {
        NEOVM_STACK_ITEM_TYPE_BOOLEAN
        | NEOVM_STACK_ITEM_TYPE_INTEGER
        | NEOVM_STACK_ITEM_TYPE_BYTESTRING
        | NEOVM_STACK_ITEM_TYPE_BUFFER
        | NEOVM_STACK_ITEM_TYPE_ARRAY
        | NEOVM_STACK_ITEM_TYPE_STRUCT
        | NEOVM_STACK_ITEM_TYPE_MAP
        | NEOVM_STACK_ITEM_TYPE_INTEROP_INTERFACE => {}
        _ => return Err(format!("unsupported CONVERT target 0x{kind:02x}")),
    }

    if matches!(value, StackValue::Null) {
        return Ok(StackValue::Null);
    }

    match kind {
        NEOVM_STACK_ITEM_TYPE_BOOLEAN
        | NEOVM_STACK_ITEM_TYPE_INTEGER
        | NEOVM_STACK_ITEM_TYPE_BYTESTRING
        | NEOVM_STACK_ITEM_TYPE_BUFFER => {
            let converted = conversion_rules::convert_value(into_abi_value(value), kind)?;
            Ok(ids.import_abi(converted))
        }
        NEOVM_STACK_ITEM_TYPE_ARRAY => Ok(match value {
            StackValue::Array(_, _) => value,
            StackValue::Struct(_, items) => ids.array(items),
            other => return Err(format!("unsupported CONVERT source for Array: {other:?}")),
        }),
        NEOVM_STACK_ITEM_TYPE_STRUCT => Ok(match value {
            StackValue::Struct(_, _) => value,
            StackValue::Array(_, items) => ids.r#struct(items),
            other => return Err(format!("unsupported CONVERT source for Struct: {other:?}")),
        }),
        NEOVM_STACK_ITEM_TYPE_MAP => Ok(match value {
            StackValue::Map(_, _) => value,
            other => return Err(format!("unsupported CONVERT source for Map: {other:?}")),
        }),
        NEOVM_STACK_ITEM_TYPE_INTEROP_INTERFACE => Ok(match value {
            StackValue::Interop(_) => value,
            other => return Err(format!("unsupported CONVERT source for Interop: {other:?}")),
        }),
        _ => Err(format!("unsupported CONVERT target 0x{kind:02x}")),
    }
}

#[inline]
pub(crate) fn decode_signed_le_bytes(bytes: &[u8]) -> Result<i64, String> {
    numeric::decode_signed_le_bytes_i64(bytes)
}

#[inline]
pub(crate) fn pop_boolean(stack: &mut Vec<StackValue>) -> Result<bool, String> {
    let value = stack.pop().ok_or_else(|| "stack underflow".to_string())?;
    Ok(comparison_rules::boolean_value(&to_abi_value(&value)))
}

pub(crate) fn pop_bytes(stack: &mut Vec<StackValue>) -> Result<Vec<u8>, String> {
    match stack.pop() {
        Some(StackValue::ByteString(value)) => Ok(value),
        Some(StackValue::Buffer(_, value)) => Ok(value),
        Some(StackValue::Integer(value)) => Ok(encode_integer(value)),
        Some(StackValue::BigInteger(value)) => Ok(value),
        Some(StackValue::Boolean(value)) => Ok(vec![if value { 1 } else { 0 }]),
        Some(StackValue::Null) => Ok(Vec::new()),
        Some(
            StackValue::Pointer(_)
            | StackValue::Array(..)
            | StackValue::Struct(..)
            | StackValue::Map(..)
            | StackValue::Interop(_)
            | StackValue::Iterator(_),
        ) => Err("expected byte string-compatible item on stack".to_string()),
        None => Err("stack underflow".to_string()),
    }
}

/// Convert a StackValue to bytes without consuming it from the stack.
/// Used by CAT to determine result type while extracting byte content.
pub(crate) fn stack_item_to_bytes(item: StackValue) -> Result<Vec<u8>, String> {
    match item {
        StackValue::ByteString(value) => Ok(value),
        StackValue::Buffer(_, value) => Ok(value),
        StackValue::Integer(value) => Ok(encode_integer(value)),
        StackValue::BigInteger(value) => Ok(value),
        StackValue::Boolean(value) => Ok(vec![if value { 1 } else { 0 }]),
        StackValue::Null => Ok(Vec::new()),
        StackValue::Pointer(_)
        | StackValue::Array(..)
        | StackValue::Struct(..)
        | StackValue::Map(..)
        | StackValue::Interop(_)
        | StackValue::Iterator(_) => {
            Err("expected byte string-compatible item on stack".to_string())
        }
    }
}

pub(crate) fn encode_integer(value: i64) -> Vec<u8> {
    crate::abi::encode_integer(value)
}

/// Distinguishes short (i8, 1-byte) from long (i32, 4-byte) jump offsets.
pub(crate) enum Offset {
    Short,
    Long,
}

/// Read a jump/call offset from the script at `ip`.
/// Returns (signed offset as isize, byte advance for operand).
pub(crate) fn read_offset(
    script: &[u8],
    ip: usize,
    kind: &Offset,
    name: &str,
) -> Result<(isize, usize), String> {
    match kind {
        Offset::Short => {
            if ip + 2 > script.len() {
                return Err(format!("truncated {name} operand"));
            }
            let offset = i8::from_le_bytes([script[ip + 1]]);
            Ok((offset as isize, 2))
        }
        Offset::Long => {
            if ip + 5 > script.len() {
                return Err(format!("truncated {name}_L operand"));
            }
            let offset = i32::from_le_bytes([
                script[ip + 1],
                script[ip + 2],
                script[ip + 3],
                script[ip + 4],
            ]);
            Ok((offset as isize, 5))
        }
    }
}

/// Compute jump/call target from ip + offset with bounds checking.
pub(crate) fn compute_jump_target_offset(
    ip: usize,
    offset: isize,
    script_len: usize,
    name: &str,
) -> Result<usize, String> {
    let target = ip as isize + offset;
    if target < 0 || target as usize > script_len {
        return Err(format!("{name} target out of bounds"));
    }
    Ok(target as usize)
}

#[inline]
pub(crate) fn trim_le_bytes_slice(bytes: &[u8]) -> Vec<u8> {
    numeric::trim_le_bytes_slice(bytes)
}
