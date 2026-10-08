//! Compound instructions for the opt-in shared arena. No legacy tree projections occur here.
use super::super::helpers::*;
use super::super::opcodes::*;
use super::super::runtime_types::{CompoundIds, StackValue, export_shared};
use super::super::state::{MAX_STACK_SIZE, PendingException};
use super::control::Dispatch;
use super::typed_item_ops::{typed_item_integer, typed_key_string};
use crate::reference::{SharedHeap, SharedValue};
use crate::{StackItemType, StackValue as Abi, new_array_default_value_for_neovm_type_tag};
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

fn pop(stack: &mut Vec<StackValue>, heap: &SharedHeap) -> Result<SharedValue, String> {
    export_shared(&pop_item(stack)?, heap)
}
fn push(
    stack: &mut Vec<StackValue>,
    ids: &mut CompoundIds,
    value: SharedValue,
) -> Result<(), String> {
    stack.push(ids.import_shared(value)?);
    Ok(())
}
fn count(stack: &mut Vec<StackValue>) -> Result<usize, String> {
    let n = pop_integer(stack)?;
    usize::try_from(n)
        .ok()
        .filter(|n| *n <= MAX_STACK_SIZE)
        .ok_or_else(|| "collection count out of range".to_string())
}
fn primitive_span(value: &SharedValue) -> Result<Vec<u8>, String> {
    let primitive = value
        .primitive_ref()
        .ok_or_else(|| "expected primitive".to_string())?;
    match primitive {
        Abi::Integer(_) | Abi::BigInteger(_) | Abi::ByteString(_) | Abi::Boolean(_) => {
            crate::stack_value_span_bytes(primitive)
                .ok_or_else(|| "primitive has no span".to_string())
        }
        _ => Err("expected primitive".to_string()),
    }
}
fn missing(item: &SharedValue, key: &StackValue) -> Result<String, String> {
    if item.kind() == StackItemType::Map {
        Ok(format!("Key {} not found in Map.", typed_key_string(key)))
    } else {
        let kind = match item.kind() {
            StackItemType::Array | StackItemType::Struct => "VMArray",
            StackItemType::Buffer => "Buffer",
            _ => "PrimitiveType",
        };
        Ok(format!(
            "The index of {kind} is out of range, {}/[0, {}).",
            typed_item_integer(key)?,
            item.len()?
        ))
    }
}

pub(super) fn execute(
    opcode: u8,
    script: &[u8],
    ip: &mut usize,
    stack: &mut Vec<StackValue>,
    ids: &mut CompoundIds,
    pending: &mut Option<PendingException>,
) -> Result<Dispatch, String> {
    let heap = ids.shared_heap().expect("reference dispatch").clone();
    match opcode {
        PACK | PACKSTRUCT => {
            let n = count(stack)?;
            if n > stack.len() {
                return Err("stack underflow for PACK".to_string());
            }
            let mut items = Vec::with_capacity(n);
            for _ in 0..n {
                items.push(pop(stack, &heap)?);
            }
            // PACK and PACKSTRUCT retain original Struct references.
            let result = if opcode == PACK {
                heap.array(items)?
            } else {
                heap.structure(items)?
            };
            push(stack, ids, result)?;
        }
        PACKMAP => {
            let n = count(stack)?;
            if n.saturating_mul(2) > stack.len() {
                return Err("stack underflow for PACKMAP".to_string());
            }
            let mut pairs = Vec::with_capacity(n);
            for _ in 0..n {
                let key = pop(stack, &heap)?;
                let value = pop(stack, &heap)?;
                pairs.push((key, value));
            }
            push(stack, ids, heap.map(pairs)?)?;
        }
        UNPACK => {
            let value = pop(stack, &heap)?;
            let n = value.len()?;
            if value.kind() == StackItemType::Map {
                for (k, v) in value.map_items()?.into_iter().rev() {
                    push(stack, ids, v)?;
                    push(stack, ids, k)?;
                }
            } else {
                for item in value.items()?.into_iter().rev() {
                    push(stack, ids, item)?;
                }
            }
            stack.push(StackValue::Integer(n as i64));
        }
        NEWARRAY0 | NEWSTRUCT0 => {
            let result = if opcode == NEWARRAY0 {
                heap.array(Vec::new())?
            } else {
                heap.structure(Vec::new())?
            };
            push(stack, ids, result)?;
        }
        NEWARRAY | NEWSTRUCT | NEWARRAY_T => {
            let n = count(stack)?;
            let default = if opcode == NEWARRAY_T {
                let tag = *script
                    .get(*ip + 1)
                    .ok_or_else(|| "truncated NEWARRAY_T".to_string())?;
                // Native only permits defined StackItemType tags.
                StackItemType::from_byte(tag)
                    .ok_or_else(|| "invalid NEWARRAY_T type".to_string())?;
                heap.import_abi(if tag == StackItemType::Boolean as u8 {
                    Abi::Boolean(false)
                } else {
                    new_array_default_value_for_neovm_type_tag(tag)
                })?
            } else {
                SharedValue::primitive(Abi::Null)?
            };
            let result = if opcode == NEWSTRUCT {
                heap.structure(vec![default; n])?
            } else {
                heap.array(vec![default; n])?
            };
            push(stack, ids, result)?;
            if opcode == NEWARRAY_T {
                *ip += 2;
                return Ok(Dispatch::Continue);
            }
        }
        NEWMAP => push(stack, ids, heap.map(Vec::new())?)?,
        SIZE => {
            let item = pop(stack, &heap)?;
            stack.push(StackValue::Integer(item.len()? as i64));
        }
        HASKEY => {
            let key = pop_item(stack)?;
            let shared_key = export_shared(&key, &heap)?;
            let item = pop(stack, &heap)?;
            let exists = if item.kind() == StackItemType::Map {
                item.item(&shared_key)?.is_some()
            } else {
                if !matches!(
                    item.kind(),
                    StackItemType::Array
                        | StackItemType::Struct
                        | StackItemType::Buffer
                        | StackItemType::ByteString
                ) {
                    return Err("invalid HASKEY collection".to_string());
                }
                let index = typed_item_integer(&key)?;
                if index < 0 {
                    return Err("negative collection index".to_string());
                }
                (index as usize) < item.len()?
            };
            stack.push(StackValue::Boolean(exists));
        }
        KEYS | VALUES => {
            let item = pop(stack, &heap)?;
            let mut values = if item.kind() == StackItemType::Map {
                item.map_items()?
                    .into_iter()
                    .map(|(k, v)| if opcode == KEYS { k } else { v })
                    .collect::<Vec<_>>()
            } else if opcode == VALUES {
                item.items()?
            } else {
                return Err("KEYS expects Map".to_string());
            };
            if opcode == VALUES {
                for value in &mut values {
                    *value = value.clone_struct(MAX_STACK_SIZE)?;
                }
            }
            push(stack, ids, heap.array(values)?)?;
        }
        APPEND => {
            let value = pop(stack, &heap)?;
            let item = pop(stack, &heap)?;
            if !matches!(item.kind(), StackItemType::Array | StackItemType::Struct) {
                return Err("APPEND expects Array or Struct".to_string());
            }
            item.append_raw(&value.clone_struct(MAX_STACK_SIZE)?)?;
        }
        PICKITEM | SETITEM => {
            let value = if opcode == SETITEM {
                Some(pop(stack, &heap)?.clone_struct(MAX_STACK_SIZE)?)
            } else {
                None
            };
            let key = pop_item(stack)?;
            let shared_key = export_shared(&key, &heap)?;
            let item = pop(stack, &heap)?;
            let found = if let Some(value) = value {
                item.set_item_raw(&shared_key, &value)?
            } else {
                let result = if item.primitive_ref().is_some() {
                    let bytes = primitive_span(&item)?;
                    let index = typed_item_integer(&key)?;
                    usize::try_from(index)
                        .ok()
                        .and_then(|i| bytes.get(i))
                        .map(|b| {
                            SharedValue::primitive(Abi::Integer(i64::from(*b))).expect("integer")
                        })
                } else {
                    item.item(&shared_key)?
                };
                if let Some(result) = result {
                    push(stack, ids, result)?;
                    true
                } else {
                    false
                }
            };
            if !found {
                *pending = Some(PendingException::thrown_value(StackValue::ByteString(
                    missing(&item, &key)?.into_bytes(),
                )));
                return Ok(Dispatch::Continue);
            }
        }
        REMOVE => {
            let key = pop(stack, &heap)?;
            let item = pop(stack, &heap)?;
            if !item.remove_item(&key)? && item.kind() != StackItemType::Map {
                return Err("REMOVE index out of range".to_string());
            }
        }
        CLEARITEMS => pop(stack, &heap)?.clear()?,
        REVERSEITEMS => pop(stack, &heap)?.reverse()?,
        POPITEM => {
            let item = pop(stack, &heap)?;
            push(
                stack,
                ids,
                item.pop()?
                    .ok_or_else(|| "POPITEM on empty array".to_string())?,
            )?;
        }
        ISNULL => {
            let item = pop_item(stack)?;
            stack.push(StackValue::Boolean(is_null(&item)));
        }
        ISTYPE | CONVERT => {
            let kind = *script
                .get(*ip + 1)
                .ok_or_else(|| "truncated type operand".to_string())?;
            let item = pop_item(stack)?;
            if opcode == ISTYPE {
                stack.push(StackValue::Boolean(is_type(kind, &item)?));
            } else {
                stack.push(convert_value(kind, item, ids)?);
            }
            *ip += 2;
            return Ok(Dispatch::Continue);
        }
        _ => {
            return Err(format!(
                "unexpected reference collection opcode 0x{opcode:02x}"
            ));
        }
    }
    Ok(Dispatch::Fallthrough)
}
