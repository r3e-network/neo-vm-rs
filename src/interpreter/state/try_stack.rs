//! Try/finally frames for ALL active call contexts.
//!
//! Canonical NeoVM keeps a SEPARATE try stack per `ExecutionContext`, each
//! capped at `MaxTryNestingDepth` (16). This is a single flat list across all
//! contexts (every frame tags its `call_depth`); the 16-deep limit is enforced
//! PER context — not globally, which previously over-faulted a valid contract
//! that opened ≤16 trys in each of several call frames.
//!
//! Storage differs by target, but the per-context limit semantics are identical:
//! - **guest (riscv32):** a fixed inline array, sized for the worst case
//!   (`MAX_TRY_FRAMES_TOTAL`). No heap — this avoids PolkaVM bump-allocator
//!   corruption during host_call round-trips.
//! - **host (native):** a heap `Vec`. There is no bump allocator on the host, so
//!   the no-heap constraint does not apply, and an inline worst-case array
//!   (~640 KiB) would overflow test-thread stacks.

#[cfg(target_arch = "riscv32")]
pub(in crate::interpreter) use inline::TryStack;
#[cfg(not(target_arch = "riscv32"))]
pub(in crate::interpreter) use heap::TryStack;

use super::TryFrame;

/// Canonical NeoVM handler test for exception unwinding (`ExecuteThrow`): a try
/// frame can handle a propagating exception iff it is NOT already running its
/// finally (State != Finally) AND it is not a spent catch (State==Catch with no
/// finally). Equivalently: State==Try, or State==Catch WITH a finally still to
/// run. Routing then sends a State==Try-with-catch frame to its catch and
/// everything else (finally-only, or a catch-with-finally re-throw) to its
/// finally. Mirrors v3.9.0 JumpTable.Control.ExecuteThrow's skip condition
/// `State==Finally || (State==Catch && !HasFinally)`.
fn frame_can_handle(frame: &TryFrame) -> bool {
    !frame.in_finally && (!frame.caught || frame.has_finally)
}

#[cfg(target_arch = "riscv32")]
mod inline {
    use super::super::{MAX_TRY_FRAMES_TOTAL, MAX_TRY_NESTING, TryFrame};
    use alloc::string::{String, ToString};

    pub(in crate::interpreter) struct TryStack {
        frames: [core::mem::MaybeUninit<TryFrame>; MAX_TRY_FRAMES_TOTAL],
        len: usize,
    }

    impl TryStack {
        #[inline]
        pub(in crate::interpreter) fn new() -> Self {
            Self {
                // Safety: MaybeUninit does not require initialization
                frames: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
                len: 0,
            }
        }

        #[inline]
        pub(in crate::interpreter) fn len(&self) -> usize {
            self.len
        }

        #[inline]
        pub(in crate::interpreter) fn is_empty(&self) -> bool {
            self.len == 0
        }

        pub(in crate::interpreter) fn push(&mut self, frame: TryFrame) -> Result<(), String> {
            if context_try_count(&self.frames, self.len, frame.call_depth) >= MAX_TRY_NESTING {
                return Err("MaxTryNestingDepth exceed.".to_string());
            }
            // Defensive: the per-context limit plus the call-depth bound keep the
            // total within MAX_TRY_FRAMES_TOTAL; never index out of the array.
            if self.len >= MAX_TRY_FRAMES_TOTAL {
                return Err("MaxTryNestingDepth exceed.".to_string());
            }
            self.frames[self.len] = core::mem::MaybeUninit::new(frame);
            self.len += 1;
            Ok(())
        }

        #[inline]
        pub(in crate::interpreter) fn pop(&mut self) -> Option<TryFrame> {
            if self.len == 0 {
                return None;
            }
            self.len -= 1;
            // Safety: frames[self.len] was previously initialized by push()
            Some(unsafe { self.frames[self.len].assume_init_read() })
        }

        #[inline]
        pub(in crate::interpreter) fn last_mut(&mut self) -> Option<&mut TryFrame> {
            if self.len == 0 {
                return None;
            }
            // Safety: frames[self.len - 1] was previously initialized by push()
            Some(unsafe { self.frames[self.len - 1].assume_init_mut() })
        }

        pub(in crate::interpreter) fn find_handler_index(&self) -> Option<usize> {
            for i in (0..self.len).rev() {
                // Safety: frames[i] was previously initialized by push()
                let frame = unsafe { &*self.frames[i].as_ptr() };
                if super::frame_can_handle(frame) {
                    return Some(i);
                }
            }
            None
        }

        #[inline]
        pub(in crate::interpreter) fn get_mut(&mut self, index: usize) -> Option<&mut TryFrame> {
            if index >= self.len {
                return None;
            }
            // Safety: frames[index] was previously initialized by push()
            Some(unsafe { self.frames[index].assume_init_mut() })
        }
    }

    /// Count the open trys belonging to `depth`'s call context. By the call
    /// discipline (a try opens only at the current/deepest call_depth, and RET
    /// pops every deeper frame) a context's frames are a contiguous block at the
    /// TOP of the stack, so this is the run of trailing frames sharing `depth`.
    fn context_try_count(
        frames: &[core::mem::MaybeUninit<TryFrame>],
        len: usize,
        depth: usize,
    ) -> usize {
        let mut count = 0usize;
        for i in (0..len).rev() {
            // Safety: frames[i] was previously initialized by push()
            let frame = unsafe { &*frames[i].as_ptr() };
            if frame.call_depth == depth {
                count += 1;
            } else {
                break;
            }
        }
        count
    }
}

#[cfg(not(target_arch = "riscv32"))]
mod heap {
    use super::super::{MAX_TRY_NESTING, TryFrame};
    use alloc::{
        string::{String, ToString},
        vec::Vec,
    };

    pub(in crate::interpreter) struct TryStack {
        frames: Vec<TryFrame>,
    }

    impl TryStack {
        #[inline]
        pub(in crate::interpreter) fn new() -> Self {
            Self { frames: Vec::new() }
        }

        #[inline]
        pub(in crate::interpreter) fn len(&self) -> usize {
            self.frames.len()
        }

        #[inline]
        pub(in crate::interpreter) fn is_empty(&self) -> bool {
            self.frames.is_empty()
        }

        pub(in crate::interpreter) fn push(&mut self, frame: TryFrame) -> Result<(), String> {
            // Per-context limit: count only the open trys sharing this frame's
            // call_depth (a contiguous block at the top of the stack).
            let depth = frame.call_depth;
            let mut in_context = 0usize;
            for existing in self.frames.iter().rev() {
                if existing.call_depth == depth {
                    in_context += 1;
                } else {
                    break;
                }
            }
            if in_context >= MAX_TRY_NESTING {
                return Err("MaxTryNestingDepth exceed.".to_string());
            }
            self.frames.push(frame);
            Ok(())
        }

        #[inline]
        pub(in crate::interpreter) fn pop(&mut self) -> Option<TryFrame> {
            self.frames.pop()
        }

        #[inline]
        pub(in crate::interpreter) fn last_mut(&mut self) -> Option<&mut TryFrame> {
            self.frames.last_mut()
        }

        pub(in crate::interpreter) fn find_handler_index(&self) -> Option<usize> {
            self.frames.iter().rposition(super::frame_can_handle)
        }

        #[inline]
        pub(in crate::interpreter) fn get_mut(&mut self, index: usize) -> Option<&mut TryFrame> {
            self.frames.get_mut(index)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::TryFrame;
    use super::TryStack;

    fn frame(call_depth: usize) -> TryFrame {
        TryFrame {
            catch_ip: 1,
            finally_ip: 0,
            has_catch: true,
            has_finally: false,
            call_depth,
            caught: false,
            in_finally: false,
            end_ip: 0,
            has_end_target: false,
        }
    }

    #[test]
    fn per_context_limit_is_sixteen() {
        let mut stack = TryStack::new();
        for _ in 0..16 {
            stack
                .push(frame(0))
                .expect("16 trys in one context are allowed");
        }
        assert!(
            stack.push(frame(0)).is_err(),
            "the 17th try in the SAME context must fault (MaxTryNestingDepth)"
        );
    }

    #[test]
    fn limit_is_per_context_not_global() {
        // 16 trys in each of 20 distinct call contexts: 320 open trys total, far
        // above the canonical per-context cap of 16 but legal because each
        // context stays within it. The previous GLOBAL cap of 16 over-faulted
        // this at the 17th try overall (D-6).
        let mut stack = TryStack::new();
        for depth in 0..20 {
            for _ in 0..16 {
                stack
                    .push(frame(depth))
                    .expect("each context may independently hold up to 16 trys");
            }
        }
        assert_eq!(stack.len(), 20 * 16);
    }
}
