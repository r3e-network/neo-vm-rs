use super::super::boundary::RuntimeException as VmException;
use super::super::state::{PendingException, TryFrame, TryStack};
use alloc::string::ToString;

/// Neo uses -1 for an absent handler and an unset ENDTRY destination.
pub(super) const ABSENT: usize = usize::MAX;

fn offsets(
    script: &[u8],
    ip: usize,
    long: bool,
    count: usize,
) -> Result<(i32, i32, usize), VmException> {
    let width = if long { 4 } else { 1 };
    let end = ip
        .checked_add(1 + count * width)
        .filter(|end| *end <= script.len())
        .ok_or_else(|| VmException::Fault("truncated exception-control operand".to_string()))?;
    let read = |start: usize| {
        if long {
            i32::from_le_bytes(script[start..start + 4].try_into().unwrap())
        } else {
            i32::from(script[start] as i8)
        }
    };
    Ok((
        read(ip + 1),
        if count == 2 { read(ip + 1 + width) } else { 0 },
        end,
    ))
}

fn target(ip: usize, offset: i32) -> Result<usize, VmException> {
    let target = i32::try_from(ip)
        .ok()
        .and_then(|ip| ip.checked_add(offset))
        .ok_or_else(|| {
            VmException::Fault("exception-control target overflows Int32".to_string())
        })?;
    Ok(if target < 0 { ABSENT } else { target as usize })
}

pub(super) fn enter_try(
    script: &[u8],
    ip: usize,
    long: bool,
    call_depth: usize,
    frames: &mut TryStack,
) -> Result<usize, VmException> {
    let (catch, finally, next) = offsets(script, ip, long, 2)?;
    if catch == 0 && finally == 0 {
        return Err(VmException::Fault(
            "catchOffset and finallyOffset can't be 0 in a TRY block".to_string(),
        ));
    }
    frames.push(TryFrame {
        catch_ip: if catch == 0 {
            ABSENT
        } else {
            target(ip, catch)?
        },
        finally_ip: if finally == 0 {
            ABSENT
        } else {
            target(ip, finally)?
        },
        call_depth,
        caught: false,
        in_finally: false,
        end_ip: ABSENT,
    })?;
    Ok(next)
}

pub(super) fn end_try(
    script: &[u8],
    ip: usize,
    long: bool,
    call_depth: usize,
    frames: &mut TryStack,
) -> Result<usize, VmException> {
    let frame = frames
        .last_mut()
        .filter(|frame| frame.call_depth == call_depth)
        .ok_or_else(|| {
            VmException::Fault("The corresponding TRY block cannot be found.".to_string())
        })?;
    if frame.in_finally {
        return Err(VmException::Fault(
            "The opcode ENDTRY can't be executed in a FINALLY block.".to_string(),
        ));
    }
    let (offset, _, _) = offsets(script, ip, long, 1)?;
    let destination = target(ip, offset)?;
    if frame.finally_ip != ABSENT {
        frame.end_ip = destination;
        frame.in_finally = true;
        checked_destination(frame.finally_ip, script.len())
    } else {
        frames.pop();
        checked_destination(destination, script.len())
    }
}

pub(super) fn end_finally(
    script_len: usize,
    ip: usize,
    call_depth: usize,
    frames: &mut TryStack,
    has_exception: bool,
) -> Result<usize, VmException> {
    if !frames
        .last_mut()
        .is_some_and(|frame| frame.call_depth == call_depth)
    {
        return Err(VmException::Fault(
            "The corresponding TRY block cannot be found.".to_string(),
        ));
    }
    let frame = frames.pop().unwrap();
    if has_exception {
        Ok(ip)
    } else {
        checked_destination(frame.end_ip, script_len)
    }
}

pub(super) fn checked_destination(ip: usize, script_len: usize) -> Result<usize, VmException> {
    if ip > script_len {
        Err(VmException::Fault(
            "exception-control target out of bounds".to_string(),
        ))
    } else {
        Ok(ip)
    }
}

pub(super) fn find_handler(
    frames: &mut TryStack,
    pending: &PendingException,
) -> Result<Option<usize>, VmException> {
    if let PendingException::Message(message) = pending {
        return Err(VmException::Fault(message.clone()));
    }
    while let Some(frame) = frames.last_mut() {
        if !frame.in_finally
            && ((!frame.caught && frame.catch_ip != ABSENT) || frame.finally_ip != ABSENT)
        {
            return Ok(Some(frames.len - 1));
        }
        frames.pop();
    }
    Ok(None)
}

pub(super) fn into_exception(pending: PendingException) -> VmException {
    match pending {
        PendingException::Message(message) => VmException::Fault(message),
        PendingException::ThrownValue(value) => VmException::Throw(value),
    }
}
