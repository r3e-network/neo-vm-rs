use super::super::runtime_types::{CompoundIds, StackValue};
use super::super::state::CallStack;
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn push_frame(
    calls: &mut CallStack,
    return_ip: usize,
    locals: &mut Vec<StackValue>,
    args: &mut Vec<StackValue>,
    initialized: bool,
    reference_mode: bool,
    base_depth: usize,
) -> Result<(), String> {
    if reference_mode {
        if base_depth
            .checked_add(calls.len())
            .is_none_or(|depth| depth >= crate::DEFAULT_MAX_INVOCATION_DEPTH as usize)
        {
            return Err("invocation depth exceeds maximum".to_string());
        }
        return calls.push_shared(
            return_ip,
            core::mem::take(locals),
            core::mem::take(args),
            initialized,
        );
    }
    #[cfg(target_arch = "riscv32")]
    {
        calls.push_frame_refs(return_ip, locals, args, initialized)?;
        // Legacy PolkaVM retains frames in its static buffer before an allocator reset.
        unsafe {
            core::ptr::write(locals, Vec::new());
            core::ptr::write(args, Vec::new());
        }
        Ok(())
    }
    #[cfg(not(target_arch = "riscv32"))]
    calls.push_frame(
        return_ip,
        core::mem::take(locals),
        core::mem::take(args),
        initialized,
    )
}

pub(super) fn push_reference_pointer(
    script: &[u8],
    ip: usize,
    stack: &mut Vec<StackValue>,
    ids: &CompoundIds,
) -> Result<usize, String> {
    let bytes = script
        .get(ip + 1..ip + 5)
        .ok_or_else(|| "truncated PUSHA operand".to_string())?;
    let offset = i32::from_le_bytes(bytes.try_into().unwrap()) as i64;
    let target = ip as i64 + offset;
    if target < 0 || target as usize > script.len() {
        return Err("PUSHA target out of bounds".to_string());
    }
    stack.push(ids.reference_pointer(target as usize));
    Ok(ip + 5)
}

pub(super) fn pointer_target(
    value: &StackValue,
    ids: &CompoundIds,
    reference_mode: bool,
) -> Result<usize, String> {
    if reference_mode {
        return ids.reference_pointer_target(value);
    }
    match value {
        StackValue::Pointer(position) => Ok(*position),
        _ => Err("CALLA expects a pointer".to_string()),
    }
}
