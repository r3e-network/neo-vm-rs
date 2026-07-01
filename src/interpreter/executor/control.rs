use super::super::state::{PendingException, TryFrame, TryStack};
use alloc::string::{String, ToString};

pub(super) enum Dispatch {
    Fallthrough,
    Continue,
}

pub(super) fn enter_try_short(
    script: &[u8],
    ip: usize,
    call_depth: usize,
    try_frames: &mut TryStack,
    pending_error: &mut Option<PendingException>,
) -> Result<usize, String> {
    if ip + 3 > script.len() {
        *pending_error = Some(PendingException::message(
            "truncated TRY operand".to_string(),
        ));
        return Ok(ip);
    }
    let catch_offset = script[ip + 1] as i8;
    let finally_offset = script[ip + 2] as i8;
    // Canonical ExecuteTry faults (uncatchable) when BOTH offsets are 0.
    if catch_offset == 0 && finally_offset == 0 {
        return Err("catchOffset and finallyOffset can't be 0 in a TRY block".to_string());
    }
    // Canonical ExceptionHandlingContext: catchPointer = catchOffset==0 ? -1 :
    // ip+catchOffset, and HasCatch => CatchPointer >= 0. A block is PRESENT only
    // when its offset is non-zero AND its target lands at IP >= 0 — a backward
    // offset whose target is before the script start is "absent", exactly like
    // offset 0. ExecuteTry does NO upper-bound check; an out-of-range target
    // faults uncatchably LATER, only when assigned to the InstructionPointer
    // (throw routing / ENDTRY->finally), matching the C# InstructionPointer
    // setter (`value < 0 || value > Script.Length` throws). Eager-faulting here
    // diverged: an out-of-bounds target in a try body that completes WITHOUT
    // throwing HALTs canonically, and a throw to it is an uncatchable engine
    // fault, not a catchable one routed to an outer try.
    let catch_target = ip as isize + catch_offset as isize;
    let finally_target = ip as isize + finally_offset as isize;
    let has_catch = catch_offset != 0 && catch_target >= 0;
    let has_finally = finally_offset != 0 && finally_target >= 0;
    let catch_ip = if has_catch { catch_target as usize } else { 0 };
    let finally_ip = if has_finally { finally_target as usize } else { 0 };
    try_frames.push(TryFrame {
        catch_ip,
        finally_ip,
        has_catch,
        has_finally,
        call_depth,
        caught: false,
        in_finally: false,
        end_ip: 0,
        has_end_target: false,
    })?;
    Ok(ip + 3)
}

pub(super) fn enter_try_long(
    script: &[u8],
    ip: usize,
    call_depth: usize,
    try_frames: &mut TryStack,
    pending_error: &mut Option<PendingException>,
) -> Result<usize, String> {
    if ip + 9 > script.len() {
        *pending_error = Some(PendingException::message(
            "truncated TRY_L operand".to_string(),
        ));
        return Ok(ip);
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
    // Canonical ExecuteTry faults (uncatchable) when BOTH offsets are 0.
    if catch_offset == 0 && finally_offset == 0 {
        return Err("catchOffset and finallyOffset can't be 0 in a TRY block".to_string());
    }
    // See enter_try_short: HasCatch/HasFinally require a non-zero offset AND a
    // target IP >= 0; the upper-bound check is deferred to the IP-assignment site
    // (canonical InstructionPointer setter), not done eagerly at TRY_L.
    let catch_target = ip as isize + catch_offset as isize;
    let finally_target = ip as isize + finally_offset as isize;
    let has_catch = catch_offset != 0 && catch_target >= 0;
    let has_finally = finally_offset != 0 && finally_target >= 0;
    let catch_ip = if has_catch { catch_target as usize } else { 0 };
    let finally_ip = if has_finally { finally_target as usize } else { 0 };
    try_frames.push(TryFrame {
        catch_ip,
        finally_ip,
        has_catch,
        has_finally,
        call_depth,
        caught: false,
        in_finally: false,
        end_ip: 0,
        has_end_target: false,
    })?;
    Ok(ip + 9)
}

pub(super) fn end_try_short(
    script: &[u8],
    ip: usize,
    try_frames: &mut TryStack,
    pending_error: &mut Option<PendingException>,
) -> Result<usize, String> {
    if ip + 2 > script.len() {
        *pending_error = Some(PendingException::message(
            "truncated ENDTRY operand".to_string(),
        ));
        return Ok(ip);
    }
    let offset = script[ip + 1] as i8;
    let target_ip = if offset != 0 {
        (ip as isize + offset as isize) as usize
    } else {
        ip + 2
    };
    end_try_at(target_ip, script.len(), try_frames)
}

pub(super) fn end_try_long(
    script: &[u8],
    ip: usize,
    try_frames: &mut TryStack,
    pending_error: &mut Option<PendingException>,
) -> Result<usize, String> {
    if ip + 5 > script.len() {
        *pending_error = Some(PendingException::message(
            "truncated ENDTRY_L operand".to_string(),
        ));
        return Ok(ip);
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
    end_try_at(target_ip, script.len(), try_frames)
}

pub(super) fn end_finally(
    ip: usize,
    script_len: usize,
    try_frames: &mut TryStack,
    pending_error: &mut Option<PendingException>,
    unwind_exception: &mut Option<PendingException>,
) -> Result<Option<usize>, String> {
    let in_finally = try_frames.last_mut().is_some_and(|frame| frame.in_finally);
    if !in_finally {
        return Err("ENDFINALLY without matching finally context".to_string());
    }

    let frame = try_frames.pop().expect("frame checked above");
    if let Some(carried) = unwind_exception.take() {
        // This finally ran during exception unwinding: now that it has completed,
        // re-propagate the carried exception to the next outer handler. The loop
        // top will dispatch it (find_handler_index skips this now-popped frame).
        *pending_error = Some(carried);
        return Ok(Some(ip));
    }
    if frame.has_end_target {
        // Canonical ENDFINALLY assigns IP = EndPointer unconditionally; the
        // stashed end target (possibly IP 0) becomes the IP now — bounds-check it
        // like the canonical InstructionPointer setter (faults on > Length).
        if frame.end_ip > script_len {
            return Err("Out of script bounds".to_string());
        }
        return Ok(Some(frame.end_ip));
    }
    Ok(None)
}

fn end_try_at(
    target_ip: usize,
    script_len: usize,
    try_frames: &mut TryStack,
) -> Result<usize, String> {
    // C# ExecuteEndTry throws "The corresponding TRY block cannot be found."
    // when the current context has no try frame; mirror that as a FAULT
    // instead of silently treating ENDTRY/ENDTRY_L as a no-op jump.
    let Some(frame) = try_frames.last_mut() else {
        return Err("The corresponding TRY block cannot be found.".to_string());
    };
    // Canonical ExecuteEndTry faults (uncatchable) if the frame is already in
    // its FINALLY block (a second ENDTRY inside the finally body).
    if frame.in_finally {
        return Err("The opcode ENDTRY can't be executed in a FINALLY block.".to_string());
    }
    if frame.has_finally {
        // The finally entry becomes the IP now, so bounds-check it like the
        // canonical InstructionPointer setter (faults uncatchably on > Length);
        // TRY no longer eager-checks it. The end target is applied LATER by
        // ENDFINALLY (bounds-checked there, lazily).
        if frame.finally_ip > script_len {
            return Err("Out of script bounds".to_string());
        }
        frame.end_ip = target_ip;
        frame.has_end_target = true;
        frame.in_finally = true;
        return Ok(frame.finally_ip);
    }
    // No finally: the end target becomes the IP now, so bounds-check it like the
    // canonical InstructionPointer setter (faults on value > Script.Length).
    if target_ip > script_len {
        return Err("Out of script bounds".to_string());
    }
    try_frames.pop();
    Ok(target_ip)
}
