use super::*;

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

#[inline]
pub(crate) fn pop_bigint_pair_allowing_null_false(
    stack: &mut Vec<StackValue>,
) -> Result<Option<(BigInt, BigInt)>, String> {
    let right = stack.pop().ok_or_else(|| "stack underflow".to_string())?;
    let left = stack.pop().ok_or_else(|| "stack underflow".to_string())?;

    if matches!(left, StackValue::Null) || matches!(right, StackValue::Null) {
        return Ok(None);
    }

    Ok(Some((
        bigint_for_comparison(left)?,
        bigint_for_comparison(right)?,
    )))
}

fn bigint_for_comparison(value: StackValue) -> Result<BigInt, String> {
    match value {
        StackValue::Integer(value) => Ok(BigInt::from(value)),
        StackValue::BigInteger(value) => decode_signed_le_bytes_bigint(&value),
        StackValue::ByteString(value) => decode_signed_le_bytes_bigint(&value),
        StackValue::Boolean(value) => Ok(BigInt::from(if value { 1 } else { 0 })),
        StackValue::Pointer(_) => Err("expected integer on stack".to_string()),
        StackValue::Array(..) => Err("expected integer on stack".to_string()),
        StackValue::Struct(..) => Err("expected integer on stack".to_string()),
        StackValue::Map(..) => Err("expected integer on stack".to_string()),
        StackValue::Buffer(_, _) => Err("expected integer on stack".to_string()),
        StackValue::Interop(_) => Err("expected integer on stack".to_string()),
        StackValue::Iterator(_) => Err("expected integer on stack".to_string()),
        StackValue::Null => Err("expected integer on stack".to_string()),
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

pub(crate) fn pop_numeric_bigint(stack: &mut Vec<StackValue>) -> Result<BigInt, String> {
    match stack.pop() {
        Some(StackValue::Integer(value)) => Ok(BigInt::from(value)),
        Some(StackValue::Boolean(value)) => Ok(BigInt::from(if value { 1 } else { 0 })),
        Some(StackValue::ByteString(value)) => decode_signed_le_bytes_bigint(&value),
        Some(StackValue::BigInteger(value)) => decode_signed_le_bytes_bigint(&value),
        Some(StackValue::Null) => Err("expected integer-compatible value".to_string()),
        Some(StackValue::Buffer(_, _)) => Err("expected integer-compatible value".to_string()),
        Some(StackValue::Pointer(_)) => Err("expected integer-compatible value".to_string()),
        Some(StackValue::Array(..)) => Err("expected integer-compatible value".to_string()),
        Some(StackValue::Struct(..)) => Err("expected integer-compatible value".to_string()),
        Some(StackValue::Map(..)) => Err("expected integer-compatible value".to_string()),
        Some(StackValue::Interop(_)) => Err("expected integer-compatible value".to_string()),
        Some(StackValue::Iterator(_)) => Err("expected integer-compatible value".to_string()),
        None => Err("stack underflow".to_string()),
    }
}

pub(crate) fn shift_value_from_item(value: StackValue) -> Result<ShiftValue, String> {
    match value {
        StackValue::Integer(value) => Ok(ShiftValue(BigInt::from(value))),
        StackValue::Boolean(value) => Ok(ShiftValue(BigInt::from(if value { 1 } else { 0 }))),
        StackValue::ByteString(value) => Ok(ShiftValue(decode_signed_le_bytes_bigint(&value)?)),
        StackValue::BigInteger(value) => Ok(ShiftValue(decode_signed_le_bytes_bigint(&value)?)),
        StackValue::Null => Err("expected integer-compatible shift value".to_string()),
        StackValue::Pointer(_) => Err("expected integer-compatible shift value".to_string()),
        StackValue::Array(..) => Err("expected integer-compatible shift value".to_string()),
        StackValue::Struct(..) => Err("expected integer-compatible shift value".to_string()),
        StackValue::Map(..) => Err("expected integer-compatible shift value".to_string()),
        StackValue::Buffer(_, _) => Err("expected integer-compatible shift value".to_string()),
        StackValue::Interop(_) => Err("expected integer-compatible shift value".to_string()),
        StackValue::Iterator(_) => Err("expected integer-compatible shift value".to_string()),
    }
}

pub(crate) fn num_equal(left: &StackValue, right: &StackValue) -> Result<bool, String> {
    Ok(integer_value_for_num_equal(left)? == integer_value_for_num_equal(right)?)
}

fn integer_value_for_num_equal(value: &StackValue) -> Result<BigInt, String> {
    match value {
        StackValue::Integer(value) => Ok(BigInt::from(*value)),
        StackValue::BigInteger(value) | StackValue::ByteString(value) => {
            decode_signed_le_bytes_bigint(value)
        }
        StackValue::Boolean(value) => Ok(BigInt::from(if *value { 1 } else { 0 })),
        StackValue::Null
        | StackValue::Pointer(_)
        | StackValue::Array(..)
        | StackValue::Struct(..)
        | StackValue::Map(..)
        | StackValue::Buffer(..)
        | StackValue::Interop(_)
        | StackValue::Iterator(_) => {
            Err("NUMEQUAL expects primitive numeric or byte string values".to_string())
        }
    }
}

#[inline]
pub(crate) fn integer_value_for_collection_index(value: &StackValue) -> Result<i64, String> {
    match value {
        StackValue::Integer(value) => Ok(*value),
        StackValue::Boolean(value) => Ok(if *value { 1 } else { 0 }),
        StackValue::ByteString(value) => decode_signed_le_bytes(value),
        StackValue::BigInteger(value) => decode_signed_le_bytes(value),
        StackValue::Null => Ok(0),
        _ => Err("expected integer-compatible collection index".to_string()),
    }
}

#[inline]
pub(crate) fn validate_map_key(key: &StackValue) -> Result<(), String> {
    match key {
        StackValue::Integer(_) | StackValue::Boolean(_) | StackValue::Null => Ok(()),
        StackValue::ByteString(value) => {
            if value.len() > 64 {
                Err("map key exceeds maximum size".to_string())
            } else {
                Ok(())
            }
        }
        _ => Err("map key must be primitive".to_string()),
    }
}

pub(crate) fn primitive_key_equals(left: &StackValue, right: &StackValue) -> bool {
    match (left, right) {
        (StackValue::Integer(left), StackValue::Integer(right)) => left == right,
        (StackValue::Boolean(left), StackValue::Boolean(right)) => left == right,
        (StackValue::Null, StackValue::Null) => true,
        (StackValue::ByteString(left), StackValue::ByteString(right)) => left == right,
        _ => false,
    }
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
        0x20 | 0x21 | 0x28 | 0x30 | 0x40 | 0x41 | 0x48 | 0x60 => {}
        _ => return Err(format!("unsupported CONVERT target 0x{kind:02x}")),
    }

    if matches!(value, StackValue::Null) {
        return Ok(StackValue::Null);
    }

    match kind {
        0x20 => Ok(StackValue::Boolean(boolean_value(&value)?)),
        0x21 => Ok(match value {
            StackValue::Integer(value) => StackValue::Integer(value),
            StackValue::Boolean(value) => StackValue::Integer(if value { 1 } else { 0 }),
            StackValue::ByteString(bytes) | StackValue::BigInteger(bytes) => numeric_result_bigint(
                decode_signed_le_bytes_bigint(&bytes)?,
                "integer size exceeds maximum",
            )?,
            StackValue::Buffer(_, bytes) => numeric_result_bigint(
                decode_signed_le_bytes_bigint(&bytes)?,
                "integer size exceeds maximum",
            )?,
            other => return Err(format!("unsupported CONVERT source for Integer: {other:?}")),
        }),
        0x28 => Ok(match value {
            StackValue::ByteString(bytes) => StackValue::ByteString(bytes),
            StackValue::Buffer(_, bytes) => StackValue::ByteString(bytes),
            StackValue::Integer(value) => StackValue::ByteString(encode_integer(value)),
            StackValue::Boolean(value) => StackValue::ByteString(vec![if value { 1 } else { 0 }]),
            StackValue::BigInteger(value) => StackValue::ByteString(value),
            other => {
                return Err(format!(
                    "unsupported CONVERT source for ByteString: {other:?}"
                ))
            }
        }),
        0x30 => Ok(match value {
            StackValue::ByteString(bytes) => ids.buffer(bytes),
            StackValue::Buffer(_, _) => value,
            StackValue::Integer(value) => ids.buffer(encode_integer(value)),
            StackValue::BigInteger(value) => ids.buffer(value),
            StackValue::Boolean(value) => ids.buffer(vec![if value { 1 } else { 0 }]),
            other => return Err(format!("unsupported CONVERT source for Buffer: {other:?}")),
        }),
        0x40 => Ok(match value {
            StackValue::Array(_, _) => value,
            StackValue::Struct(_, items) => ids.array(items),
            other => return Err(format!("unsupported CONVERT source for Array: {other:?}")),
        }),
        0x41 => Ok(match value {
            StackValue::Struct(_, _) => value,
            StackValue::Array(_, items) => ids.r#struct(items),
            other => return Err(format!("unsupported CONVERT source for Struct: {other:?}")),
        }),
        0x48 => Ok(match value {
            StackValue::Map(_, _) => value,
            other => return Err(format!("unsupported CONVERT source for Map: {other:?}")),
        }),
        0x60 => Ok(match value {
            StackValue::Interop(_) => value,
            other => return Err(format!("unsupported CONVERT source for Interop: {other:?}")),
        }),
        _ => Err(format!("unsupported CONVERT target 0x{kind:02x}")),
    }
}

#[inline]
pub(crate) fn boolean_value(value: &StackValue) -> Result<bool, String> {
    match value {
        StackValue::Boolean(value) => Ok(*value),
        StackValue::Integer(value) => Ok(*value != 0),
        StackValue::BigInteger(value) => Ok(value.iter().any(|byte| *byte != 0)),
        StackValue::ByteString(value) => Ok(value.iter().any(|byte| *byte != 0)),
        StackValue::Buffer(_, _) => Ok(true), // Buffer is a compound type, always true
        StackValue::Pointer(_) => Ok(true),
        StackValue::Array(..) => Ok(true),
        StackValue::Struct(..) => Ok(true),
        StackValue::Map(..) => Ok(true),
        StackValue::Interop(_) => Ok(true),
        StackValue::Iterator(_) => Ok(true),
        StackValue::Null => Ok(false),
    }
}

pub(crate) fn decode_signed_le_bytes(bytes: &[u8]) -> Result<i64, String> {
    if bytes.is_empty() {
        return Ok(0);
    }
    if bytes.len() > MAX_INTEGER_SIZE {
        return Err("integer size exceeds maximum".to_string());
    }

    let sign_extend = if bytes.last().is_some_and(|byte| byte & 0x80 != 0) {
        0xff
    } else {
        0x00
    };

    if bytes.len() > 8 {
        if bytes.iter().all(|byte| *byte == 0) {
            return Ok(0);
        }

        if bytes[8..].iter().all(|byte| *byte == sign_extend)
            && ((bytes[7] & 0x80) == (sign_extend & 0x80))
        {
            let mut buffer = [sign_extend; 8];
            buffer.copy_from_slice(&bytes[..8]);
            return Ok(i64::from_le_bytes(buffer));
        }

        return Err("integer exceeds i64 range".to_string());
    }

    let mut buffer = [sign_extend; 8];
    buffer[..bytes.len()].copy_from_slice(bytes);
    Ok(i64::from_le_bytes(buffer))
}

pub(crate) fn decode_signed_le_bytes_bigint(bytes: &[u8]) -> Result<BigInt, String> {
    if bytes.len() > MAX_INTEGER_SIZE {
        return Err("integer size exceeds maximum".to_string());
    }
    Ok(BigInt::from_signed_bytes_le(bytes))
}

pub(crate) struct ShiftValue(BigInt);

impl ShiftValue {
    pub(crate) fn shift_left(self, shift: u32) -> Result<StackValue, String> {
        numeric_result_bigint(self.0 << (shift as usize), "integer overflow for SHL")
    }

    pub(crate) fn shift_right(self, shift: u32) -> Result<StackValue, String> {
        numeric_result_bigint(self.0 >> (shift as usize), "integer overflow for SHR")
    }
}

#[inline]
pub(crate) fn pop_boolean(stack: &mut Vec<StackValue>) -> Result<bool, String> {
    match stack.pop() {
        Some(StackValue::Boolean(value)) => Ok(value),
        Some(StackValue::Integer(value)) => Ok(value != 0),
        Some(StackValue::BigInteger(value)) => Ok(value.iter().any(|byte| *byte != 0)),
        Some(StackValue::ByteString(value)) => Ok(value.iter().any(|byte| *byte != 0)),
        Some(StackValue::Buffer(_, _)) => Ok(true),
        Some(StackValue::Null) => Ok(false),
        Some(_) => Ok(true),
        None => Err("stack underflow".to_string()),
    }
}

/// Convert a StackValue to boolean via the integer path (NeoVM GetBoolean).
/// ByteString/BigInteger > 32 bytes will FAULT, matching NeoVM's MaxSize check.
pub(crate) fn item_to_boolean_strict(item: &StackValue) -> Result<bool, String> {
    match item {
        StackValue::Boolean(value) => Ok(*value),
        StackValue::Integer(value) => Ok(*value != 0),
        StackValue::BigInteger(value) => {
            if value.len() > MAX_INTEGER_SIZE {
                return Err("integer size exceeds maximum".to_string());
            }
            Ok(value.iter().any(|byte| *byte != 0))
        }
        StackValue::ByteString(value) => {
            if value.len() > MAX_INTEGER_SIZE {
                return Err("integer size exceeds maximum".to_string());
            }
            Ok(value.iter().any(|byte| *byte != 0))
        }
        StackValue::Buffer(_, _) => Ok(true),
        StackValue::Null => Ok(false),
        _ => Ok(true),
    }
}

pub(crate) fn mod_pow_bigint(
    base: BigInt,
    exponent: BigInt,
    modulus: BigInt,
) -> Result<BigInt, String> {
    if modulus.is_zero() {
        return Err("division by zero for MODPOW".to_string());
    }

    if exponent == BigInt::from(-1) {
        if base <= BigInt::zero() {
            return Err("value has no modular inverse".to_string());
        }
        if modulus <= BigInt::from(1) {
            return Err("invalid modulus for modular inverse".to_string());
        }
        return base
            .modinv(&modulus)
            .ok_or_else(|| "value is not invertible for MODPOW".to_string());
    }

    if exponent < BigInt::zero() {
        return Err("negative exponent for MODPOW".to_string());
    }

    let mut result = BigInt::from(1) % &modulus;
    let mut power = base % &modulus;
    let mut exponent = exponent;
    while exponent > BigInt::zero() {
        if (&exponent % 2u8) != BigInt::zero() {
            result = (result * &power) % &modulus;
        }
        exponent >>= 1usize;
        if exponent > BigInt::zero() {
            power = (&power * &power) % &modulus;
        }
    }
    Ok(result)
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

pub(crate) fn numeric_result_bigint(
    value: BigInt,
    overflow_message: &str,
) -> Result<StackValue, String> {
    let bytes = trim_le_bytes(value.to_signed_bytes_le());
    if bytes.len() > MAX_INTEGER_SIZE {
        return Err(overflow_message.to_string());
    }
    Ok(bigint_or_integer(bytes))
}

pub(crate) fn bigint_sign(value: &BigInt) -> i64 {
    if value.is_zero() {
        0
    } else if value < &BigInt::zero() {
        -1
    } else {
        1
    }
}

pub(crate) fn bigint_abs(value: BigInt) -> BigInt {
    if value < BigInt::zero() {
        -value
    } else {
        value
    }
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

pub(crate) fn trim_le_bytes(mut bytes: Vec<u8>) -> Vec<u8> {
    if bytes.is_empty() {
        return bytes;
    }

    let sign_extend = if bytes.last().is_some_and(|byte| byte & 0x80 != 0) {
        0xff
    } else {
        0x00
    };
    while bytes.len() > 1 && bytes.last() == Some(&sign_extend) {
        let next = bytes[bytes.len() - 2];
        if (next & 0x80 != 0) == (sign_extend == 0xff) {
            bytes.pop();
        } else {
            break;
        }
    }
    bytes
}

/// Convert LE two's complement bytes to i64.
/// Values that don't fit in i64 are truncated to the lower 8 bytes.
pub(crate) fn bytes_to_integer(bytes: &[u8]) -> i64 {
    if bytes.is_empty() {
        return 0;
    }

    // Determine sign from the last byte's high bit
    let negative = bytes.last().is_some_and(|b| b & 0x80 != 0);

    let mut buf = [0u8; 8];
    let copy_len = bytes.len().min(8);
    buf[..copy_len].copy_from_slice(&bytes[..copy_len]);

    // Sign-extend if negative
    if negative && copy_len < 8 {
        for b in &mut buf[copy_len..] {
            *b = 0xFF;
        }
    }

    i64::from_le_bytes(buf)
}

pub(crate) fn bitwise_result<F>(
    left: &StackValue,
    right: &StackValue,
    op: F,
) -> Result<StackValue, String>
where
    F: Fn(u8, u8) -> u8,
{
    let left_bytes = bitwise_operand_bytes(left)?;
    let right_bytes = bitwise_operand_bytes(right)?;
    Ok(bigint_or_integer(bitwise_signed_bytes(
        &left_bytes,
        &right_bytes,
        op,
    )?))
}

fn bitwise_operand_bytes(value: &StackValue) -> Result<Vec<u8>, String> {
    match value {
        StackValue::Integer(value) => Ok(encode_integer(*value)),
        StackValue::BigInteger(value) | StackValue::ByteString(value) => Ok(value.clone()),
        StackValue::Boolean(value) => Ok(encode_integer(if *value { 1 } else { 0 })),
        _ => Err("bitwise op expects primitive numeric or byte string operands".to_string()),
    }
}

pub(crate) fn bigint_or_integer(bytes: Vec<u8>) -> StackValue {
    let trimmed = trim_le_bytes(bytes);
    if trimmed.len() <= 8 {
        StackValue::Integer(bytes_to_integer(&trimmed))
    } else {
        StackValue::BigInteger(trimmed)
    }
}

fn bitwise_signed_bytes<F>(left: &[u8], right: &[u8], op: F) -> Result<Vec<u8>, String>
where
    F: Fn(u8, u8) -> u8,
{
    let len = left.len().max(right.len());
    if len == 0 {
        return Ok(vec![op(0, 0)]);
    }
    if len > 32 {
        return Err("bitwise operand exceeds supported size".to_string());
    }

    let left_fill = if left.last().is_some_and(|byte| byte & 0x80 != 0) {
        0xff
    } else {
        0x00
    };
    let right_fill = if right.last().is_some_and(|byte| byte & 0x80 != 0) {
        0xff
    } else {
        0x00
    };
    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        let lb = left.get(i).copied().unwrap_or(left_fill);
        let rb = right.get(i).copied().unwrap_or(right_fill);
        result.push(op(lb, rb));
    }
    Ok(result)
}

#[inline]
pub(crate) fn trim_le_bytes_slice(bytes: &[u8]) -> Vec<u8> {
    if bytes.is_empty() {
        return Vec::new();
    }

    let sign_extend = if bytes.last().is_some_and(|byte| byte & 0x80 != 0) {
        0xff
    } else {
        0x00
    };
    let mut end = bytes.len();
    while end > 1 && bytes[end - 1] == sign_extend {
        let next = bytes[end - 2];
        if (next & 0x80 != 0) == (sign_extend == 0xff) {
            end -= 1;
        } else {
            break;
        }
    }
    bytes[..end].to_vec()
}
