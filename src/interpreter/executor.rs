use super::api::SyscallProvider;
mod byte_ops;
mod compound_ops;
mod control;
mod numeric_ops;
mod push_ops;
mod result_ops;
mod slot_ops;
mod stack_ops;
use super::helpers::*;
use super::opcodes::*;
use super::runtime_types::{to_abi_stack, to_abi_value, CompoundIds, StackValue};
use super::state::*;
use crate::{
    semantics::comparison as comparison_rules, ExecutionResult, StackValue as AbiStackValue,
    VmState,
};
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::sync::atomic::Ordering;

pub(super) fn interpret_with_stack_and_syscalls_at_internal<H: SyscallProvider>(
    script: &[u8],
    initial_stack: Vec<AbiStackValue>,
    initial_ip: usize,
    initializer_ip: Option<usize>,
    result_stack_limit: Option<usize>,
    host: &mut H,
) -> Result<ExecutionResult, String> {
    LAST_INTERPRETER_IP.store(u32::MAX, Ordering::Relaxed);
    LAST_RESULT_STAGE.store(0, Ordering::Relaxed);
    LAST_RESULT_STACK_LEN.store(0, Ordering::Relaxed);
    LAST_RESULT_LIMIT.store(u32::MAX, Ordering::Relaxed);

    if initial_ip > script.len() {
        return Err("initial instruction pointer out of bounds".to_string());
    }
    if initializer_ip.is_some_and(|ip| ip > script.len()) {
        return Err("initializer instruction pointer out of bounds".to_string());
    }
    let mut ids = CompoundIds::default();
    let mut method_initial_stack = initial_stack
        .into_iter()
        .map(|item| ids.import_abi(item))
        .collect::<Vec<_>>();
    let mut running_initializer = initializer_ip.is_some();
    let retained_initializer_method_stack_len = if running_initializer {
        retain_initializer_method_stack(&method_initial_stack)?
    } else {
        None
    };
    let mut stack = if running_initializer {
        Vec::new()
    } else {
        core::mem::take(&mut method_initial_stack)
    };
    let mut ip = initializer_ip.unwrap_or(initial_ip);
    let mut locals: Vec<StackValue> = Vec::with_capacity(16);
    let mut args: Vec<StackValue> = Vec::with_capacity(16);
    let mut static_fields: Vec<StackValue> = Vec::with_capacity(16);
    let mut slots_initialized = false;
    let mut static_fields_initialized = false;
    let mut try_frames = TryStack::new();
    let mut call_stack = CallStack::new();
    let mut consumed_mutations: Vec<StackValue> = Vec::with_capacity(16);
    let mut pending_error: Option<PendingException> = None;
    let mut previous_opcode_was_callt = false;

    macro_rules! apply_dispatch {
        ($dispatch:expr) => {
            if let control::Dispatch::Continue = $dispatch {
                continue;
            }
        };
    }

    'main_loop: loop {
        if pending_error.is_some() {
            // Find the topmost un-caught frame
            if let Some(frame_index) = try_frames.find_uncaught_index() {
                let frame_call_depth = try_frames
                    .get_mut(frame_index)
                    .map(|frame| frame.call_depth)
                    .unwrap_or(call_stack.len());
                while call_stack.len() > frame_call_depth {
                    if let Some((_, mut saved_locals, mut saved_args, saved_init)) =
                        call_stack.pop_and_restore()?
                    {
                        let current_mutations = core::mem::take(&mut consumed_mutations);
                        propagate_active_aliases_into_saved_frame(
                            &mut saved_locals,
                            &mut saved_args,
                            &stack,
                            &locals,
                            &args,
                            &static_fields,
                            &current_mutations,
                        );
                        locals = saved_locals;
                        args = saved_args;
                        reset_consumed_mutations(&mut consumed_mutations);
                        slots_initialized = saved_init;
                    }
                }
                while try_frames.len > frame_index + 1 {
                    try_frames.pop();
                }
                let frame = try_frames
                    .get_mut(frame_index)
                    .ok_or_else(|| "missing try frame during exception unwind".to_string())?;
                frame.caught = true;
                // Save frame fields into locals BEFORE any allocating operations.
                // Under PolkaVM's bump allocator, host-call round-trips can reset the
                // allocator offset, causing subsequent allocations to overwrite the
                // TryFrame backing buffer.  Copying into stack locals avoids the stale read.
                let saved_catch_ip = frame.catch_ip;
                let saved_finally_ip = frame.finally_ip;
                // NeoVM: error goes to catch first, then finally
                let pending = pending_error.take().ok_or_else(|| {
                    "missing pending exception during exception unwind".to_string()
                })?;
                if saved_catch_ip != 0 {
                    stack.push(pending.into_catch_item());
                    ip = saved_catch_ip;
                } else if saved_finally_ip != 0 {
                    frame.in_finally = true;
                    ip = saved_finally_ip;
                    // Keep pending_error for re-throw after ENDFINALLY
                    pending_error = Some(pending);
                } else {
                    continue;
                }
                continue;
            } else {
                return Err(pending_error
                    .take()
                    .ok_or_else(|| "missing pending exception without try frame".to_string())?
                    .into_fault_message());
            }
        }

        // Stack overflow check: NeoVM faults when stack exceeds 2048 items.
        if stack.len() > MAX_STACK_SIZE {
            return Err("stack overflow".to_string());
        }

        if ip >= script.len() {
            break;
        }

        let opcode = script[ip];
        record_interpreter_ip(ip);
        host.on_instruction(opcode)?;
        let callt_result_pending_store = previous_opcode_was_callt;
        previous_opcode_was_callt = false;
        match opcode {
            PUSHINT8
            | PUSHINT16
            | PUSHINT32
            | PUSHINT64
            | PUSHINT128
            | PUSHINT256
            | PUSHT
            | PUSHF
            | PUSHNULL
            | PUSHDATA1
            | PUSHDATA2
            | PUSHDATA4
            | PUSHM1
            | PUSH0
            | PUSH1..=PUSH16
            | PUSHA => {
                apply_dispatch!(push_ops::execute(opcode, script, &mut ip, &mut stack)?);
            }
            // FLOW CONTROL OPCODES (0x21-0x40)
            JMP | JMP_L => {
                let is_long = opcode == JMP_L;
                let (offset, _advance) = read_offset(
                    script,
                    ip,
                    if is_long {
                        &Offset::Long
                    } else {
                        &Offset::Short
                    },
                    "JMP",
                )?;
                ip = compute_jump_target_offset(ip, offset, script.len(), "JMP")?;
                continue;
            }
            JMPIF | JMPIF_L => {
                let is_long = opcode == JMPIF_L;
                let (offset, advance) = read_offset(
                    script,
                    ip,
                    if is_long {
                        &Offset::Long
                    } else {
                        &Offset::Short
                    },
                    "JMPIF",
                )?;
                let condition = pop_boolean(&mut stack)?;
                if condition {
                    ip = compute_jump_target_offset(ip, offset, script.len(), "JMPIF")?;
                    continue;
                }
                ip += advance;
                continue;
            }
            JMPEQ | JMPEQ_L => {
                let is_long = opcode == JMPEQ_L;
                let (offset, advance) = read_offset(
                    script,
                    ip,
                    if is_long {
                        &Offset::Long
                    } else {
                        &Offset::Short
                    },
                    "JMPEQ",
                )?;
                let right = pop_item(&mut stack)?;
                let left = pop_item(&mut stack)?;
                if vm_equal(&left, &right) {
                    ip = compute_jump_target_offset(ip, offset, script.len(), "JMPEQ")?;
                    continue;
                }
                ip += advance;
                continue;
            }
            ABORT => {
                // ABORT is uncatchable — always FAULT. Return via the Ok(Fault) channel
                // so we can attribute the faulting IP; the host converts this back to an
                // Err(String) at the FFI boundary (lib.rs fault-message override).
                return Ok(ExecutionResult {
                    fee_consumed_pico: 0,
                    state: VmState::Fault,
                    stack: to_abi_stack(&stack),
                    fault_message: Some("ABORT".to_string()),
                    fault_ip: Some(ip as u32),
                    fault_locals: {
                        let cloned = locals.clone();
                        Some(crate::abi::fast_codec::encode_stack(&to_abi_stack(&cloned)))
                    },
                });
            }
            ASSERT => {
                let value = pop_boolean(&mut stack)?;
                if !value {
                    // ASSERT is uncatchable — always FAULT. Same Ok(Fault) channel as ABORT
                    // so the faulting IP is preserved.
                    return Ok(ExecutionResult {
                        fee_consumed_pico: 0,
                        state: VmState::Fault,
                        stack: to_abi_stack(&stack),
                        fault_message: Some("ASSERT failed".to_string()),
                        fault_ip: Some(ip as u32),
                        fault_locals: {
                            let cloned = locals.clone();
                            Some(crate::abi::fast_codec::encode_stack(&to_abi_stack(&cloned)))
                        },
                    });
                }
            }
            SYSCALL => {
                if ip + 5 > script.len() {
                    return Err("truncated SYSCALL operand".to_string());
                }

                let api = u32::from_le_bytes([
                    script[ip + 1],
                    script[ip + 2],
                    script[ip + 3],
                    script[ip + 4],
                ]);
                if let Err(e) = invoke_syscall(
                    host,
                    api,
                    ip,
                    &mut stack,
                    &mut args,
                    &mut locals,
                    &mut static_fields,
                    &mut consumed_mutations,
                    &mut ids,
                ) {
                    if try_frames.is_empty() {
                        return Err(e);
                    }
                    pending_error = Some(PendingException::message(e));
                    continue;
                }
                ip += 5;
                continue;
            }
            CALLT => {
                if ip + 3 > script.len() {
                    return Err("truncated CALLT operand".to_string());
                }
                let token = u16::from_le_bytes([script[ip + 1], script[ip + 2]]);
                if let Err(e) = invoke_callt(
                    host,
                    token,
                    ip,
                    &mut stack,
                    &mut args,
                    &mut locals,
                    &mut static_fields,
                    &mut consumed_mutations,
                    &mut ids,
                ) {
                    if try_frames.is_empty() {
                        return Err(e);
                    }
                    pending_error = Some(PendingException::message(e));
                    continue;
                }
                previous_opcode_was_callt = true;
                ip += 3;
                continue;
            }
            NOP => {}
            DEPTH | DROP | DUP | SWAP => {
                apply_dispatch!(stack_ops::execute(opcode, &mut stack)?);
            }
            INITSSLOT
            | INITSLOT
            | LDSFLD0..=LDSFLD6
            | LDSFLD
            | LDLOC0..=LDLOC6
            | LDLOC
            | STLOC0..=STLOC6
            | STSFLD0..=STSFLD6
            | STSFLD
            | STLOC => {
                apply_dispatch!(slot_ops::execute(
                    opcode,
                    script,
                    &mut ip,
                    &mut stack,
                    &mut locals,
                    &mut args,
                    &mut static_fields,
                    &mut slots_initialized,
                    &mut static_fields_initialized,
                    &mut ids,
                    callt_result_pending_store,
                )?);
            }
            CAT | LEFT | NEWBUFFER => {
                apply_dispatch!(byte_ops::execute(
                    opcode,
                    &mut stack,
                    &mut ids,
                    &mut locals,
                    &mut args,
                    &mut static_fields,
                    &mut consumed_mutations,
                )?);
            }
            INVERT | AND | OR | XOR | NUMEQUAL | NUMNOTEQUAL | EQUAL | NOTEQUAL | LT | LE | GT
            | GE | SIGN | ABS | NEGATE | ADD | INC | SUB | POW | SQRT | MODMUL | MODPOW | SHL
            | SHR | NOT => {
                apply_dispatch!(numeric_ops::execute(opcode, &mut stack)?);
            }
            PACKMAP | PACKSTRUCT | PACK | UNPACK | NEWARRAY0 | NEWARRAY | NEWARRAY_T
            | NEWSTRUCT0 | NEWSTRUCT | NEWMAP | SIZE | HASKEY | KEYS | VALUES | APPEND
            | PICKITEM | SETITEM | REMOVE | CLEARITEMS | POPITEM | CONVERT | REVERSEITEMS => {
                apply_dispatch!(compound_ops::execute(
                    opcode,
                    script,
                    &mut ip,
                    &mut stack,
                    &mut ids,
                    &mut locals,
                    &mut args,
                    &mut static_fields,
                    &mut consumed_mutations,
                    &mut try_frames,
                    &mut pending_error,
                )?);
            }
            // EXCEPTION HANDLING OPCODES
            THROW => {
                let msg = pop_item(&mut stack)?;
                let err_msg = match &msg {
                    StackValue::ByteString(bytes) | StackValue::Buffer(_, bytes) => {
                        match alloc::string::String::from_utf8(bytes.clone()) {
                            Ok(text) => format!("THROW: {text}"),
                            Err(_) => format!("THROW: {:?}", msg),
                        }
                    }
                    _ => format!("THROW: {:?}", msg),
                };
                if try_frames.is_empty() {
                    return Ok(ExecutionResult {
                        fee_consumed_pico: 0,
                        state: VmState::Fault,
                        stack: to_abi_stack(&stack),
                        fault_message: Some(err_msg),
                        fault_ip: Some(ip as u32),
                        fault_locals: {
                            let cloned = locals.clone();
                            Some(crate::abi::fast_codec::encode_stack(&to_abi_stack(&cloned)))
                        },
                    });
                }
                pending_error = Some(PendingException::thrown_value(msg));
                continue;
            }
            MUL | DIV | MOD | DEC | BOOLAND | BOOLOR | NZ | MIN | MAX | WITHIN => {
                apply_dispatch!(numeric_ops::execute(opcode, &mut stack)?);
            }
            RIGHT | SUBSTR | MEMCPY => {
                apply_dispatch!(byte_ops::execute(
                    opcode,
                    &mut stack,
                    &mut ids,
                    &mut locals,
                    &mut args,
                    &mut static_fields,
                    &mut consumed_mutations,
                )?);
            }
            ISTYPE | ISNULL => {
                apply_dispatch!(compound_ops::execute(
                    opcode,
                    script,
                    &mut ip,
                    &mut stack,
                    &mut ids,
                    &mut locals,
                    &mut args,
                    &mut static_fields,
                    &mut consumed_mutations,
                    &mut try_frames,
                    &mut pending_error,
                )?);
            }
            NIP | OVER | PICK | ROT | ROLL | REVERSE3 | REVERSE4 | REVERSEN | TUCK | XDROP => {
                apply_dispatch!(stack_ops::execute(opcode, &mut stack)?);
            }
            LDARG0..=LDARG6 | LDARG | STARG0..=STARG6 | STARG => {
                apply_dispatch!(slot_ops::execute(
                    opcode,
                    script,
                    &mut ip,
                    &mut stack,
                    &mut locals,
                    &mut args,
                    &mut static_fields,
                    &mut slots_initialized,
                    &mut static_fields_initialized,
                    &mut ids,
                    callt_result_pending_store,
                )?);
            }
            CLEAR => {
                apply_dispatch!(stack_ops::execute(opcode, &mut stack)?);
            }
            CALL | CALL_L => {
                let is_long = opcode == CALL_L;
                let (offset, advance) = read_offset(
                    script,
                    ip,
                    if is_long {
                        &Offset::Long
                    } else {
                        &Offset::Short
                    },
                    "CALL",
                )?;
                let return_ip = ip + advance;
                let saved_init = slots_initialized;
                reset_consumed_mutations(&mut consumed_mutations);
                #[cfg(target_arch = "riscv32")]
                {
                    call_stack.push_frame_refs(return_ip, &locals, &args, saved_init)?;
                    // The frame has been encoded into RETAINED_CALL_STACK_BUF. Avoid
                    // moving or dropping active Vecs immediately after host calls on
                    // PolkaVM/riscv32; the VM allocator is reset per execution.
                    unsafe {
                        core::ptr::write(&mut locals, Vec::new());
                        core::ptr::write(&mut args, Vec::new());
                    }
                }
                #[cfg(not(target_arch = "riscv32"))]
                {
                    let saved_locals = core::mem::take(&mut locals);
                    let saved_args = core::mem::take(&mut args);
                    call_stack.push_frame(return_ip, saved_locals, saved_args, saved_init)?;
                }
                slots_initialized = false;
                ip = compute_jump_target_offset(ip, offset, script.len(), "CALL")?;
                continue;
            }
            CALLA => {
                let item = pop_item(&mut stack)?;
                let offset = match item {
                    StackValue::Pointer(p) => p,
                    _ => return Err("CALLA expects a pointer".to_string()),
                };
                let return_ip = ip + 1;
                let saved_init = slots_initialized;
                reset_consumed_mutations(&mut consumed_mutations);
                #[cfg(target_arch = "riscv32")]
                {
                    call_stack.push_frame_refs(return_ip, &locals, &args, saved_init)?;
                    unsafe {
                        core::ptr::write(&mut locals, Vec::new());
                        core::ptr::write(&mut args, Vec::new());
                    }
                }
                #[cfg(not(target_arch = "riscv32"))]
                {
                    let saved_locals = core::mem::take(&mut locals);
                    let saved_args = core::mem::take(&mut args);
                    call_stack.push_frame(return_ip, saved_locals, saved_args, saved_init)?;
                }
                slots_initialized = false;
                if offset > script.len() {
                    return Err("CALLA target out of bounds".to_string());
                }
                ip = offset;
                continue;
            }
            ABORTMSG => {
                let msg = pop_bytes(&mut stack)?;
                let msg_str = String::from_utf8_lossy(&msg);
                let err_msg = format!("ABORTMSG is executed. Reason: {}", msg_str);
                // ABORTMSG is uncatchable — always FAULT
                return Ok(ExecutionResult {
                    fee_consumed_pico: 0,
                    state: VmState::Fault,
                    stack: to_abi_stack(&stack),
                    fault_message: Some(err_msg),
                    fault_ip: Some(ip as u32),
                    fault_locals: {
                        let cloned = locals.clone();
                        Some(crate::abi::fast_codec::encode_stack(&to_abi_stack(&cloned)))
                    },
                });
            }
            ASSERTMSG => {
                let msg = pop_bytes(&mut stack)?;
                let condition = pop_boolean(&mut stack)?;
                if !condition {
                    let msg_str = String::from_utf8_lossy(&msg);
                    let err_msg = format!(
                        "ASSERTMSG is executed with false result. Reason: {}",
                        msg_str
                    );
                    // ASSERTMSG is uncatchable — always FAULT
                    return Ok(ExecutionResult {
                        fee_consumed_pico: 0,
                        state: VmState::Fault,
                        stack: to_abi_stack(&stack),
                        fault_message: Some(err_msg),
                        fault_ip: Some(ip as u32),
                        fault_locals: {
                            let cloned = locals.clone();
                            Some(crate::abi::fast_codec::encode_stack(&to_abi_stack(&cloned)))
                        },
                    });
                }
            }
            JMPIFNOT | JMPIFNOT_L => {
                let is_long = opcode == JMPIFNOT_L;
                let (offset, advance) = read_offset(
                    script,
                    ip,
                    if is_long {
                        &Offset::Long
                    } else {
                        &Offset::Short
                    },
                    "JMPIFNOT",
                )?;
                let condition = pop_boolean(&mut stack)?;
                if !condition {
                    ip = compute_jump_target_offset(ip, offset, script.len(), "JMPIFNOT")?;
                    continue;
                }
                ip += advance;
                continue;
            }
            JMPNE | JMPNE_L => {
                let is_long = opcode == JMPNE_L;
                let (offset, advance) = read_offset(
                    script,
                    ip,
                    if is_long {
                        &Offset::Long
                    } else {
                        &Offset::Short
                    },
                    "JMPNE",
                )?;
                let right = pop_item(&mut stack)?;
                let left = pop_item(&mut stack)?;
                if !vm_equal(&left, &right) {
                    ip = compute_jump_target_offset(ip, offset, script.len(), "JMPNE")?;
                    continue;
                }
                ip += advance;
                continue;
            }
            JMPGT | JMPGT_L => {
                let is_long = opcode == JMPGT_L;
                let (offset, advance) = read_offset(
                    script,
                    ip,
                    if is_long {
                        &Offset::Long
                    } else {
                        &Offset::Short
                    },
                    "JMPGT",
                )?;
                if pop_jump_comparison(&mut stack, comparison_rules::greater_than_values)? {
                    ip = compute_jump_target_offset(ip, offset, script.len(), "JMPGT")?;
                    continue;
                }
                ip += advance;
                continue;
            }
            JMPGE | JMPGE_L => {
                let is_long = opcode == JMPGE_L;
                let (offset, advance) = read_offset(
                    script,
                    ip,
                    if is_long {
                        &Offset::Long
                    } else {
                        &Offset::Short
                    },
                    "JMPGE",
                )?;
                if pop_jump_comparison(&mut stack, comparison_rules::greater_or_equal_values)? {
                    ip = compute_jump_target_offset(ip, offset, script.len(), "JMPGE")?;
                    continue;
                }
                ip += advance;
                continue;
            }
            JMPLT | JMPLT_L => {
                let is_long = opcode == JMPLT_L;
                let (offset, advance) = read_offset(
                    script,
                    ip,
                    if is_long {
                        &Offset::Long
                    } else {
                        &Offset::Short
                    },
                    "JMPLT",
                )?;
                if pop_jump_comparison(&mut stack, comparison_rules::less_than_values)? {
                    ip = compute_jump_target_offset(ip, offset, script.len(), "JMPLT")?;
                    continue;
                }
                ip += advance;
                continue;
            }
            JMPLE | JMPLE_L => {
                let is_long = opcode == JMPLE_L;
                let (offset, advance) = read_offset(
                    script,
                    ip,
                    if is_long {
                        &Offset::Long
                    } else {
                        &Offset::Short
                    },
                    "JMPLE",
                )?;
                if pop_jump_comparison(&mut stack, comparison_rules::less_or_equal_values)? {
                    ip = compute_jump_target_offset(ip, offset, script.len(), "JMPLE")?;
                    continue;
                }
                ip += advance;
                continue;
            }
            RET => {
                if let Some((return_ip, mut saved_locals, mut saved_args, saved_init)) =
                    call_stack.pop_and_restore()?
                {
                    let current_mutations = core::mem::take(&mut consumed_mutations);
                    propagate_active_aliases_into_saved_frame(
                        &mut saved_locals,
                        &mut saved_args,
                        &stack,
                        &locals,
                        &args,
                        &static_fields,
                        &current_mutations,
                    );
                    locals = saved_locals;
                    args = saved_args;
                    reset_consumed_mutations(&mut consumed_mutations);
                    slots_initialized = saved_init;
                    while try_frames
                        .last_mut()
                        .is_some_and(|frame| frame.call_depth > call_stack.len())
                    {
                        try_frames.pop();
                    }
                    ip = return_ip;
                    continue;
                }
                if running_initializer {
                    restore_initializer_method_stack(
                        &mut method_initial_stack,
                        retained_initializer_method_stack_len,
                    )?;
                    complete_initializer_retaining_state(
                        host,
                        ip,
                        &mut method_initial_stack,
                        &mut static_fields,
                    )?;
                    stack = core::mem::take(&mut method_initial_stack);
                    ip = initial_ip;
                    locals = Vec::with_capacity(16);
                    args = Vec::with_capacity(16);
                    slots_initialized = false;
                    try_frames = TryStack::new();
                    call_stack = CallStack::new();
                    pending_error = None;
                    previous_opcode_was_callt = false;
                    running_initializer = false;
                    reset_consumed_mutations(&mut consumed_mutations);
                    continue;
                }
                break 'main_loop;
            }
            TRY => {
                // NeoVM TRY: TRY catch_offset_i8, finally_offset_i8 (3 bytes total)
                if ip + 3 > script.len() {
                    pending_error = Some(PendingException::message(
                        "truncated TRY operand".to_string(),
                    ));
                    continue;
                }
                let catch_offset = script[ip + 1] as i8;
                let finally_offset = script[ip + 2] as i8;
                let catch_ip = if catch_offset != 0 {
                    (ip as isize + catch_offset as isize) as usize
                } else {
                    0
                };
                let finally_ip = if finally_offset != 0 {
                    (ip as isize + finally_offset as isize) as usize
                } else {
                    0
                };
                if catch_ip > script.len() || finally_ip > script.len() {
                    pending_error = Some(PendingException::message(
                        "TRY target out of bounds".to_string(),
                    ));
                    continue;
                }
                try_frames.push(TryFrame {
                    catch_ip,
                    finally_ip,
                    call_depth: call_stack.len(),
                    caught: false,
                    in_finally: false,
                    end_ip: 0,
                })?;
                ip += 3;
                continue;
            }
            ENDTRY => {
                // ENDTRY offset_i8: end of try/catch block
                if ip + 2 > script.len() {
                    pending_error = Some(PendingException::message(
                        "truncated ENDTRY operand".to_string(),
                    ));
                    continue;
                }
                let offset = script[ip + 1] as i8;
                let target_ip = if offset != 0 {
                    (ip as isize + offset as isize) as usize
                } else {
                    ip + 2
                };
                if let Some(frame) = try_frames.last_mut() {
                    if frame.finally_ip != 0 && !frame.in_finally {
                        // Save continuation IP for ENDFINALLY to use
                        frame.end_ip = target_ip;
                        frame.in_finally = true;
                        ip = frame.finally_ip;
                        continue;
                    }
                }
                // No finally or already in finally — pop frame and jump
                try_frames.pop();
                ip = target_ip;
                continue;
            }
            TRY_L => {
                // TRY_L: long-form TRY with i32 catch_offset, i32 finally_offset (9 bytes)
                if ip + 9 > script.len() {
                    pending_error = Some(PendingException::message(
                        "truncated TRY_L operand".to_string(),
                    ));
                    continue;
                }
                let catch_offset = i32::from_le_bytes([
                    script[ip + 1],
                    script[ip + 2],
                    script[ip + 3],
                    script[ip + 4],
                ]);
                let finally_offset = i32::from_le_bytes([
                    script[ip + 5],
                    script[ip + 6],
                    script[ip + 7],
                    script[ip + 8],
                ]);
                let catch_ip = if catch_offset != 0 {
                    (ip as isize + catch_offset as isize) as usize
                } else {
                    0
                };
                let finally_ip = if finally_offset != 0 {
                    (ip as isize + finally_offset as isize) as usize
                } else {
                    0
                };
                if catch_ip > script.len() || finally_ip > script.len() {
                    pending_error = Some(PendingException::message(
                        "TRY_L target out of bounds".to_string(),
                    ));
                    continue;
                }
                try_frames.push(TryFrame {
                    catch_ip,
                    finally_ip,
                    call_depth: call_stack.len(),
                    caught: false,
                    in_finally: false,
                    end_ip: 0,
                })?;
                ip += 9;
                continue;
            }
            ENDTRY_L => {
                // ENDTRY_L offset_i32: long-form ENDTRY (5 bytes)
                if ip + 5 > script.len() {
                    pending_error = Some(PendingException::message(
                        "truncated ENDTRY_L operand".to_string(),
                    ));
                    continue;
                }
                let offset = i32::from_le_bytes([
                    script[ip + 1],
                    script[ip + 2],
                    script[ip + 3],
                    script[ip + 4],
                ]);
                let target_ip = if offset != 0 {
                    (ip as isize + offset as isize) as usize
                } else {
                    ip + 5
                };
                if let Some(frame) = try_frames.last_mut() {
                    if frame.finally_ip != 0 && !frame.in_finally {
                        frame.end_ip = target_ip;
                        frame.in_finally = true;
                        ip = frame.finally_ip;
                        continue;
                    }
                }
                try_frames.pop();
                ip = target_ip;
                continue;
            }
            ENDFINALLY => {
                // ENDFINALLY must be reached via the finally path
                let in_finally = try_frames.last_mut().is_some_and(|f| f.in_finally);
                if in_finally {
                    let frame = try_frames.pop().unwrap();
                    if pending_error.is_some() {
                        // Re-throw: pending_error will be processed at top of loop
                        continue;
                    }
                    // Normal completion after finally — jump to saved end_ip
                    if frame.end_ip != 0 {
                        ip = frame.end_ip;
                        continue;
                    }
                } else {
                    // ENDFINALLY without being in finally state = FAULT
                    return Err("ENDFINALLY without matching finally context".to_string());
                }
            }
            other => {
                return Err(format!("unsupported opcode 0x{other:02x}"));
            }
        }
        ip += 1;
    }

    LAST_RESULT_STAGE.store(1, Ordering::Relaxed);
    LAST_RESULT_STACK_LEN.store(stack.len().min(u32::MAX as usize) as u32, Ordering::Relaxed);
    LAST_RESULT_LIMIT.store(
        result_stack_limit
            .unwrap_or(usize::MAX)
            .min(u32::MAX as usize) as u32,
        Ordering::Relaxed,
    );
    result_ops::trim_halt_stack_for_result_limit(&mut stack, result_stack_limit);
    LAST_RESULT_STAGE.store(2, Ordering::Relaxed);
    LAST_RESULT_STACK_LEN.store(stack.len().min(u32::MAX as usize) as u32, Ordering::Relaxed);
    let abi_stack = to_abi_stack(&stack);
    LAST_RESULT_STAGE.store(3, Ordering::Relaxed);
    core::mem::forget(stack);
    core::mem::forget(locals);
    core::mem::forget(args);
    core::mem::forget(static_fields);
    core::mem::forget(method_initial_stack);
    core::mem::forget(consumed_mutations);
    core::mem::forget(pending_error);
    LAST_RESULT_STAGE.store(4, Ordering::Relaxed);

    Ok(ExecutionResult {
        fee_consumed_pico: 0,
        state: VmState::Halt,
        stack: abi_stack,
        fault_message: None,
        fault_ip: None,
        fault_locals: None,
    })
}

fn pop_jump_comparison(
    stack: &mut Vec<StackValue>,
    compare: fn(&AbiStackValue, &AbiStackValue) -> Result<bool, String>,
) -> Result<bool, String> {
    let right = pop_item(stack)?;
    let left = pop_item(stack)?;
    compare(&to_abi_value(&left), &to_abi_value(&right))
}
