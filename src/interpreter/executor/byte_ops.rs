use super::super::helpers;
use super::super::helpers::{pop_bytes, pop_integer, pop_item};
use super::super::opcodes::*;
use super::super::runtime_types::{propagate_update, CompoundIds, StackValue};
use super::super::state::remember_consumed_mutation;
use super::control::Dispatch;
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
    stack: &mut Vec<StackValue>,
    ids: &mut CompoundIds,
    locals: &mut [StackValue],
    args: &mut [StackValue],
    static_fields: &mut [StackValue],
    consumed_mutations: &mut Vec<StackValue>,
) -> Result<Dispatch, String> {
    match opcode {
        // SPLICE OPERATIONS (0x88-0x8e)
        // =============================================================================
        CAT => {
            let right_item = pop_item(stack)?;
            let left_item = pop_item(stack)?;
            let left_bytes = helpers::stack_item_to_bytes(left_item)?;
            let right_bytes = helpers::stack_item_to_bytes(right_item)?;
            let mut result_bytes = left_bytes;
            result_bytes.extend_from_slice(&right_bytes);
            // NeoVM: CAT result must not exceed max item size (1024*1024)
            const MAX_ITEM_SIZE: usize = 1024 * 1024;
            if result_bytes.len() > MAX_ITEM_SIZE {
                return Err("CAT result exceeds max item size".to_string());
            }
            // NeoVM: CAT always produces a Buffer
            stack.push(ids.buffer(result_bytes));
        }
        LEFT => {
            let count = pop_integer(stack)?;
            if count < 0 {
                return Err("negative count for LEFT".to_string());
            }
            let bytes = pop_bytes(stack)?;
            let count = count as usize;
            if count > bytes.len() {
                return Err("count out of range for LEFT".to_string());
            }
            // NeoVM splice operations materialize a mutable Buffer result.
            stack.push(ids.buffer(bytes[..count].to_vec()));
        }
        NEWBUFFER => {
            let count = pop_integer(stack)?;
            if count < 0 {
                return Err("negative count for NEWBUFFER".to_string());
            }
            if count > 1_048_576 {
                return Err("buffer size exceeds MaxItemSize (1MB)".to_string());
            }
            stack.push(ids.buffer(vec![0u8; count as usize]));
        }
        RIGHT => {
            let count = pop_integer(stack)?;
            if count < 0 {
                return Err("negative count for RIGHT".to_string());
            }
            let count = count as usize;
            let mut bytes = pop_bytes(stack)?;
            if count > bytes.len() {
                return Err("count out of range for RIGHT".to_string());
            }
            let start = bytes.len() - count;
            bytes = bytes[start..].to_vec();
            stack.push(ids.buffer(bytes));
        }
        SUBSTR => {
            let count = pop_integer(stack)?;
            let index = pop_integer(stack)?;
            if count < 0 {
                return Err("negative count for SUBSTR".to_string());
            }
            if index < 0 {
                return Err("negative index for SUBSTR".to_string());
            }
            let index = index as usize;
            let count = count as usize;
            let bytes = pop_bytes(stack)?;
            // NeoVM reference: error if index + count > length (NOT index > length)
            let end = index
                .checked_add(count)
                .ok_or_else(|| "SUBSTR index+count overflow".to_string())?;
            if end > bytes.len() {
                return Err("index + count out of range for SUBSTR".to_string());
            }
            stack.push(ids.buffer(bytes[index..end].to_vec()));
        }
        MEMCPY => {
            // NeoVM MEMCPY: stack = [dst, di, src, si, count] (count on top)
            let count = pop_integer(stack)?;
            let si = pop_integer(stack)?;
            let src_item = pop_item(stack)?;
            let di = pop_integer(stack)?;
            let dst_item = pop_item(stack)?;
            if count < 0 || si < 0 || di < 0 {
                return Err("negative index/count for MEMCPY".to_string());
            }
            let count = count as usize;
            let si = si as usize;
            let di = di as usize;
            let src_bytes = helpers::stack_item_to_bytes(src_item)?;
            let (dst_id, mut dst_bytes) = match dst_item {
                StackValue::Buffer(id, bytes) => (id, bytes),
                other => {
                    return Err(format!(
                        "MEMCPY expects buffer as destination, got {:?}",
                        other
                    ))
                }
            };
            if si + count > src_bytes.len() || di + count > dst_bytes.len() {
                return Err("MEMCPY out of bounds".to_string());
            }
            dst_bytes[di..di + count].copy_from_slice(&src_bytes[si..si + count]);
            let updated = StackValue::Buffer(dst_id, dst_bytes);
            remember_consumed_mutation(consumed_mutations, &updated);
            propagate_update(&updated, stack, locals, args, static_fields, None);
        }
        _ => unreachable!("opcode routed to byte_ops: 0x{opcode:02x}"),
    }
    Ok(Dispatch::Fallthrough)
}
