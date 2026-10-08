use super::super::helpers::{pop_item, primitive_key_equals, to_abi_value};
use super::super::opcodes::{PICKITEM, SETITEM};
use super::super::runtime_types::{CompoundIds, StackValue};
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use num_bigint::BigInt;
use num_traits::ToPrimitive;

pub(super) enum TypedItemOutcome {
    Read(StackValue),
    Write(StackValue),
    Catch(String),
}

fn require_primitive(value: &StackValue) -> Result<(), String> {
    match value {
        StackValue::Integer(_) | StackValue::Boolean(_) | StackValue::ByteString(_) => Ok(()),
        StackValue::BigInteger(bytes) if bytes.len() <= 32 => Ok(()),
        _ => Err("collection key or value must be a primitive type".to_string()),
    }
}

pub(super) fn typed_item_integer(value: &StackValue) -> Result<i32, String> {
    require_primitive(value)?;
    match value {
        StackValue::Integer(value) => {
            i32::try_from(*value).map_err(|_| "collection integer exceeds Int32 range".to_string())
        }
        StackValue::Boolean(value) => Ok(i32::from(*value)),
        StackValue::ByteString(bytes) | StackValue::BigInteger(bytes) => {
            if bytes.len() > 32 {
                return Err("integer size exceeds maximum".to_string());
            }
            BigInt::from_signed_bytes_le(bytes)
                .to_i32()
                .ok_or_else(|| "collection integer exceeds Int32 range".to_string())
        }
        _ => unreachable!("primitive type checked above"),
    }
}

fn typed_map_key(key: &StackValue) -> Result<(), String> {
    require_primitive(key)?;
    if matches!(key, StackValue::ByteString(bytes) if bytes.len() > 64) {
        return Err("map key exceeds maximum size".to_string());
    }
    Ok(())
}

fn typed_key_equal(left: &StackValue, right: &StackValue) -> bool {
    match (left, right) {
        (StackValue::Integer(left), StackValue::BigInteger(right))
        | (StackValue::BigInteger(right), StackValue::Integer(left)) => {
            BigInt::from(*left) == BigInt::from_signed_bytes_le(right)
        }
        (StackValue::BigInteger(left), StackValue::BigInteger(right)) => {
            BigInt::from_signed_bytes_le(left) == BigInt::from_signed_bytes_le(right)
        }
        _ => primitive_key_equals(left, right),
    }
}

// ByteString.ToString uses strict UTF-8, falling back to padded RFC 4648 Base64.
pub(super) fn typed_key_string(key: &StackValue) -> String {
    match key {
        StackValue::Integer(value) => value.to_string(),
        StackValue::BigInteger(bytes) => BigInt::from_signed_bytes_le(bytes).to_string(),
        StackValue::Boolean(true) => "True".to_string(),
        StackValue::Boolean(false) => "False".to_string(),
        StackValue::ByteString(bytes) => match core::str::from_utf8(bytes) {
            Ok(text) => format!("\"{text}\""),
            Err(_) => {
                const ALPHABET: &[u8] =
                    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
                let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
                for chunk in bytes.chunks(3) {
                    let a = chunk[0];
                    let b = chunk.get(1).copied().unwrap_or(0);
                    let c = chunk.get(2).copied().unwrap_or(0);
                    encoded.push(ALPHABET[(a >> 2) as usize] as char);
                    encoded.push(ALPHABET[(((a & 3) << 4) | (b >> 4)) as usize] as char);
                    encoded.push(if chunk.len() > 1 {
                        ALPHABET[(((b & 15) << 2) | (c >> 6)) as usize] as char
                    } else {
                        '='
                    });
                    encoded.push(if chunk.len() > 2 {
                        ALPHABET[(c & 63) as usize] as char
                    } else {
                        '='
                    });
                }
                format!("\"Base64: {encoded}\"")
            }
        },
        _ => unreachable!("map primitive key checked before formatting"),
    }
}

fn index_out_of_range(kind: &str, index: i32, count: usize) -> TypedItemOutcome {
    TypedItemOutcome::Catch(format!(
        "The index of {kind} is out of range, {index}/[0, {count})."
    ))
}

pub(super) fn typed_item(
    opcode: u8,
    stack: &mut Vec<StackValue>,
    ids: &mut CompoundIds,
) -> Result<TypedItemOutcome, String> {
    let value = if opcode == SETITEM {
        Some(ids.clone_struct_for_storage(&pop_item(stack)?))
    } else {
        None
    };
    let key = pop_item(stack)?;
    require_primitive(&key)?;
    let item = pop_item(stack)?;
    let item_is_struct = matches!(&item, StackValue::Struct(..));
    match item {
        StackValue::Map(id, mut items) => {
            typed_map_key(&key)?;
            let index = items
                .iter()
                .position(|(candidate, _)| typed_key_equal(candidate, &key));
            if let Some(value) = value {
                if let Some(index) = index {
                    items[index].1 = value;
                } else {
                    items.push((key, value));
                }
                Ok(TypedItemOutcome::Write(StackValue::Map(id, items)))
            } else if let Some(index) = index {
                Ok(TypedItemOutcome::Read(items[index].1.clone()))
            } else {
                Ok(TypedItemOutcome::Catch(format!(
                    "Key {} not found in Map.",
                    typed_key_string(&key)
                )))
            }
        }
        StackValue::Array(id, mut items) | StackValue::Struct(id, mut items) => {
            let index = typed_item_integer(&key)?;
            if index < 0 || index as usize >= items.len() {
                return Ok(index_out_of_range("VMArray", index, items.len()));
            }
            if let Some(value) = value {
                items[index as usize] = value;
                let updated = if item_is_struct {
                    StackValue::Struct(id, items)
                } else {
                    StackValue::Array(id, items)
                };
                Ok(TypedItemOutcome::Write(updated))
            } else {
                Ok(TypedItemOutcome::Read(items[index as usize].clone()))
            }
        }
        StackValue::Buffer(id, mut bytes) => {
            let index = typed_item_integer(&key)?;
            if index < 0 || index as usize >= bytes.len() {
                return Ok(index_out_of_range("Buffer", index, bytes.len()));
            }
            if let Some(value) = value {
                let value = typed_item_integer(&value)?;
                if !(-128..=255).contains(&value) {
                    return Err(format!("Overflow in SETITEM, {value} is not a byte type."));
                }
                bytes[index as usize] = value as u8;
                Ok(TypedItemOutcome::Write(StackValue::Buffer(id, bytes)))
            } else {
                Ok(TypedItemOutcome::Read(StackValue::Integer(i64::from(
                    bytes[index as usize],
                ))))
            }
        }
        item @ (StackValue::ByteString(_)
        | StackValue::Integer(_)
        | StackValue::BigInteger(_)
        | StackValue::Boolean(_))
            if opcode == PICKITEM =>
        {
            let bytes = match &item {
                StackValue::BigInteger(bytes) => {
                    let integer = BigInt::from_signed_bytes_le(bytes);
                    if integer == BigInt::from(0) {
                        Vec::new()
                    } else {
                        integer.to_signed_bytes_le()
                    }
                }
                _ => to_abi_value(&item)
                    .to_byte_string_bytes()
                    .expect("primitive has a byte span"),
            };
            let index = typed_item_integer(&key)?;
            if index < 0 || index as usize >= bytes.len() {
                return Ok(index_out_of_range("PrimitiveType", index, bytes.len()));
            }
            Ok(TypedItemOutcome::Read(StackValue::Integer(i64::from(
                bytes[index as usize],
            ))))
        }
        _ => Err("invalid collection type for item operation".to_string()),
    }
}
