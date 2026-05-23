use super::super::helpers::*;
use super::super::opcodes::*;
use super::super::runtime_types::{
    find_affected_indices, propagate_update, CompoundIds, StackValue,
};
use super::super::state::{remember_consumed_mutation, PendingException, TryStack, MAX_STACK_SIZE};
use super::control::Dispatch;
use crate::{
    NEOVM_STACK_ITEM_TYPE_ANY, NEOVM_STACK_ITEM_TYPE_ARRAY, NEOVM_STACK_ITEM_TYPE_BOOLEAN,
    NEOVM_STACK_ITEM_TYPE_BUFFER, NEOVM_STACK_ITEM_TYPE_BYTESTRING, NEOVM_STACK_ITEM_TYPE_INTEGER,
    NEOVM_STACK_ITEM_TYPE_INTEROP_INTERFACE, NEOVM_STACK_ITEM_TYPE_MAP,
    NEOVM_STACK_ITEM_TYPE_POINTER, NEOVM_STACK_ITEM_TYPE_STRUCT,
};
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

#[allow(clippy::too_many_arguments)]
#[inline]
pub(super) fn execute(
    opcode: u8,
    script: &[u8],
    ip_ref: &mut usize,
    stack: &mut Vec<StackValue>,
    ids: &mut CompoundIds,
    locals: &mut [StackValue],
    args: &mut [StackValue],
    static_fields: &mut [StackValue],
    consumed_mutations: &mut Vec<StackValue>,
    try_frames: &mut TryStack,
    pending_error: &mut Option<PendingException>,
) -> Result<Dispatch, String> {
    let mut ip = *ip_ref;
    macro_rules! finish {
        ($dispatch:expr) => {{
            *ip_ref = ip;
            return Ok($dispatch);
        }};
    }

    match opcode {
        // COMPOUND TYPE OPERATIONS (0xbe-0xd3)
        // =============================================================================
        PACKMAP => {
            let count = pop_integer(stack)?;
            if count < 0 {
                return Err("negative count for PACKMAP".to_string());
            }
            let count = count as usize;
            if stack.len() < count.saturating_mul(2) {
                return Err("stack underflow for PACKMAP".to_string());
            }

            let mut pairs = Vec::with_capacity(count);
            for _ in 0..count {
                let key = pop_item(stack)?;
                let value = pop_item(stack)?;
                pairs.push((key, value));
            }
            stack.push(ids.map(pairs));
        }
        PACKSTRUCT => {
            let count = pop_integer(stack)?;
            if count < 0 {
                return Err("negative count for PACKSTRUCT".to_string());
            }
            let count = count as usize;
            if stack.len() < count {
                return Err("stack underflow for PACKSTRUCT".to_string());
            }

            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                items.push(pop_item(stack)?);
            }
            let items = items
                .into_iter()
                .map(|item| ids.clone_struct_for_storage(&item))
                .collect();
            stack.push(ids.r#struct(items));
        }
        PACK => {
            let count = pop_integer(stack)?;
            if count < 0 {
                return Err("negative count for PACK".to_string());
            }
            let count = count as usize;
            if stack.len() < count {
                return Err("stack underflow for PACK".to_string());
            }

            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                items.push(pop_item(stack)?);
            }
            let items = items
                .into_iter()
                .map(|item| ids.clone_struct_for_storage(&item))
                .collect();
            stack.push(ids.array(items));
        }
        UNPACK => {
            let item = pop_item(stack)?;
            match item {
                StackValue::Array(_, items) | StackValue::Struct(_, items) => {
                    let count = items.len() as i64;
                    for item in items.into_iter().rev() {
                        stack.push(item);
                    }
                    stack.push(StackValue::Integer(count));
                }
                StackValue::Map(_, items) => {
                    let count = items.len() as i64;
                    for (key, value) in items.into_iter().rev() {
                        stack.push(value);
                        stack.push(key);
                    }
                    stack.push(StackValue::Integer(count));
                }
                _ => return Err("UNPACK expects an array, struct, or map".to_string()),
            }
        }
        NEWARRAY0 => {
            stack.push(ids.array(Vec::new()));
        }
        NEWARRAY => {
            let count = pop_integer(stack)?;
            if count < 0 {
                return Err("negative count for NEWARRAY".to_string());
            }
            if count > MAX_STACK_SIZE as i64 {
                return Err("NEWARRAY count exceeds maximum stack size".to_string());
            }
            stack.push(ids.array(vec![StackValue::Null; count as usize]));
        }
        NEWARRAY_T => {
            if ip + 2 > script.len() {
                return Err("truncated NEWARRAY_T type".to_string());
            }
            let count = pop_integer(stack)?;
            if count < 0 {
                return Err("negative count for NEWARRAY_T".to_string());
            }
            if count > MAX_STACK_SIZE as i64 {
                return Err("NEWARRAY_T count exceeds maximum stack size".to_string());
            }
            let kind = script[ip + 1];
            let default_value = match kind {
                NEOVM_STACK_ITEM_TYPE_INTEGER => StackValue::Integer(0),
                NEOVM_STACK_ITEM_TYPE_BYTESTRING => StackValue::ByteString(Vec::new()),
                _ => StackValue::Null,
            };
            stack.push(ids.array(vec![default_value; count as usize]));
            ip += 2;
            finish!(Dispatch::Continue);
        }
        NEWSTRUCT0 => {
            stack.push(ids.r#struct(Vec::new()));
        }
        NEWSTRUCT => {
            let count = pop_integer(stack)?;
            if count < 0 {
                return Err("negative count for NEWSTRUCT".to_string());
            }
            if count > MAX_STACK_SIZE as i64 {
                return Err("NEWSTRUCT count exceeds maximum stack size".to_string());
            }
            stack.push(ids.r#struct(vec![StackValue::Null; count as usize]));
        }
        NEWMAP => {
            stack.push(ids.map(Vec::new()));
        }
        SIZE => {
            let item = pop_item(stack)?;
            let size = match item {
                StackValue::ByteString(bytes) => bytes.len() as i64,
                StackValue::Integer(value) => encode_integer(value).len() as i64,
                StackValue::BigInteger(bytes) => bytes.len() as i64,
                StackValue::Boolean(_) => 1,
                StackValue::Array(_, items) => items.len() as i64,
                StackValue::Struct(_, items) => items.len() as i64,
                StackValue::Map(_, items) => items.len() as i64,
                StackValue::Buffer(_, bytes) => bytes.len() as i64,
                StackValue::Null
                | StackValue::Pointer(_)
                | StackValue::Interop(_)
                | StackValue::Iterator(_) => return Err("SIZE expects a collection".to_string()),
            };
            stack.push(StackValue::Integer(size));
        }
        HASKEY => {
            let key = pop_item(stack)?;
            let item = pop_item(stack)?;
            let has_key = match item {
                StackValue::ByteString(bytes) => {
                    let index = integer_value_for_collection_index(&key)?;
                    index >= 0 && (index as usize) < bytes.len()
                }
                StackValue::Buffer(_, bytes) => {
                    let index = integer_value_for_collection_index(&key)?;
                    index >= 0 && (index as usize) < bytes.len()
                }
                StackValue::Array(_, items) => {
                    let index = integer_value_for_collection_index(&key)?;
                    index >= 0 && (index as usize) < items.len()
                }
                StackValue::Struct(_, items) => {
                    let index = integer_value_for_collection_index(&key)?;
                    index >= 0 && (index as usize) < items.len()
                }
                StackValue::Map(_, items) => {
                    validate_map_key(&key)?;
                    items
                        .iter()
                        .any(|(candidate, _)| primitive_key_equals(candidate, &key))
                }
                _ => return Err("HASKEY expects an array, buffer, or map".to_string()),
            };
            stack.push(StackValue::Boolean(has_key));
        }
        KEYS => {
            let item = pop_item(stack)?;
            match item {
                StackValue::Map(_, items) => {
                    let len = items.len();
                    let keys: Vec<_> = {
                        let mut v = Vec::with_capacity(len);
                        v.extend(items.into_iter().map(|(key, _)| key));
                        v
                    };
                    stack.push(ids.array(keys));
                }
                _ => return Err("KEYS expects a map".to_string()),
            }
        }
        VALUES => {
            let item = pop_item(stack)?;
            match item {
                StackValue::Map(_, items) => {
                    let values = items
                        .into_iter()
                        .map(|(_, value)| value)
                        .collect::<Vec<_>>();
                    stack.push(ids.array(values));
                }
                StackValue::Array(_, items) => {
                    stack.push(ids.array(items));
                }
                StackValue::Struct(_, items) => {
                    stack.push(ids.array(items));
                }
                _ => return Err("VALUES expects a map, array, or struct".to_string()),
            }
        }
        APPEND => {
            let value = pop_item(stack)?;
            let item = pop_item(stack)?;
            match item {
                StackValue::Array(id, mut items) => {
                    items.push(ids.clone_struct_for_storage(&value));
                    let updated = StackValue::Array(id, items);
                    remember_consumed_mutation(consumed_mutations, &updated);
                    let affected = find_affected_indices(id, stack);
                    propagate_update(
                        &updated,
                        stack,
                        locals,
                        args,
                        static_fields,
                        Some(&affected),
                    );
                }
                StackValue::Struct(id, mut items) => {
                    items.push(ids.clone_struct_for_storage(&value));
                    let updated = StackValue::Struct(id, items);
                    remember_consumed_mutation(consumed_mutations, &updated);
                    let affected = find_affected_indices(id, stack);
                    propagate_update(
                        &updated,
                        stack,
                        locals,
                        args,
                        static_fields,
                        Some(&affected),
                    );
                }
                _ => return Err("APPEND expects an array or struct".to_string()),
            }
        }
        PICKITEM => {
            let pick_result =
                (|| -> Result<(), String> {
                    let key_or_index = pop_item(stack)?;
                    let item = pop_item(stack)?;
                    match item {
                        StackValue::Map(_, items) => {
                            // Map key can be any primitive type
                            validate_map_key(&key_or_index)?;
                            let value = items
                                .iter()
                                .find(|(candidate, _)| {
                                    primitive_key_equals(candidate, &key_or_index)
                                })
                                .map(|(_, value)| value.clone())
                                .ok_or_else(|| "key not found for PICKITEM".to_string())?;
                            stack.push(value);
                        }
                        _ => {
                            // Array, Struct, Buffer, ByteString and Integer-like values:
                            // key must be an integer index. NeoVM treats Integer as its
                            // little-endian signed byte representation for PICKITEM; old
                            // mainnet contracts rely on this for integer payload routing.
                            let index = match key_or_index {
                                StackValue::Integer(v) if v >= 0 => v as usize,
                                StackValue::Boolean(v) => {
                                    if v {
                                        1
                                    } else {
                                        0
                                    }
                                }
                                StackValue::Null => 0,
                                _ => {
                                    return Err(
                                        "PICKITEM index must be a non-negative integer".to_string()
                                    )
                                }
                            };
                            match item {
                                StackValue::Array(_, items) | StackValue::Struct(_, items) => {
                                    let value = items.get(index).cloned().ok_or_else(|| {
                                        "index out of range for PICKITEM".to_string()
                                    })?;
                                    if cfg!(target_arch = "riscv32") {
                                        core::mem::forget(items);
                                    }
                                    stack.push(value);
                                }
                                StackValue::Buffer(_, bytes) => {
                                    let value = bytes.get(index).copied().ok_or_else(|| {
                                        "index out of range for PICKITEM".to_string()
                                    })?;
                                    stack.push(StackValue::Integer(i64::from(value)));
                                }
                                StackValue::ByteString(bytes) => {
                                    let value = bytes.get(index).copied().ok_or_else(|| {
                                        "index out of range for PICKITEM".to_string()
                                    })?;
                                    stack.push(StackValue::Integer(i64::from(value)));
                                }
                                StackValue::Integer(value) => {
                                    let bytes = encode_integer(value);
                                    let value = bytes.get(index).copied().ok_or_else(|| {
                                        "index out of range for PICKITEM".to_string()
                                    })?;
                                    stack.push(StackValue::Integer(i64::from(value)));
                                }
                                StackValue::Boolean(value) => {
                                    let bytes = [u8::from(value)];
                                    let value = bytes.get(index).copied().ok_or_else(|| {
                                        "index out of range for PICKITEM".to_string()
                                    })?;
                                    stack.push(StackValue::Integer(i64::from(value)));
                                }
                                StackValue::BigInteger(bytes) => {
                                    let value = bytes.get(index).copied().ok_or_else(|| {
                                        "index out of range for PICKITEM".to_string()
                                    })?;
                                    stack.push(StackValue::Integer(i64::from(value)));
                                }
                                _ => {
                                    return Err(
                                        "PICKITEM expects an array, map, byte string, or integer"
                                            .to_string(),
                                    )
                                }
                            }
                        }
                    }
                    Ok(())
                })();
            if let Err(error) = pick_result {
                if try_frames.is_empty() {
                    return Err(error);
                }
                *pending_error = Some(PendingException::message(error));
                finish!(Dispatch::Continue);
            }
        }
        SETITEM => {
            let value = pop_item(stack)?;
            let key = pop_item(stack)?;
            let item = pop_item(stack)?;
            match item {
                StackValue::ByteString(_) => {
                    return Err(
                        "SETITEM expects a mutable buffer, array, struct, or map".to_string()
                    )
                }
                StackValue::Buffer(id, mut bytes) => {
                    let index = integer_value_for_collection_index(&key)?;
                    if index < 0 || (index as usize) >= bytes.len() {
                        return Err("index out of range for SETITEM".to_string());
                    }
                    let byte = match value {
                        StackValue::Integer(value) if (-128..=255).contains(&value) => value as u8,
                        StackValue::ByteString(value) if value.len() == 1 => value[0],
                        _ => return Err("SETITEM on buffer expects a byte value".to_string()),
                    };
                    bytes[index as usize] = byte;
                    let updated = StackValue::Buffer(id, bytes);
                    remember_consumed_mutation(consumed_mutations, &updated);
                    let affected = find_affected_indices(id, stack);
                    propagate_update(
                        &updated,
                        stack,
                        locals,
                        args,
                        static_fields,
                        Some(&affected),
                    );
                }
                StackValue::Array(id, mut items) => {
                    let index = integer_value_for_collection_index(&key)?;
                    if index < 0 || (index as usize) >= items.len() {
                        return Err("index out of range for SETITEM".to_string());
                    }
                    items[index as usize] = ids.clone_struct_for_storage(&value);
                    let updated = StackValue::Array(id, items);
                    remember_consumed_mutation(consumed_mutations, &updated);
                    let affected = find_affected_indices(id, stack);
                    propagate_update(
                        &updated,
                        stack,
                        locals,
                        args,
                        static_fields,
                        Some(&affected),
                    );
                }
                StackValue::Struct(id, mut items) => {
                    let index = integer_value_for_collection_index(&key)?;
                    if index < 0 || (index as usize) >= items.len() {
                        return Err("index out of range for SETITEM".to_string());
                    }
                    items[index as usize] = ids.clone_struct_for_storage(&value);
                    let updated = StackValue::Struct(id, items);
                    remember_consumed_mutation(consumed_mutations, &updated);
                    let affected = find_affected_indices(id, stack);
                    propagate_update(
                        &updated,
                        stack,
                        locals,
                        args,
                        static_fields,
                        Some(&affected),
                    );
                }
                StackValue::Map(id, mut items) => {
                    validate_map_key(&key)?;
                    if let Some(index) = items
                        .iter()
                        .position(|(candidate, _)| primitive_key_equals(candidate, &key))
                    {
                        items[index].1 = ids.clone_struct_for_storage(&value);
                    } else {
                        let mut updated_items = Vec::with_capacity(items.len() + 1);
                        updated_items.extend(items);
                        updated_items.push((key, ids.clone_struct_for_storage(&value)));
                        items = updated_items;
                    }
                    let updated = StackValue::Map(id, items);
                    remember_consumed_mutation(consumed_mutations, &updated);
                    let affected = find_affected_indices(id, stack);
                    propagate_update(
                        &updated,
                        stack,
                        locals,
                        args,
                        static_fields,
                        Some(&affected),
                    );
                }
                _ => return Err("SETITEM expects an array, buffer, or map".to_string()),
            }
        }
        REMOVE => {
            let key = pop_item(stack)?;
            let item = pop_item(stack)?;
            match item {
                StackValue::Array(id, mut items) => {
                    let index = integer_value_for_collection_index(&key)?;
                    if index < 0 || (index as usize) >= items.len() {
                        return Err("index out of range for REMOVE".to_string());
                    }
                    items.remove(index as usize);
                    let updated = StackValue::Array(id, items);
                    remember_consumed_mutation(consumed_mutations, &updated);
                    let affected = find_affected_indices(id, stack);
                    propagate_update(
                        &updated,
                        stack,
                        locals,
                        args,
                        static_fields,
                        Some(&affected),
                    );
                }
                StackValue::Struct(id, mut items) => {
                    let index = integer_value_for_collection_index(&key)?;
                    if index < 0 || (index as usize) >= items.len() {
                        return Err("index out of range for REMOVE".to_string());
                    }
                    items.remove(index as usize);
                    let updated = StackValue::Struct(id, items);
                    remember_consumed_mutation(consumed_mutations, &updated);
                    let affected = find_affected_indices(id, stack);
                    propagate_update(
                        &updated,
                        stack,
                        locals,
                        args,
                        static_fields,
                        Some(&affected),
                    );
                }
                StackValue::Map(id, mut items) => {
                    validate_map_key(&key)?;
                    let index = items
                        .iter()
                        .position(|(candidate, _)| primitive_key_equals(candidate, &key))
                        .ok_or_else(|| "key not found for REMOVE".to_string())?;
                    items.remove(index);
                    let updated = StackValue::Map(id, items);
                    remember_consumed_mutation(consumed_mutations, &updated);
                    let affected = find_affected_indices(id, stack);
                    propagate_update(
                        &updated,
                        stack,
                        locals,
                        args,
                        static_fields,
                        Some(&affected),
                    );
                }
                _ => return Err("REMOVE expects an array, struct, or map".to_string()),
            }
        }
        CLEARITEMS => {
            let item = pop_item(stack)?;
            match item {
                StackValue::Array(id, _) => {
                    let updated = StackValue::Array(id, Vec::new());
                    remember_consumed_mutation(consumed_mutations, &updated);
                    propagate_update(&updated, stack, locals, args, static_fields, None);
                }
                StackValue::Struct(id, _) => {
                    let updated = StackValue::Struct(id, Vec::new());
                    remember_consumed_mutation(consumed_mutations, &updated);
                    propagate_update(&updated, stack, locals, args, static_fields, None);
                }
                StackValue::Map(id, _) => {
                    let updated = StackValue::Map(id, Vec::new());
                    remember_consumed_mutation(consumed_mutations, &updated);
                    propagate_update(&updated, stack, locals, args, static_fields, None);
                }
                StackValue::Buffer(id, _) => {
                    let updated = StackValue::Buffer(id, Vec::new());
                    remember_consumed_mutation(consumed_mutations, &updated);
                    propagate_update(&updated, stack, locals, args, static_fields, None);
                }
                _ => return Err("CLEARITEMS expects a compound value".to_string()),
            }
        }
        POPITEM => {
            let item = pop_item(stack)?;
            match item {
                StackValue::Array(id, mut items) => {
                    let popped = items
                        .pop()
                        .ok_or_else(|| "POPITEM on empty array".to_string())?;
                    let updated = StackValue::Array(id, items);
                    remember_consumed_mutation(consumed_mutations, &updated);
                    propagate_update(&updated, stack, locals, args, static_fields, None);
                    stack.push(popped);
                }
                StackValue::Struct(id, mut items) => {
                    let popped = items
                        .pop()
                        .ok_or_else(|| "POPITEM on empty struct".to_string())?;
                    let updated = StackValue::Struct(id, items);
                    remember_consumed_mutation(consumed_mutations, &updated);
                    propagate_update(&updated, stack, locals, args, static_fields, None);
                    stack.push(popped);
                }
                StackValue::Map(id, mut entries) => {
                    let (key, value) = entries
                        .pop()
                        .ok_or_else(|| "POPITEM on empty map".to_string())?;
                    let updated = StackValue::Map(id, entries);
                    remember_consumed_mutation(consumed_mutations, &updated);
                    propagate_update(&updated, stack, locals, args, static_fields, None);
                    stack.push(key);
                    stack.push(value);
                }
                StackValue::Buffer(id, mut bytes) => {
                    let byte = bytes
                        .pop()
                        .ok_or_else(|| "POPITEM on empty buffer".to_string())?;
                    let updated = StackValue::Buffer(id, bytes);
                    remember_consumed_mutation(consumed_mutations, &updated);
                    propagate_update(&updated, stack, locals, args, static_fields, None);
                    stack.push(StackValue::Integer(byte as i64));
                }
                _ => return Err("POPITEM expects a compound value".to_string()),
            }
        }
        CONVERT => {
            if ip + 2 > script.len() {
                return Err("truncated CONVERT operand".to_string());
            }
            let kind = script[ip + 1];
            let value = pop_item(stack)?;
            let converted = convert_value(kind, value, ids)?;
            stack.push(converted);
            ip += 2;
            finish!(Dispatch::Continue);
        }
        REVERSEITEMS => {
            let item = pop_item(stack)?;
            match item {
                StackValue::Array(id, mut items) => {
                    items.reverse();
                    let updated = StackValue::Array(id, items);
                    remember_consumed_mutation(consumed_mutations, &updated);
                    propagate_update(&updated, stack, locals, args, static_fields, None);
                }
                StackValue::Struct(id, mut items) => {
                    items.reverse();
                    let updated = StackValue::Struct(id, items);
                    remember_consumed_mutation(consumed_mutations, &updated);
                    propagate_update(&updated, stack, locals, args, static_fields, None);
                }
                StackValue::Buffer(id, mut bytes) => {
                    bytes.reverse();
                    let updated = StackValue::Buffer(id, bytes);
                    remember_consumed_mutation(consumed_mutations, &updated);
                    propagate_update(&updated, stack, locals, args, static_fields, None);
                }
                _ => {
                    return Err(format!(
                        "REVERSEITEMS expects an array, struct, or buffer at ip {ip}: {item:?}"
                    ));
                }
            }
        }
        // =============================================================================
        // TYPE OPERATIONS (0xd9, 0xdb)
        // =============================================================================
        ISTYPE => {
            if ip + 2 > script.len() {
                return Err("truncated ISTYPE operand".to_string());
            }
            let kind = script[ip + 1];
            let item = pop_item(stack)?;
            let result = match kind {
                NEOVM_STACK_ITEM_TYPE_ANY => {
                    return Err(format!(
                        "unsupported ISTYPE kind {NEOVM_STACK_ITEM_TYPE_ANY:#04x}"
                    ))
                }
                NEOVM_STACK_ITEM_TYPE_POINTER => matches!(item, StackValue::Pointer(_)),
                NEOVM_STACK_ITEM_TYPE_BOOLEAN => matches!(item, StackValue::Boolean(_)),
                NEOVM_STACK_ITEM_TYPE_INTEGER => {
                    matches!(item, StackValue::Integer(_) | StackValue::BigInteger(_))
                }
                NEOVM_STACK_ITEM_TYPE_BYTESTRING => matches!(item, StackValue::ByteString(_)),
                NEOVM_STACK_ITEM_TYPE_BUFFER => matches!(item, StackValue::Buffer(_, _)),
                NEOVM_STACK_ITEM_TYPE_ARRAY => matches!(item, StackValue::Array(_, _)),
                NEOVM_STACK_ITEM_TYPE_STRUCT => matches!(item, StackValue::Struct(_, _)),
                NEOVM_STACK_ITEM_TYPE_MAP => matches!(item, StackValue::Map(_, _)),
                NEOVM_STACK_ITEM_TYPE_INTEROP_INTERFACE => {
                    matches!(item, StackValue::Interop(_))
                }
                _ => return Err(format!("unsupported ISTYPE kind 0x{kind:02x}")),
            };
            stack.push(StackValue::Boolean(result));
            ip += 2;
            finish!(Dispatch::Continue);
        }
        ISNULL => {
            let item = pop_item(stack)?;
            stack.push(StackValue::Boolean(matches!(item, StackValue::Null)));
        }
        _ => unreachable!("opcode routed to compound_ops: 0x{opcode:02x}"),
    }
    *ip_ref = ip;
    Ok(Dispatch::Fallthrough)
}
