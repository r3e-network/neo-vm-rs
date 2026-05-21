use neo_vm_rs::{
    interpret, interpret_with_stack_and_syscalls_at, ExecutionResult, StackValue, SyscallProvider,
    VmState,
};

#[test]
fn executes_basic_arithmetic_script() {
    let result = interpret(&[0x12, 0x13, 0x9e, 0x40]).expect("script should execute");

    assert_eq!(result.state, VmState::Halt);
    assert_eq!(result.stack, vec![StackValue::Integer(5)]);
}

#[test]
fn executes_local_slot_round_trip() {
    let result = interpret(&[
        0x57, 0x01, 0x00, // INITSLOT locals=1, args=0
        0x15, // PUSH5
        0x70, // STLOC0
        0x68, // LDLOC0
        0x40, // RET
    ])
    .expect("slot script should execute");

    assert_eq!(result.state, VmState::Halt);
    assert_eq!(result.stack, vec![StackValue::Integer(5)]);
}

#[test]
fn delegates_syscall_to_host_provider() {
    struct Host {
        seen_api: Option<u32>,
    }

    impl SyscallProvider for Host {
        fn syscall(
            &mut self,
            api: u32,
            _ip: usize,
            stack: &mut Vec<StackValue>,
        ) -> Result<(), String> {
            self.seen_api = Some(api);
            stack.push(StackValue::Integer(42));
            Ok(())
        }
    }

    let mut host = Host { seen_api: None };
    let result = interpret_with_stack_and_syscalls_at(
        &[
            0x41, 0x78, 0x56, 0x34, 0x12, // SYSCALL 0x12345678
            0x40, // RET
        ],
        Vec::new(),
        0,
        &mut host,
    )
    .expect("syscall script should execute");

    assert_eq!(host.seen_api, Some(0x1234_5678));
    assert_eq!(result.stack, vec![StackValue::Integer(42)]);
}

#[test]
fn catches_syscall_faults_with_try() {
    struct ErrorHost;

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

    let mut host = ErrorHost;
    let result = interpret_with_stack_and_syscalls_at(
        &[
            0x3b, 0x0a, 0x00, // TRY catch_offset=10, finally_offset=0
            0x41, 0xde, 0xad, 0xde, 0xad, // SYSCALL
            0x3d, 0x05, // ENDTRY offset=5
            0x11, // PUSH1
            0x3d, 0x02, // ENDTRY offset=2
            0x12, // PUSH2
        ],
        Vec::new(),
        0,
        &mut host,
    )
    .expect("try/catch script should execute");

    assert_eq!(result.state, VmState::Halt);
    assert_eq!(result.stack.len(), 3);
}

fn _assert_result_is_public(_: ExecutionResult) {}
