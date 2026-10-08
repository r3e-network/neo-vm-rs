use super::api::{
    ReferenceSyscallProvider, SharedExecutionResult, SharedVmException, SyscallProvider,
    VmException,
};
use super::executor::interpret_with_stack_and_syscalls_at_internal as execute;
use super::helpers;
use super::runtime_types::{CompoundIds, StackValue, export_shared, into_abi_value, to_abi_stack};
use crate::reference::{SharedFrameRoots, SharedHeap, SharedValue};
use crate::{ExecutionResult, StackValue as AbiStackValue, VmState};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

/// Keep thrown values internal until the selected public boundary is reached.
pub(super) enum RuntimeException {
    Fault(String),
    Throw(StackValue),
}

impl From<String> for RuntimeException {
    fn from(value: String) -> Self {
        Self::Fault(value)
    }
}

pub(super) enum RuntimeResult {
    LegacyFault(ExecutionResult),
    Halt(Vec<StackValue>),
}

pub(super) trait RuntimeHost {
    fn on_instruction(&mut self, opcode: u8) -> Result<(), String>;
    fn check_references(&mut self, _roots: usize) -> Result<(), String> {
        Ok(())
    }
    fn complete_initializer(
        &mut self,
        ip: usize,
        initial: &mut Vec<StackValue>,
        static_fields: &mut Vec<StackValue>,
    ) -> Result<(), String>;
    #[allow(clippy::too_many_arguments)]
    fn syscall(
        &mut self,
        api: u32,
        ip: usize,
        stack: &mut Vec<StackValue>,
        args: &mut Vec<StackValue>,
        locals: &mut Vec<StackValue>,
        static_fields: &mut Vec<StackValue>,
        mutations: &mut Vec<StackValue>,
        ids: &mut CompoundIds,
        typed: bool,
        depth: usize,
    ) -> Result<(), RuntimeException>;
    #[allow(clippy::too_many_arguments)]
    fn callt(
        &mut self,
        token: u16,
        ip: usize,
        stack: &mut Vec<StackValue>,
        args: &mut Vec<StackValue>,
        locals: &mut Vec<StackValue>,
        static_fields: &mut Vec<StackValue>,
        mutations: &mut Vec<StackValue>,
        ids: &mut CompoundIds,
        typed: bool,
        depth: usize,
    ) -> Result<(), RuntimeException>;
}

struct AbiHost<'a, H>(&'a mut H);

fn import_exception(error: VmException, ids: &mut CompoundIds) -> RuntimeException {
    match error {
        VmException::Fault(message) => RuntimeException::Fault(message),
        VmException::Throw(value) => RuntimeException::Throw(ids.import_abi(value)),
    }
}

impl<H: SyscallProvider> RuntimeHost for AbiHost<'_, H> {
    fn on_instruction(&mut self, opcode: u8) -> Result<(), String> {
        self.0.on_instruction(opcode)
    }
    fn complete_initializer(
        &mut self,
        ip: usize,
        initial: &mut Vec<StackValue>,
        static_fields: &mut Vec<StackValue>,
    ) -> Result<(), String> {
        helpers::complete_initializer_retaining_state(self.0, ip, initial, static_fields)
    }
    fn syscall(
        &mut self,
        api: u32,
        ip: usize,
        stack: &mut Vec<StackValue>,
        args: &mut Vec<StackValue>,
        locals: &mut Vec<StackValue>,
        static_fields: &mut Vec<StackValue>,
        mutations: &mut Vec<StackValue>,
        ids: &mut CompoundIds,
        typed: bool,
        _depth: usize,
    ) -> Result<(), RuntimeException> {
        helpers::invoke_syscall(
            self.0,
            api,
            ip,
            stack,
            args,
            locals,
            static_fields,
            mutations,
            ids,
            typed,
        )
        .map_err(|error| import_exception(error, ids))
    }
    fn callt(
        &mut self,
        token: u16,
        ip: usize,
        stack: &mut Vec<StackValue>,
        args: &mut Vec<StackValue>,
        locals: &mut Vec<StackValue>,
        static_fields: &mut Vec<StackValue>,
        mutations: &mut Vec<StackValue>,
        ids: &mut CompoundIds,
        typed: bool,
        _depth: usize,
    ) -> Result<(), RuntimeException> {
        helpers::invoke_callt(
            self.0,
            token,
            ip,
            stack,
            args,
            locals,
            static_fields,
            mutations,
            ids,
            typed,
        )
        .map_err(|error| import_exception(error, ids))
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn interpret_abi<H: SyscallProvider>(
    script: &[u8],
    initial: Vec<AbiStackValue>,
    initial_ip: usize,
    initializer: Option<usize>,
    result_limit: Option<usize>,
    typed: bool,
    host: &mut H,
) -> Result<ExecutionResult, VmException> {
    let mut ids = CompoundIds::default();
    let initial = initial
        .into_iter()
        .map(|value| ids.import_abi(value))
        .collect();
    match execute(
        script,
        initial,
        ids,
        initial_ip,
        initializer,
        result_limit,
        typed,
        false,
        0,
        &mut AbiHost(host),
    ) {
        Ok(RuntimeResult::LegacyFault(result)) => Ok(result),
        Ok(RuntimeResult::Halt(stack)) => {
            let abi_stack = to_abi_stack(&stack);
            #[cfg(target_arch = "riscv32")]
            core::mem::forget(stack);
            Ok(ExecutionResult {
                fee_consumed_pico: 0,
                state: VmState::Halt,
                stack: abi_stack,
                fault_message: None,
                fault_ip: None,
                fault_locals: None,
            })
        }
        Err(RuntimeException::Fault(message)) => Err(VmException::Fault(message)),
        Err(RuntimeException::Throw(value)) => Err(VmException::Throw(into_abi_value(value))),
    }
}

struct SharedHost<'a, H> {
    host: &'a mut H,
    heap: SharedHeap,
    roots: SharedFrameRoots,
    root_count: usize,
}

impl<H: ReferenceSyscallProvider> SharedHost<'_, H> {
    fn detach_arguments(&mut self, count: usize, stack_len: usize) -> Result<usize, String> {
        let count = if count == usize::MAX {
            stack_len
        } else {
            count
        };
        // The host must charge the syscall before reporting insufficient arguments.
        let count = count.min(stack_len);
        self.roots.set(self.root_count.saturating_sub(count));
        Ok(stack_len - count)
    }
    fn import_result(
        &self,
        result: Result<(), SharedVmException>,
        ids: &mut CompoundIds,
    ) -> Result<(), RuntimeException> {
        match result {
            Ok(()) => Ok(()),
            Err(SharedVmException::Fault(message)) => Err(RuntimeException::Fault(message)),
            Err(SharedVmException::Throw(value)) => {
                Err(RuntimeException::Throw(ids.import_shared(value)?))
            }
        }
    }
}

impl<H: ReferenceSyscallProvider> RuntimeHost for SharedHost<'_, H> {
    fn check_references(&mut self, roots: usize) -> Result<(), String> {
        self.root_count = roots;
        self.roots.set(roots);
        if self.heap.reference_count() > 2048 {
            self.heap.collect();
            if self.heap.reference_count() > 2048 {
                return Err("reference count exceeds maximum".to_string());
            }
        }
        Ok(())
    }
    fn on_instruction(&mut self, opcode: u8) -> Result<(), String> {
        self.host.on_instruction(opcode)
    }
    fn complete_initializer(
        &mut self,
        ip: usize,
        _initial: &mut Vec<StackValue>,
        _static_fields: &mut Vec<StackValue>,
    ) -> Result<(), String> {
        self.host.initializer_complete(ip)
    }
    fn syscall(
        &mut self,
        api: u32,
        ip: usize,
        stack: &mut Vec<StackValue>,
        _args: &mut Vec<StackValue>,
        _locals: &mut Vec<StackValue>,
        _static_fields: &mut Vec<StackValue>,
        _mutations: &mut Vec<StackValue>,
        ids: &mut CompoundIds,
        _typed: bool,
        depth: usize,
    ) -> Result<(), RuntimeException> {
        let keep = self.detach_arguments(
            self.host.syscall_arg_count(api, ip).unwrap_or(usize::MAX),
            stack.len(),
        )?;
        let mut arguments = stack[keep..]
            .iter()
            .map(|value| export_shared(value, &self.heap))
            .collect::<Result<Vec<_>, _>>()?;
        // Consumed arguments must stop keeping the caller's old graph alive during a child call.
        stack.truncate(keep);
        let result = self.host.syscall(api, ip, depth, &mut arguments);
        // Restore only the host's remaining stack, retaining the prefix and every handle.
        for value in arguments {
            stack.push(ids.import_shared(value)?);
        }
        self.import_result(result, ids)
    }
    fn callt(
        &mut self,
        token: u16,
        ip: usize,
        stack: &mut Vec<StackValue>,
        _args: &mut Vec<StackValue>,
        _locals: &mut Vec<StackValue>,
        _static_fields: &mut Vec<StackValue>,
        _mutations: &mut Vec<StackValue>,
        ids: &mut CompoundIds,
        _typed: bool,
        depth: usize,
    ) -> Result<(), RuntimeException> {
        let keep = self.detach_arguments(
            self.host.callt_arg_count(token).unwrap_or(usize::MAX),
            stack.len(),
        )?;
        let mut arguments = stack[keep..]
            .iter()
            .map(|value| export_shared(value, &self.heap))
            .collect::<Result<Vec<_>, _>>()?;
        stack.truncate(keep);
        let result = self.host.callt(token, ip, depth, &mut arguments);
        for value in arguments {
            stack.push(ids.import_shared(value)?);
        }
        self.import_result(result, ids)
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn interpret_reference<H: ReferenceSyscallProvider>(
    script: &[u8],
    initial: Vec<SharedValue>,
    initial_ip: usize,
    initializer: Option<usize>,
    result_limit: Option<usize>,
    heap: SharedHeap,
    base_depth: usize,
    host: &mut H,
) -> Result<SharedExecutionResult, SharedVmException> {
    if base_depth == 0
        || base_depth
            .checked_add(usize::from(initializer.is_some()))
            .is_none_or(|depth| depth > crate::DEFAULT_MAX_INVOCATION_DEPTH as usize)
    {
        return Err(SharedVmException::Fault(
            "invocation depth exceeds maximum".to_string(),
        ));
    }
    let mut ids = CompoundIds::with_shared_heap(heap.clone());
    let initial = initial
        .into_iter()
        .map(|value| ids.import_shared(value))
        .collect::<Result<Vec<_>, _>>()?;
    let mut host = SharedHost {
        host,
        heap: heap.clone(),
        roots: heap.register_frame(),
        root_count: 0,
    };
    match execute(
        script,
        initial,
        ids,
        initial_ip,
        initializer,
        result_limit,
        true,
        true,
        base_depth,
        &mut host,
    ) {
        Ok(RuntimeResult::Halt(stack)) => Ok(SharedExecutionResult {
            fee_consumed_pico: 0,
            state: VmState::Halt,
            stack: stack
                .iter()
                .map(|value| export_shared(value, &heap))
                .collect::<Result<Vec<_>, _>>()?,
        }),
        Ok(RuntimeResult::LegacyFault(_)) => Err(SharedVmException::Fault(
            "unexpected legacy result in reference execution".to_string(),
        )),
        Err(RuntimeException::Fault(message)) => Err(SharedVmException::Fault(message)),
        Err(RuntimeException::Throw(value)) => {
            Err(SharedVmException::Throw(export_shared(&value, &heap)?))
        }
    }
}
