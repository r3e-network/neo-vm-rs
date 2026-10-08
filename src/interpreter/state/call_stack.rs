#[cfg(target_arch = "riscv32")]
use super::super::helpers::{
    RETAINED_CALL_STACK_BUF, decode_retained_prefix_into, encode_retained_prefix_to_slice,
};
use super::{MAX_CALL_DEPTH, StackValue, call_frame::CallFrame};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

pub(in crate::interpreter) type RestoredCallFrame = (usize, Vec<StackValue>, Vec<StackValue>, bool);

pub(in crate::interpreter) struct CallStack {
    frames: [core::mem::MaybeUninit<CallFrame>; MAX_CALL_DEPTH],
    len: usize,
    shared_frames: Option<Vec<RestoredCallFrame>>,
    shared_root_count: usize,
    #[cfg(target_arch = "riscv32")]
    retained_len: usize,
}

impl CallStack {
    #[inline]
    pub(in crate::interpreter) fn new() -> Self {
        Self {
            // Safety: MaybeUninit does not require initialization
            frames: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
            len: 0,
            shared_frames: None,
            shared_root_count: 0,
            #[cfg(target_arch = "riscv32")]
            retained_len: 0,
        }
    }

    #[inline]
    pub(in crate::interpreter) fn len(&self) -> usize {
        self.shared_frames.as_ref().map_or(self.len, Vec::len)
    }

    pub(in crate::interpreter) fn reference_root_count(&self) -> usize {
        self.shared_root_count
    }

    pub(in crate::interpreter) fn new_shared() -> Self {
        let mut stack = Self::new();
        stack.shared_frames = Some(Vec::new());
        stack
    }

    pub(in crate::interpreter) fn push_shared(
        &mut self,
        return_ip: usize,
        locals: Vec<StackValue>,
        args: Vec<StackValue>,
        initialized: bool,
    ) -> Result<(), String> {
        let frames = self
            .shared_frames
            .as_mut()
            .ok_or_else(|| "missing reference call frames".to_string())?;
        if frames.len() >= MAX_CALL_DEPTH {
            return Err("call depth exceeds maximum".to_string());
        }
        self.shared_root_count += locals.len() + args.len();
        frames.push((return_ip, locals, args, initialized));
        Ok(())
    }

    #[cfg(target_arch = "riscv32")]
    pub(in crate::interpreter) fn push_frame_refs(
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
    pub(in crate::interpreter) fn push_frame(
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

    pub(in crate::interpreter) fn pop_and_restore(
        &mut self,
    ) -> Result<Option<RestoredCallFrame>, String> {
        if let Some(frames) = &mut self.shared_frames {
            let frame = frames.pop();
            if let Some((_, locals, args, _)) = &frame {
                self.shared_root_count -= locals.len() + args.len();
            }
            return Ok(frame);
        }
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
