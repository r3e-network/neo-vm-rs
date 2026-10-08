use super::boundary::{RuntimeException as VmException, RuntimeHost, RuntimeResult};
mod byte_ops;
mod call_ops;
mod comparison_ops;
mod compound_ops;
mod control;
mod numeric_ops;
mod push_ops;
mod reference_coercion_ops;
mod reference_ops;
mod result_ops;
mod slot_ops;
mod stack_ops;
mod typed_control;
mod typed_item_ops;
use super::helpers::*;
use super::opcodes::*;
use super::runtime_types::{CompoundIds, StackValue, to_abi_value};
use super::state::*;
use crate::{StackValue as AbiStackValue, semantics::comparison as comparison_rules};
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::sync::atomic::Ordering;

pub(super) fn interpret_with_stack_and_syscalls_at_internal<H: RuntimeHost>(
    script: &[u8],
    mut method_initial_stack: Vec<StackValue>,
    mut ids: CompoundIds,
    initial_ip: usize,
    initializer_ip: Option<usize>,
    result_stack_limit: Option<usize>,
    typed_exceptions: bool,
    reference_mode: bool,
    base_invocation_depth: usize,
    host: &mut H,
) -> Result<RuntimeResult, VmException> {
    LAST_INTERPRETER_IP.store(u32::MAX, Ordering::Relaxed);
    LAST_RESULT_STAGE.store(0, Ordering::Relaxed);
    LAST_RESULT_STACK_LEN.store(0, Ordering::Relaxed);
    LAST_RESULT_LIMIT.store(u32::MAX, Ordering::Relaxed);

    if initial_ip > script.len() {
        return Err(VmException::Fault(
            "initial instruction pointer out of bounds".to_string(),
        ));
    }
    if initializer_ip.is_some_and(|ip| ip > script.len()) {
        return Err(VmException::Fault(
            "initializer instruction pointer out of bounds".to_string(),
        ));
    }
    let mut running_initializer = initializer_ip.is_some();
    let retained_initializer_method_stack_len = if running_initializer && !reference_mode {
        retain_initializer_method_stack(&method_initial_stack)?
    } else {
        None
    };
    let mut stack = if running_initializer && !reference_mode {
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
    let mut call_stack = if reference_mode {
        CallStack::new_shared()
    } else {
        CallStack::new()
    };
    let mut consumed_mutations: Vec<StackValue> = Vec::with_capacity(16);
    let mut pending_error: Option<PendingException> = None;
    let mut deferred_exception: Option<PendingException> = None;
    let mut previous_opcode_was_callt = false;

    macro_rules! apply_dispatch {
        ($dispatch:expr) => {
            if let control::Dispatch::Continue = $dispatch {
                continue;
            }
        };
    }

    'main_loop: loop {
        host.check_references(
            stack.len()
                + locals.len()
                + args.len()
                + static_fields.len()
                + method_initial_stack.len()
                + call_stack.reference_root_count(),
        )?;
        if pending_error.is_some() {
            let handler = if typed_exceptions {
                // Neo has one UncaughtException slot: a new throw replaces it,
                // and entering any catch clears the previous exception.
                deferred_exception = None;
                typed_control::find_handler(&mut try_frames, pending_error.as_ref().unwrap())?
            } else {
                try_frames.find_uncaught_index()
            };
            if let Some(frame_index) = handler {
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
                let enter_catch = !typed_exceptions || !frame.caught;
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
                let absent = if typed_exceptions {
                    typed_control::ABSENT
                } else {
                    0
                };
                if enter_catch && saved_catch_ip != absent {
                    stack.push(pending.into_catch_item());
                    ip = if typed_exceptions {
                        typed_control::checked_destination(saved_catch_ip, script.len())?
                    } else {
                        saved_catch_ip
                    };
                } else if saved_finally_ip != absent {
                    frame.in_finally = true;
                    ip = if typed_exceptions {
                        typed_control::checked_destination(saved_finally_ip, script.len())?
                    } else {
                        saved_finally_ip
                    };
                    if typed_exceptions {
                        // Execute finally before resuming this exception's unwind.
                        deferred_exception = Some(pending);
                    } else {
                        pending_error = Some(pending);
                    }
                } else {
                    continue;
                }
                continue;
            } else {
                let pending = pending_error
                    .take()
                    .ok_or_else(|| "missing pending exception without try frame".to_string())?;
                return Err(if typed_exceptions {
                    typed_control::into_exception(pending)
                } else {
                    VmException::Fault(pending.into_fault_message())
                });
            }
        }

        // Stack overflow check: NeoVM faults when stack exceeds 2048 items.
        if stack.len() > MAX_STACK_SIZE {
            return Err(VmException::Fault("stack overflow".to_string()));
        }

        let opcode = match script.get(ip) {
            Some(opcode) => *opcode,
            None if reference_mode => RET,
            None => break,
        };
        record_interpreter_ip(ip);
        host.on_instruction(opcode)?;
        if reference_mode {
            reference_coercion_ops::validate(opcode, script, ip, &stack)?;
        }
        let callt_result_pending_store = previous_opcode_was_callt;
        previous_opcode_was_callt = false;
        match opcode {
            PUSHA if reference_mode => {
                ip = call_ops::push_reference_pointer(script, ip, &mut stack, &ids)?;
                continue;
            }
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
                if comparison_ops::jump_equal(&mut stack, reference_mode)? {
                    ip = compute_jump_target_offset(ip, offset, script.len(), "JMPEQ")?;
                    continue;
                }
                ip += advance;
                continue;
            }
            ABORT => {
                if typed_exceptions {
                    return Err(VmException::Fault("ABORT".to_string()));
                }
                // ABORT is uncatchable — always FAULT. Return via the Ok(Fault) channel
                // so we can attribute the faulting IP; the host converts this back to an
                // Err(String) at the FFI boundary (lib.rs fault-message override).
                return Ok(result_ops::fault_result(
                    &stack,
                    &locals,
                    ip,
                    "ABORT".to_string(),
                ));
            }
            ASSERT => {
                let value = pop_boolean(&mut stack)?;
                if !value {
                    if typed_exceptions {
                        return Err(VmException::Fault("ASSERT failed".to_string()));
                    }
                    // ASSERT is uncatchable — always FAULT. Same Ok(Fault) channel as ABORT
                    // so the faulting IP is preserved.
                    return Ok(result_ops::fault_result(
                        &stack,
                        &locals,
                        ip,
                        "ASSERT failed".to_string(),
                    ));
                }
            }
            SYSCALL => {
                if ip + 5 > script.len() {
                    return Err(VmException::Fault("truncated SYSCALL operand".to_string()));
                }

                let api = u32::from_le_bytes([
                    script[ip + 1],
                    script[ip + 2],
                    script[ip + 3],
                    script[ip + 4],
                ]);
                if let Err(e) = host.syscall(
                    api,
                    ip,
                    &mut stack,
                    &mut args,
                    &mut locals,
                    &mut static_fields,
                    &mut consumed_mutations,
                    &mut ids,
                    typed_exceptions,
                    base_invocation_depth + usize::from(running_initializer) + call_stack.len(),
                ) {
                    if try_frames.is_empty() {
                        return Err(e);
                    }
                    pending_error = Some(match e {
                        VmException::Fault(message) if !typed_exceptions => {
                            PendingException::message(message)
                        }
                        VmException::Fault(message) => return Err(VmException::Fault(message)),
                        VmException::Throw(value) => PendingException::thrown_value(value),
                    });
                    continue;
                }
                ip += 5;
                continue;
            }
            CALLT => {
                if ip + 3 > script.len() {
                    return Err(VmException::Fault("truncated CALLT operand".to_string()));
                }
                let token = u16::from_le_bytes([script[ip + 1], script[ip + 2]]);
                if let Err(e) = host.callt(
                    token,
                    ip,
                    &mut stack,
                    &mut args,
                    &mut locals,
                    &mut static_fields,
                    &mut consumed_mutations,
                    &mut ids,
                    typed_exceptions,
                    base_invocation_depth + usize::from(running_initializer) + call_stack.len(),
                ) {
                    if try_frames.is_empty() {
                        return Err(e);
                    }
                    pending_error = Some(match e {
                        VmException::Fault(message) if !typed_exceptions => {
                            PendingException::message(message)
                        }
                        VmException::Fault(message) => return Err(VmException::Fault(message)),
                        VmException::Throw(value) => PendingException::thrown_value(value),
                    });
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
                    typed_exceptions,
                )?);
            }
            // EXCEPTION HANDLING OPCODES
            THROW => {
                let msg = pop_item(&mut stack)?;
                if typed_exceptions && try_frames.is_empty() {
                    return Err(VmException::Throw(msg));
                }
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
                    return Ok(result_ops::fault_result(&stack, &locals, ip, err_msg));
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
                    typed_exceptions,
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
                call_ops::push_frame(
                    &mut call_stack,
                    return_ip,
                    &mut locals,
                    &mut args,
                    saved_init,
                    reference_mode,
                    base_invocation_depth + usize::from(running_initializer),
                )?;
                slots_initialized = false;
                ip = compute_jump_target_offset(ip, offset, script.len(), "CALL")?;
                continue;
            }
            CALLA => {
                let item = pop_item(&mut stack)?;
                let offset = call_ops::pointer_target(&item, &ids, reference_mode)?;
                let return_ip = ip + 1;
                let saved_init = slots_initialized;
                reset_consumed_mutations(&mut consumed_mutations);
                call_ops::push_frame(
                    &mut call_stack,
                    return_ip,
                    &mut locals,
                    &mut args,
                    saved_init,
                    reference_mode,
                    base_invocation_depth + usize::from(running_initializer),
                )?;
                slots_initialized = false;
                if offset > script.len() {
                    return Err(VmException::Fault("CALLA target out of bounds".to_string()));
                }
                ip = offset;
                continue;
            }
            ABORTMSG => {
                let msg = pop_bytes(&mut stack)?;
                let msg_str = String::from_utf8_lossy(&msg);
                let err_msg = format!("ABORTMSG is executed. Reason: {}", msg_str);
                if typed_exceptions {
                    return Err(VmException::Fault(err_msg));
                }
                // ABORTMSG is uncatchable — always FAULT
                return Ok(result_ops::fault_result(&stack, &locals, ip, err_msg));
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
                    if typed_exceptions {
                        return Err(VmException::Fault(err_msg));
                    }
                    // ASSERTMSG is uncatchable — always FAULT
                    return Ok(result_ops::fault_result(&stack, &locals, ip, err_msg));
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
                if !comparison_ops::jump_equal(&mut stack, reference_mode)? {
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
                    host.complete_initializer(ip, &mut method_initial_stack, &mut static_fields)?;
                    if !reference_mode {
                        stack = core::mem::take(&mut method_initial_stack);
                    }
                    ip = initial_ip;
                    locals = Vec::with_capacity(16);
                    args = Vec::with_capacity(16);
                    slots_initialized = false;
                    try_frames = TryStack::new();
                    call_stack = if reference_mode {
                        CallStack::new_shared()
                    } else {
                        CallStack::new()
                    };
                    pending_error = None;
                    deferred_exception = None;
                    previous_opcode_was_callt = false;
                    running_initializer = false;
                    reset_consumed_mutations(&mut consumed_mutations);
                    continue;
                }
                break 'main_loop;
            }
            TRY => {
                if typed_exceptions {
                    ip = typed_control::enter_try(
                        script,
                        ip,
                        false,
                        call_stack.len(),
                        &mut try_frames,
                    )?;
                    continue;
                }
                ip = control::enter_try_short(
                    script,
                    ip,
                    call_stack.len(),
                    &mut try_frames,
                    &mut pending_error,
                )?;
                continue;
            }
            ENDTRY => {
                if typed_exceptions {
                    ip = typed_control::end_try(
                        script,
                        ip,
                        false,
                        call_stack.len(),
                        &mut try_frames,
                    )?;
                    continue;
                }
                ip = control::end_try_short(script, ip, &mut try_frames, &mut pending_error);
                continue;
            }
            TRY_L => {
                if typed_exceptions {
                    ip = typed_control::enter_try(
                        script,
                        ip,
                        true,
                        call_stack.len(),
                        &mut try_frames,
                    )?;
                    continue;
                }
                ip = control::enter_try_long(
                    script,
                    ip,
                    call_stack.len(),
                    &mut try_frames,
                    &mut pending_error,
                )?;
                continue;
            }
            ENDTRY_L => {
                if typed_exceptions {
                    ip = typed_control::end_try(
                        script,
                        ip,
                        true,
                        call_stack.len(),
                        &mut try_frames,
                    )?;
                    continue;
                }
                ip = control::end_try_long(script, ip, &mut try_frames, &mut pending_error);
                continue;
            }
            ENDFINALLY => {
                if typed_exceptions {
                    pending_error = deferred_exception.take();
                    ip = typed_control::end_finally(
                        script.len(),
                        ip,
                        call_stack.len(),
                        &mut try_frames,
                        pending_error.is_some(),
                    )?;
                    continue;
                }
                if let Some(next_ip) = control::end_finally(ip, &mut try_frames, &pending_error)? {
                    ip = next_ip;
                    continue;
                }
            }
            other => {
                return Err(VmException::Fault(format!(
                    "unsupported opcode 0x{other:02x}"
                )));
            }
        }
        ip += 1;
    }

    host.check_references(
        stack.len()
            + locals.len()
            + args.len()
            + static_fields.len()
            + method_initial_stack.len()
            + call_stack.reference_root_count(),
    )?;
    result_ops::finish_halt_result(
        stack,
        locals,
        args,
        static_fields,
        method_initial_stack,
        consumed_mutations,
        pending_error,
        result_stack_limit,
        reference_mode,
    )
}

fn pop_jump_comparison(
    stack: &mut Vec<StackValue>,
    compare: fn(&AbiStackValue, &AbiStackValue) -> Result<bool, String>,
) -> Result<bool, String> {
    let right = pop_item(stack)?;
    let left = pop_item(stack)?;
    compare(&to_abi_value(&left), &to_abi_value(&right))
}
