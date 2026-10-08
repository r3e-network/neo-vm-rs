use super::*;
use crate::VmState;
use crate::reference::{SharedHeap, SharedValue};

/// A native fault or an ordinary VM exception retaining its object identity.
#[derive(Clone, Debug)]
pub enum SharedVmException {
    Fault(String),
    Throw(SharedValue),
}

impl From<String> for SharedVmException {
    fn from(value: String) -> Self {
        Self::Fault(value)
    }
}

impl core::fmt::Display for SharedVmException {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Fault(message) => formatter.write_str(message),
            Self::Throw(value) => write!(formatter, "THROW: {value:?}"),
        }
    }
}

/// Successful V3 execution with references into the supplied shared heap.
#[derive(Clone, Debug)]
pub struct SharedExecutionResult {
    pub fee_consumed_pico: i64,
    pub state: VmState,
    pub stack: Vec<SharedValue>,
}

/// V3 host boundary preserving mutable values across calls and exceptions.
/// `invocation_depth` includes the current method and all its internal CALL frames.
pub trait ReferenceSyscallProvider {
    fn on_instruction(&mut self, _opcode: u8) -> Result<(), String> {
        Ok(())
    }
    fn initializer_complete(&mut self, _ip: usize) -> Result<(), String> {
        Ok(())
    }
    /// Exact consumed argument count, or usize::MAX for a dynamic whole-stack call.
    fn syscall_arg_count(&self, api: u32, _ip: usize) -> Result<usize, String> {
        Ok(crate::syscall_arg_count(api))
    }
    /// Hosts with method tokens should return the token's parameter count.
    fn callt_arg_count(&self, _token: u16) -> Result<usize, String> {
        Ok(usize::MAX)
    }
    fn syscall(
        &mut self,
        api: u32,
        ip: usize,
        invocation_depth: usize,
        stack: &mut Vec<SharedValue>,
    ) -> Result<(), SharedVmException>;
    fn callt(
        &mut self,
        token: u16,
        ip: usize,
        invocation_depth: usize,
        stack: &mut Vec<SharedValue>,
    ) -> Result<(), SharedVmException> {
        self.syscall(CALLT_MARKER | u32::from(token), ip, invocation_depth, stack)
    }
}

/// Executes a method using one shared object heap. The entry depth starts at one.
pub fn interpret_with_stack_and_syscalls_at_reference<H: ReferenceSyscallProvider>(
    script: &[u8],
    initial_stack: Vec<SharedValue>,
    initial_ip: usize,
    heap: SharedHeap,
    base_invocation_depth: usize,
    host: &mut H,
) -> Result<SharedExecutionResult, SharedVmException> {
    super::super::boundary::interpret_reference(
        script,
        initial_stack,
        initial_ip,
        None,
        None,
        heap,
        base_invocation_depth,
        host,
    )
}

/// Shared execution retaining only the requested HALT result items.
pub fn interpret_with_stack_and_syscalls_at_with_result_limit_reference<
    H: ReferenceSyscallProvider,
>(
    script: &[u8],
    initial_stack: Vec<SharedValue>,
    initial_ip: usize,
    result_stack_limit: usize,
    heap: SharedHeap,
    base_invocation_depth: usize,
    host: &mut H,
) -> Result<SharedExecutionResult, SharedVmException> {
    super::super::boundary::interpret_reference(
        script,
        initial_stack,
        initial_ip,
        None,
        Some(result_stack_limit),
        heap,
        base_invocation_depth,
        host,
    )
}

/// Shared execution beginning with a contract initializer.
pub fn interpret_with_stack_and_syscalls_at_with_initializer_reference<
    H: ReferenceSyscallProvider,
>(
    script: &[u8],
    initial_stack: Vec<SharedValue>,
    initial_ip: usize,
    initializer_ip: usize,
    heap: SharedHeap,
    base_invocation_depth: usize,
    host: &mut H,
) -> Result<SharedExecutionResult, SharedVmException> {
    super::super::boundary::interpret_reference(
        script,
        initial_stack,
        initial_ip,
        Some(initializer_ip),
        None,
        heap,
        base_invocation_depth,
        host,
    )
}

/// Shared initializer execution retaining only the requested HALT result items.
pub fn interpret_with_stack_and_syscalls_at_with_initializer_and_result_limit_reference<
    H: ReferenceSyscallProvider,
>(
    script: &[u8],
    initial_stack: Vec<SharedValue>,
    initial_ip: usize,
    initializer_ip: usize,
    result_stack_limit: usize,
    heap: SharedHeap,
    base_invocation_depth: usize,
    host: &mut H,
) -> Result<SharedExecutionResult, SharedVmException> {
    super::super::boundary::interpret_reference(
        script,
        initial_stack,
        initial_ip,
        Some(initializer_ip),
        Some(result_stack_limit),
        heap,
        base_invocation_depth,
        host,
    )
}
