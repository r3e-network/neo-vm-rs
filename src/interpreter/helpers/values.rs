use super::*;
use crate::{
    NEOVM_STACK_ITEM_TYPE_ANY, NEOVM_STACK_ITEM_TYPE_ARRAY, NEOVM_STACK_ITEM_TYPE_BOOLEAN,
    NEOVM_STACK_ITEM_TYPE_BUFFER, NEOVM_STACK_ITEM_TYPE_BYTESTRING, NEOVM_STACK_ITEM_TYPE_INTEGER,
    NEOVM_STACK_ITEM_TYPE_INTEROP_INTERFACE, NEOVM_STACK_ITEM_TYPE_MAP,
    NEOVM_STACK_ITEM_TYPE_POINTER, NEOVM_STACK_ITEM_TYPE_STRUCT, StackItemType,
    interpreter::runtime_types::structurally_equal,
    semantics::{
        collections as collection_rules, comparison as comparison_rules,
        conversion as conversion_rules, numeric,
    },
    stack_value_span_bytes,
};

#[inline]
pub(crate) fn pop_item(stack: &mut Vec<StackValue>) -> Result<StackValue, String> {
    stack.pop().ok_or_else(|| "stack underflow".to_string())
}

#[inline]
pub(crate) fn pop_integer(stack: &mut Vec<StackValue>) -> Result<i64, String> {
    let value = pop_item(stack)?;
    stack_value_i64(&value)?.ok_or_else(|| "expected integer on stack".to_string())
}

pub(crate) fn pop_shift_count(stack: &mut Vec<StackValue>) -> Result<i64, String> {
    let value = pop_item(stack)?;
    if let Some(integer) = stack_value_i64(&value)? {
        return Ok(integer);
    }

    match value {
        StackValue::Null | StackValue::Buffer(_, _) => {
            Err("expected integer-compatible value".to_string())
        }
        _ => Err("expected integer-compatible shift count".to_string()),
    }
}

/// Decode a stack value as an i64 the way canonical NeoVM's `(int)GetInteger()`
/// does for opcode count/index operands. Integer/Boolean/ByteString/BigInteger
/// decode (ByteString/BigInteger capped at 32 bytes by the shared decoder).
/// Everything else — **Buffer** (NOT a PrimitiveType; its `GetInteger()` throws
/// an uncatchable InvalidCastException), Null, compounds, Pointer, Interop —
/// returns `None` so the caller faults, matching canonical. Buffer was
/// previously accepted here, which let PICK/ROLL/XDROP/REVERSEN, NEWBUFFER/
/// LEFT/RIGHT/SUBSTR/MEMCPY and NEWARRAY/NEWSTRUCT/PACK* HALT on a Buffer
/// count/index where canonical FAULTs (consensus split).
fn stack_value_i64(value: &StackValue) -> Result<Option<i64>, String> {
    match value {
        StackValue::Integer(value) => Ok(Some(*value)),
        StackValue::Boolean(value) => Ok(Some(if *value { 1 } else { 0 })),
        StackValue::ByteString(bytes) | StackValue::BigInteger(bytes) => {
            numeric::decode_signed_le_bytes_i64(bytes).map(Some)
        }
        _ => Ok(None),
    }
}

/// Strict canonical index decode for opcodes whose key is popped as a
/// `PrimitiveType` and cast `(int)GetInteger()` — a null/non-integer key or an
/// Int32-overflowing value is an (uncatchable) fault, not coerced to 0 (REMOVE,
/// SETITEM array/buffer). The returned value may be negative; the caller decides.
#[inline]
pub(crate) fn integer_value_for_collection_index_strict(value: &StackValue) -> Result<i64, String> {
    collection_rules::collection_index_value_strict(value)
}

#[inline]
pub(crate) fn validate_map_key(key: &StackValue) -> Result<(), String> {
    collection_rules::validate_map_key_value(key)
}

pub(crate) fn primitive_key_equals(left: &StackValue, right: &StackValue) -> bool {
    collection_rules::primitive_key_equal(left, right)
}

pub(crate) fn vm_equal(left: &StackValue, right: &StackValue) -> bool {
    match (left, right) {
        (StackValue::Integer(l), StackValue::Integer(r)) => l == r,
        (StackValue::Integer(l), StackValue::BigInteger(r))
        | (StackValue::BigInteger(r), StackValue::Integer(l)) => {
            crate::abi::encode_integer(*l) == *r
        }
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
        (StackValue::Struct(_, _), StackValue::Struct(_, _)) => structurally_equal(left, right),
        _ => false,
    }
}

/// C# `ExecutionEngineLimits.MaxComparableSize` (Neo.VM/ExecutionEngineLimits.cs:47).
const MAX_COMPARABLE_SIZE: usize = 65536;

#[cfg(test)]
mod equals_limits_tests {
    use super::*;

    #[test]
    fn bytestring_over_max_comparable_size_faults() {
        let big = StackValue::ByteString(vec![0u8; MAX_COMPARABLE_SIZE + 1]);
        let other = StackValue::ByteString(vec![0u8; 1]);
        assert!(
            equals_with_limits(&big, &other).is_err(),
            "comparing a >MaxComparableSize ByteString must FAULT like C#"
        );
    }

    #[test]
    fn bytestring_at_max_comparable_size_ok() {
        let a = StackValue::ByteString(vec![0u8; MAX_COMPARABLE_SIZE]);
        let b = StackValue::ByteString(vec![0u8; MAX_COMPARABLE_SIZE]);
        assert!(equals_with_limits(&a, &b).unwrap());
    }

    #[test]
    fn small_bytestrings_compare_by_value() {
        let a = StackValue::ByteString(vec![1, 2, 3]);
        let b = StackValue::ByteString(vec![1, 2, 3]);
        let c = StackValue::ByteString(vec![1, 2, 4]);
        assert!(equals_with_limits(&a, &b).unwrap());
        assert!(!equals_with_limits(&a, &c).unwrap());
    }
}

/// EQUAL/NOTEQUAL with the C# comparable-size limits enforced, mirroring
/// `StackItem.Equals(StackItem, ExecutionEngineLimits)`. Dispatches on the LEFT
/// operand exactly like C# `x1.Equals(x2, limits)`: only a ByteString left
/// operand is size-checked, and Struct comparison is bounded by both the item
/// count (`MaxStackSize`) and the cumulative comparable size (`MaxComparableSize`).
/// Returns `Err` (engine FAULT) when a limit is exceeded.
pub(crate) fn equals_with_limits(left: &StackValue, right: &StackValue) -> Result<bool, String> {
    match left {
        StackValue::ByteString(l) => {
            let mut budget = MAX_COMPARABLE_SIZE;
            byte_string_size_eq_with_budget(l, right, &mut budget)
        }
        StackValue::Struct(lid, _) => match right {
            StackValue::Struct(rid, _) => {
                if lid == rid {
                    Ok(true)
                } else {
                    bounded_structural_equal(left, right)
                }
            }
            _ => Ok(false),
        },
        // Arrays/Maps/Buffers compare by reference, primitives by value — no size
        // budget applies (C# only ByteString/Struct enforce MaxComparableSize).
        _ => Ok(vm_equal(left, right)),
    }
}

/// Port of C# `ByteString.Equals(other, limits)` (Neo.VM/Types/ByteString.cs:60-72):
/// faults if either operand size exceeds the remaining comparable-size budget,
/// and decrements that budget by the compared size.
fn byte_string_size_eq_with_budget(
    a: &[u8],
    other: &StackValue,
    limits: &mut usize,
) -> Result<bool, String> {
    let a_size = a.len();
    if a_size > *limits || *limits == 0 {
        return Err(format!(
            "The operand exceeds the maximum comparable size, {a_size}/{limits}.",
            limits = *limits
        ));
    }
    let mut compared_size = 1usize;
    let result = match other {
        StackValue::ByteString(b) => {
            compared_size = compared_size.max(a_size).max(b.len());
            if b.len() > *limits {
                return Err(format!(
                    "The operand exceeds the maximum comparable size, {}/{}.",
                    b.len(),
                    *limits
                ));
            }
            Ok(a == b.as_slice())
        }
        _ => Ok(false),
    };
    *limits = limits.saturating_sub(compared_size);
    result
}

/// Port of C# `Struct.Equals(other, limits)` (Neo.VM/Types/Struct.cs:91-132): an
/// iterative two-stack walk bounded by `MaxStackSize` (item count) and
/// `MaxComparableSize` (cumulative comparable size). Both `left` and `right` are
/// `Struct`.
fn bounded_structural_equal(left: &StackValue, right: &StackValue) -> Result<bool, String> {
    let mut stack1: Vec<StackValue> = vec![left.clone()];
    let mut stack2: Vec<StackValue> = vec![right.clone()];
    let mut count = crate::interpreter::state::MAX_STACK_SIZE;
    let mut max_comparable_size = MAX_COMPARABLE_SIZE;

    while let Some(a) = stack1.pop() {
        if count == 0 {
            return Err("Too many struct items to compare in struct comparison.".to_string());
        }
        count -= 1;

        let b = stack2
            .pop()
            .ok_or_else(|| "Struct comparison stack underflow".to_string())?;

        if let StackValue::ByteString(bytes) = &a {
            if !byte_string_size_eq_with_budget(bytes, &b, &mut max_comparable_size)? {
                return Ok(false);
            }
        } else {
            if max_comparable_size == 0 {
                return Err(
                    "The operand exceeds the maximum comparable size in struct comparison."
                        .to_string(),
                );
            }
            max_comparable_size -= 1;

            if let StackValue::Struct(a_id, a_items) = &a {
                match &b {
                    StackValue::Struct(b_id, b_items) => {
                        if a_id == b_id {
                            continue;
                        }
                        if a_items.len() != b_items.len() {
                            return Ok(false);
                        }
                        for item in a_items.iter() {
                            stack1.push(item.clone());
                        }
                        for item in b_items.iter() {
                            stack2.push(item.clone());
                        }
                    }
                    _ => return Ok(false),
                }
            } else if !vm_equal(&a, &b) {
                return Ok(false);
            }
        }
    }

    Ok(true)
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
        | NEOVM_STACK_ITEM_TYPE_POINTER
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
            let converted = conversion_rules::convert_value(value, kind)?;
            Ok(converted)
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
        // Canonical: base StackItem.ConvertTo returns `this` when type == Type,
        // so Pointer->Pointer succeeds (Null->Pointer is handled by the Null
        // short-circuit above). Any other source hits InvalidCastException.
        NEOVM_STACK_ITEM_TYPE_POINTER => Ok(match value {
            StackValue::Pointer(_) => value,
            other => return Err(format!("unsupported CONVERT source for Pointer: {other:?}")),
        }),
        _ => Err(format!("unsupported CONVERT target 0x{kind:02x}")),
    }
}

pub(crate) fn is_type(kind: u8, value: &StackValue) -> Result<bool, String> {
    match StackItemType::from_byte(kind) {
        Some(StackItemType::Any) => Err(format!(
            "unsupported ISTYPE kind {NEOVM_STACK_ITEM_TYPE_ANY:#04x}"
        )),
        Some(_) => Ok(conversion_rules::is_type(value, kind)),
        None => Err(format!("unsupported ISTYPE kind 0x{kind:02x}")),
    }
}

#[inline]
pub(crate) fn is_null(value: &StackValue) -> bool {
    comparison_rules::is_null(value)
}

#[inline]
pub(crate) fn pop_boolean(stack: &mut Vec<StackValue>) -> Result<bool, String> {
    let value = stack.pop().ok_or_else(|| "stack underflow".to_string())?;
    // Canonical boolean coercion is `GetBoolean()`, which faults (uncatchable
    // InvalidCastException) for a ByteString/Integer wider than Integer.MaxSize
    // (32 bytes). JMPIF/JMPIFNOT/ASSERT/etc. must use this strict path, not a lax
    // any-nonzero-byte truthiness check.
    comparison_rules::strict_boolean_value(&value)
}

pub(crate) fn pop_bytes(stack: &mut Vec<StackValue>) -> Result<Vec<u8>, String> {
    let value = stack.pop().ok_or_else(|| "stack underflow".to_string())?;
    stack_item_to_bytes(value)
}

/// Convert a StackValue to bytes without consuming it from the stack.
/// Used by CAT to determine result type while extracting byte content.
pub(crate) fn stack_item_to_bytes(item: StackValue) -> Result<Vec<u8>, String> {
    stack_value_span_bytes(&item)
        .ok_or_else(|| "expected byte memory-compatible item on stack".to_string())
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

/// Compute a CALL/CALLA target from ip + offset. Canonical CALL assigns the
/// target through the InstructionPointer setter, which faults only on
/// `value > Script.Length` — so a target == `script_len` is allowed.
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

/// Compute a JMP-family target from ip + offset, rejecting `target == script_len`.
///
/// Canonical `ExecuteJump` faults when `position >= Script.Length` (strict `>=`),
/// unlike CALL which uses the IP setter's `> Script.Length`. So the JMP* opcodes
/// must reject a target landing exactly at the end of the script (which would
/// otherwise fall through and HALT cleanly instead of FAULTing).
pub(crate) fn compute_jump_target_strict(
    ip: usize,
    offset: isize,
    script_len: usize,
    name: &str,
) -> Result<usize, String> {
    let target = ip as isize + offset;
    if target < 0 || target as usize >= script_len {
        return Err(format!("{name} target out of bounds"));
    }
    Ok(target as usize)
}
