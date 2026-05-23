use neo_vm_rs::{StackValue, SyscallProvider};

pub(super) struct Host {
    pub(super) seen_api: Option<u32>,
}

impl SyscallProvider for Host {
    fn syscall(&mut self, api: u32, _ip: usize, stack: &mut Vec<StackValue>) -> Result<(), String> {
        self.seen_api = Some(api);
        stack.push(StackValue::Integer(42));
        Ok(())
    }
}
