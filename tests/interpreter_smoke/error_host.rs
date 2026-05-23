use neo_vm_rs::{StackValue, SyscallProvider};

pub(super) struct ErrorHost;

impl SyscallProvider for ErrorHost {
    fn syscall(
        &mut self,
        _api: u32,
        _ip: usize,
        _stack: &mut Vec<StackValue>,
    ) -> Result<(), String> {
        Err("boom".to_string())
    }
}
