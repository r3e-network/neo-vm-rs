#[cfg(target_arch = "riscv32")]
use super::helpers::{
    decode_retained_prefix_into, encode_retained_prefix_to_slice, RETAINED_CALL_STACK_BUF,
};
use super::runtime_types::{compound_id, propagate_aliases_from_sources, StackValue};
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::sync::atomic::{AtomicU32, Ordering};

pub(super) static LAST_INTERPRETER_IP: AtomicU32 = AtomicU32::new(u32::MAX);
pub(super) static LAST_RESULT_STAGE: AtomicU32 = AtomicU32::new(0);
pub(super) static LAST_RESULT_STACK_LEN: AtomicU32 = AtomicU32::new(0);
pub(super) static LAST_RESULT_LIMIT: AtomicU32 = AtomicU32::new(u32::MAX);

#[inline]
pub(super) fn record_interpreter_ip(ip: usize) {
    let value = if ip <= u32::MAX as usize {
        ip as u32
    } else {
        u32::MAX
    };
    LAST_INTERPRETER_IP.store(value, Ordering::Relaxed);
}

pub fn last_interpreter_ip() -> u32 {
    LAST_INTERPRETER_IP.load(Ordering::Relaxed)
}

pub fn last_result_stage() -> u32 {
    LAST_RESULT_STAGE.load(Ordering::Relaxed)
}

pub fn last_result_stack_len() -> u32 {
    LAST_RESULT_STACK_LEN.load(Ordering::Relaxed)
}

pub fn last_result_limit() -> u32 {
    LAST_RESULT_LIMIT.load(Ordering::Relaxed)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn propagate_active_aliases_into_saved_frame(
    saved_locals: &mut [StackValue],
    saved_args: &mut [StackValue],
    stack: &[StackValue],
    locals: &[StackValue],
    args: &[StackValue],
    static_fields: &[StackValue],
    consumed_mutations: &[StackValue],
) {
    for targets in [saved_locals, saved_args] {
        propagate_aliases_from_sources(targets, stack);
        propagate_aliases_from_sources(targets, locals);
        propagate_aliases_from_sources(targets, args);
        propagate_aliases_from_sources(targets, static_fields);
        propagate_aliases_from_sources(targets, consumed_mutations);
    }
}

pub(super) fn remember_consumed_mutation(
    consumed_mutations: &mut Vec<StackValue>,
    updated: &StackValue,
) {
    let Some(id) = compound_id(updated) else {
        return;
    };

    if let Some(existing) = consumed_mutations
        .iter_mut()
        .find(|value| compound_id(value) == Some(id))
    {
        *existing = updated.clone();
    } else {
        consumed_mutations.push(updated.clone());
    }
}

#[inline]
pub(super) fn reset_consumed_mutations(consumed_mutations: &mut Vec<StackValue>) {
    if cfg!(target_arch = "riscv32") {
        // Avoid walking and dropping potentially large alias-tracking values on
        // the PolkaVM/riscv32 path. The guest allocator is reset per execution.
        unsafe {
            core::ptr::write(consumed_mutations, Vec::new());
        }
    } else {
        consumed_mutations.clear();
    }
}

#[derive(Debug, Clone)]
pub(super) struct TryFrame {
    pub(super) catch_ip: usize,
    pub(super) finally_ip: usize,
    pub(super) call_depth: usize,
    pub(super) caught: bool,
    pub(super) in_finally: bool,
    pub(super) end_ip: usize,
}

#[derive(Debug)]
pub(super) enum PendingException {
    Message(String),
    ThrownValue(StackValue),
}

impl PendingException {
    pub(super) fn message(message: String) -> Self {
        Self::Message(message)
    }

    pub(super) fn thrown_value(value: StackValue) -> Self {
        Self::ThrownValue(value)
    }

    pub(super) fn into_catch_item(self) -> StackValue {
        match self {
            Self::Message(message) => StackValue::ByteString(message.into_bytes()),
            Self::ThrownValue(value) => value,
        }
    }

    pub(super) fn into_fault_message(self) -> String {
        match self {
            Self::Message(message) => message,
            Self::ThrownValue(value) => format!("THROW: {:?}", value),
        }
    }
}

/// Fixed-capacity stack for TryFrames — avoids heap allocation to prevent
/// PolkaVM bump allocator corruption during host_call round-trips.
pub(super) const MAX_STACK_SIZE: usize = 2048;
pub(super) const MAX_TRY_NESTING: usize = 16;
pub(super) const MAX_CALL_DEPTH: usize = 64;

pub(super) struct TryStack {
    frames: [core::mem::MaybeUninit<TryFrame>; MAX_TRY_NESTING],
    pub(super) len: usize,
}

pub(super) struct CallFrame {
    return_ip: usize,
    initialized: bool,
    #[cfg(target_arch = "riscv32")]
    retained_offset: usize,
    #[cfg(target_arch = "riscv32")]
    args_len: usize,
    #[cfg(target_arch = "riscv32")]
    locals_len: usize,
    #[cfg(not(target_arch = "riscv32"))]
    locals: Vec<StackValue>,
    #[cfg(not(target_arch = "riscv32"))]
    args: Vec<StackValue>,
}

pub(super) type RestoredCallFrame = (usize, Vec<StackValue>, Vec<StackValue>, bool);

pub(super) struct CallStack {
    frames: [core::mem::MaybeUninit<CallFrame>; MAX_CALL_DEPTH],
    len: usize,
    #[cfg(target_arch = "riscv32")]
    retained_len: usize,
}

impl CallStack {
    #[inline]
    pub(super) fn new() -> Self {
        Self {
            // Safety: MaybeUninit does not require initialization
            frames: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
            len: 0,
            #[cfg(target_arch = "riscv32")]
            retained_len: 0,
        }
    }

    #[inline]
    pub(super) fn len(&self) -> usize {
        self.len
    }

    #[cfg(target_arch = "riscv32")]
    pub(super) fn push_frame_refs(
        &mut self,
        return_ip: usize,
        locals: &[StackValue],
        args: &[StackValue],
        initialized: bool,
    ) -> Result<(), String> {
        if self.len >= MAX_CALL_DEPTH {
            return Err("call depth exceeds maximum".to_string());
        }

        let retained_offset = self.retained_len;
        let buf = unsafe { RETAINED_CALL_STACK_BUF.as_mut_slice() };
        let args_len = encode_retained_prefix_to_slice(args, &mut buf[retained_offset..])?;
        let locals_offset = retained_offset + args_len;
        let locals_len = encode_retained_prefix_to_slice(locals, &mut buf[locals_offset..])?;
        self.retained_len = locals_offset + locals_len;

        self.frames[self.len] = core::mem::MaybeUninit::new(CallFrame {
            return_ip,
            initialized,
            retained_offset,
            args_len,
            locals_len,
        });
        self.len += 1;
        Ok(())
    }

    #[cfg(not(target_arch = "riscv32"))]
    pub(super) fn push_frame(
        &mut self,
        return_ip: usize,
        locals: Vec<StackValue>,
        args: Vec<StackValue>,
        initialized: bool,
    ) -> Result<(), String> {
        if self.len >= MAX_CALL_DEPTH {
            return Err("call depth exceeds maximum".to_string());
        }

        #[cfg(target_arch = "riscv32")]
        {
            let retained_offset = self.retained_len;
            let buf = unsafe { RETAINED_CALL_STACK_BUF.as_mut_slice() };
            let args_len = encode_retained_prefix_to_slice(&args, &mut buf[retained_offset..])?;
            let locals_offset = retained_offset + args_len;
            let locals_len = encode_retained_prefix_to_slice(&locals, &mut buf[locals_offset..])?;
            self.retained_len = locals_offset + locals_len;

            self.frames[self.len] = core::mem::MaybeUninit::new(CallFrame {
                return_ip,
                initialized,
                #[cfg(target_arch = "riscv32")]
                retained_offset,
                #[cfg(target_arch = "riscv32")]
                args_len,
                #[cfg(target_arch = "riscv32")]
                locals_len,
            });
        }
        #[cfg(not(target_arch = "riscv32"))]
        {
            self.frames[self.len] = core::mem::MaybeUninit::new(CallFrame {
                return_ip,
                initialized,
                #[cfg(not(target_arch = "riscv32"))]
                locals,
                #[cfg(not(target_arch = "riscv32"))]
                args,
            });
        }
        self.len += 1;
        Ok(())
    }

    pub(super) fn pop_and_restore(&mut self) -> Result<Option<RestoredCallFrame>, String> {
        if self.len == 0 {
            return Ok(None);
        }

        self.len -= 1;
        // Safety: frames[self.len] was previously initialized by push_frame()
        let frame = unsafe { self.frames[self.len].assume_init_read() };
        #[cfg(target_arch = "riscv32")]
        {
            let locals_offset = frame.retained_offset + frame.args_len;
            let end = locals_offset + frame.locals_len;
            let bytes = unsafe { RETAINED_CALL_STACK_BUF.as_slice(end) };

            let mut args = Vec::new();
            decode_retained_prefix_into(
                &bytes[frame.retained_offset..frame.retained_offset + frame.args_len],
                &mut args,
            )?;

            let mut locals = Vec::new();
            decode_retained_prefix_into(
                &bytes[locals_offset..locals_offset + frame.locals_len],
                &mut locals,
            )?;

            self.retained_len = frame.retained_offset;
            Ok(Some((frame.return_ip, locals, args, frame.initialized)))
        }
        #[cfg(not(target_arch = "riscv32"))]
        {
            Ok(Some((
                frame.return_ip,
                frame.locals,
                frame.args,
                frame.initialized,
            )))
        }
    }
}

impl TryStack {
    #[inline]
    pub(super) fn new() -> Self {
        Self {
            // Safety: MaybeUninit does not require initialization
            frames: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    #[inline]
    pub(super) fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub(super) fn push(&mut self, frame: TryFrame) -> Result<(), String> {
        if self.len >= MAX_TRY_NESTING {
            return Err("TRY nesting exceeds maximum depth".to_string());
        }
        self.frames[self.len] = core::mem::MaybeUninit::new(frame);
        self.len += 1;
        Ok(())
    }

    #[inline]
    pub(super) fn pop(&mut self) -> Option<TryFrame> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        // Safety: frames[self.len] was previously initialized by push()
        Some(unsafe { self.frames[self.len].assume_init_read() })
    }

    #[inline]
    pub(super) fn last_mut(&mut self) -> Option<&mut TryFrame> {
        if self.len == 0 {
            return None;
        }
        // Safety: frames[self.len - 1] was previously initialized by push()
        Some(unsafe { self.frames[self.len - 1].assume_init_mut() })
    }

    /// Find the last uncaught frame index (iterating in reverse)
    pub(super) fn find_uncaught_index(&self) -> Option<usize> {
        for i in (0..self.len).rev() {
            // Safety: frames[i] was previously initialized by push()
            let frame = unsafe { &*self.frames[i].as_ptr() };
            if !frame.caught {
                return Some(i);
            }
        }
        None
    }

    #[inline]
    pub(super) fn get_mut(&mut self, index: usize) -> Option<&mut TryFrame> {
        if index >= self.len {
            return None;
        }
        // Safety: frames[index] was previously initialized by push()
        Some(unsafe { self.frames[index].assume_init_mut() })
    }
}
