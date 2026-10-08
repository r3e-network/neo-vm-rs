use neo_vm_rs::{
    ExecutionResult, OpCode, StackValue, SyscallProvider, VmException, VmState,
    interpret_with_stack_and_syscalls_at, interpret_with_stack_and_syscalls_at_typed,
    interpret_with_stack_and_syscalls_at_with_initializer_and_result_limit_typed,
    interpret_with_stack_and_syscalls_at_with_initializer_typed,
    interpret_with_stack_and_syscalls_at_with_result_limit_typed,
};

const CALL: u32 = 0xdead_beef;
const TRACE: u32 = 0xdead_bee0;

fn syscall(api: u32) -> Vec<u8> {
    [vec![OpCode::SYSCALL as u8], api.to_le_bytes().to_vec()].concat()
}

fn catching(body: &[u8]) -> Vec<u8> {
    let mut script = vec![OpCode::TRY as u8, (body.len() + 5) as u8, 0];
    script.extend(body);
    script.extend([
        OpCode::ENDTRY as u8,
        4,
        OpCode::ENDTRY as u8,
        2,
        OpCode::RET as u8,
    ]);
    script
}

fn finalizing(body: &[u8], finalizer: &[u8]) -> Vec<u8> {
    let mut script = vec![OpCode::TRY as u8, 0, (body.len() + 5) as u8];
    script.extend(body);
    script.extend([OpCode::ENDTRY as u8, (finalizer.len() + 3) as u8]);
    script.extend(finalizer);
    script.extend([OpCode::ENDFINALLY as u8, OpCode::RET as u8]);
    script
}

#[derive(Default)]
struct Host {
    outcome: Option<VmException>,
    traces: usize,
    instruction_fault: Option<u8>,
    initializers: usize,
}

impl SyscallProvider for Host {
    fn on_instruction(&mut self, opcode: u8) -> Result<(), String> {
        if self.instruction_fault == Some(opcode) {
            return Err("gas exhausted".into());
        }
        Ok(())
    }

    fn syscall(
        &mut self,
        _api: u32,
        _ip: usize,
        _stack: &mut Vec<StackValue>,
    ) -> Result<(), String> {
        Err("legacy provider error".into())
    }

    fn typed_syscall(
        &mut self,
        api: u32,
        _ip: usize,
        _stack: &mut Vec<StackValue>,
    ) -> Result<(), VmException> {
        if api == TRACE {
            self.traces += 1;
            return Ok(());
        }
        Err(self
            .outcome
            .clone()
            .expect("test must choose the host outcome"))
    }

    fn typed_callt(
        &mut self,
        _token: u16,
        ip: usize,
        stack: &mut Vec<StackValue>,
    ) -> Result<(), VmException> {
        self.typed_syscall(CALL, ip, stack)
    }

    fn initializer_complete(&mut self, _ip: usize) -> Result<(), String> {
        self.initializers += 1;
        Ok(())
    }
}

fn values() -> Vec<StackValue> {
    vec![
        StackValue::Null,
        StackValue::Boolean(true),
        StackValue::Integer(-7),
        StackValue::BigInteger(vec![0, 0, 0, 0, 0, 0, 0, 0, 1]),
        StackValue::ByteString(vec![0, 255]),
        StackValue::Buffer(vec![1, 2]),
        StackValue::Array(vec![StackValue::Integer(1), StackValue::Null]),
        StackValue::Struct(vec![StackValue::Boolean(false)]),
        StackValue::Map(vec![(StackValue::Boolean(true), StackValue::Integer(2))]),
    ]
}

#[test]
fn explicit_throw_preserves_the_typed_item_caught_and_uncaught() {
    for value in values() {
        let mut host = Host::default();
        let result = interpret_with_stack_and_syscalls_at_typed(
            &[OpCode::THROW as u8],
            vec![value.clone()],
            0,
            &mut host,
        );
        assert_eq!(result, Err(VmException::Throw(value.clone())));
        let result = interpret_with_stack_and_syscalls_at_typed(
            &catching(&[OpCode::THROW as u8]),
            vec![value.clone()],
            0,
            &mut host,
        )
        .unwrap();
        assert_eq!(result.state, VmState::Halt);
        assert_eq!(result.stack, vec![value]);
    }
}

#[test]
fn typed_syscall_and_callt_preserve_throw_but_never_catch_fault() {
    for body in [syscall(CALL), vec![OpCode::CALLT as u8, 0, 0]] {
        for value in values() {
            let mut host = Host {
                outcome: Some(VmException::Throw(value.clone())),
                ..Host::default()
            };
            let result =
                interpret_with_stack_and_syscalls_at_typed(&catching(&body), vec![], 0, &mut host)
                    .unwrap();
            assert_eq!(result.stack, vec![value.clone()]);
            assert_eq!(
                interpret_with_stack_and_syscalls_at_typed(&body, vec![], 0, &mut host),
                Err(VmException::Throw(value))
            );
        }
        let mut host = Host {
            outcome: Some(VmException::Fault("native failure".into())),
            ..Host::default()
        };
        assert_eq!(
            interpret_with_stack_and_syscalls_at_typed(&catching(&body), vec![], 0, &mut host),
            Err(VmException::Fault("native failure".into()))
        );
        assert_eq!(
            interpret_with_stack_and_syscalls_at_typed(
                &finalizing(&body, &syscall(TRACE)),
                vec![],
                0,
                &mut host
            ),
            Err(VmException::Fault("native failure".into()))
        );
        assert_eq!(host.traces, 0);
    }
}

#[test]
fn legacy_entry_points_do_not_call_the_typed_provider_hooks() {
    for body in [syscall(CALL), vec![OpCode::CALLT as u8, 0, 0]] {
        let mut host = Host {
            outcome: Some(VmException::Fault("typed fatal".into())),
            ..Host::default()
        };
        let legacy =
            interpret_with_stack_and_syscalls_at(&catching(&body), vec![], 0, &mut host).unwrap();
        assert_eq!(
            legacy.stack,
            vec![StackValue::ByteString(b"legacy provider error".to_vec())]
        );
        assert_eq!(
            interpret_with_stack_and_syscalls_at(&body, vec![], 0, &mut host).unwrap_err(),
            "legacy provider error"
        );
        assert!(matches!(
            interpret_with_stack_and_syscalls_at_typed(&catching(&body), vec![], 0, &mut host),
            Err(VmException::Fault(_))
        ));
    }
}

#[test]
fn default_typed_hooks_adapt_legacy_errors_to_catchable_bytes() {
    struct Legacy;
    impl SyscallProvider for Legacy {
        fn syscall(
            &mut self,
            _api: u32,
            _ip: usize,
            _stack: &mut Vec<StackValue>,
        ) -> Result<(), String> {
            Err("old message".into())
        }
    }
    for body in [syscall(CALL), vec![OpCode::CALLT as u8, 0, 0]] {
        let mut host = Legacy;
        let result =
            interpret_with_stack_and_syscalls_at_typed(&catching(&body), vec![], 0, &mut host)
                .unwrap();
        assert_eq!(
            result.stack,
            vec![StackValue::ByteString(b"old message".to_vec())]
        );
    }
}

#[test]
fn abort_assert_and_gas_exhaustion_are_fatal_even_inside_try() {
    let bodies = [
        vec![OpCode::ABORT as u8],
        vec![OpCode::PUSH0 as u8, OpCode::ASSERT as u8],
        vec![OpCode::PUSHDATA1 as u8, 1, b'x', OpCode::ABORTMSG as u8],
        vec![
            OpCode::PUSH0 as u8,
            OpCode::PUSHDATA1 as u8,
            1,
            b'x',
            OpCode::ASSERTMSG as u8,
        ],
    ];
    for body in bodies {
        let mut host = Host::default();
        assert!(matches!(
            interpret_with_stack_and_syscalls_at_typed(&catching(&body), vec![], 0, &mut host),
            Err(VmException::Fault(_))
        ));
        let legacy =
            interpret_with_stack_and_syscalls_at(&catching(&body), vec![], 0, &mut host).unwrap();
        assert_eq!(legacy.state, VmState::Fault);
    }
    let mut host = Host {
        instruction_fault: Some(OpCode::PUSH1 as u8),
        ..Host::default()
    };
    assert_eq!(
        interpret_with_stack_and_syscalls_at_typed(
            &catching(&[OpCode::PUSH1 as u8]),
            vec![],
            0,
            &mut host
        ),
        Err(VmException::Fault("gas exhausted".into()))
    );
}

#[test]
fn finally_executes_before_the_original_typed_exception_escapes() {
    for value in values() {
        let mut host = Host::default();
        let script = finalizing(&[OpCode::THROW as u8], &syscall(TRACE));
        assert_eq!(
            interpret_with_stack_and_syscalls_at_typed(&script, vec![value.clone()], 0, &mut host),
            Err(VmException::Throw(value))
        );
        assert_eq!(host.traces, 1);
    }
}

#[test]
fn outer_catch_runs_after_inner_finally() {
    let mut host = Host::default();
    let mut inner = finalizing(&[OpCode::THROW as u8], &syscall(TRACE));
    inner.pop();
    let result = interpret_with_stack_and_syscalls_at_typed(
        &catching(&inner),
        vec![StackValue::Integer(7)],
        0,
        &mut host,
    )
    .unwrap();
    assert_eq!(result.stack, vec![StackValue::Integer(7)]);
    assert_eq!(host.traces, 1);
}

#[test]
fn throw_inside_catch_runs_its_finally_and_preserves_the_new_value() {
    let mut script = vec![
        OpCode::TRY as u8,
        7,
        11,
        OpCode::PUSH1 as u8,
        OpCode::THROW as u8,
        OpCode::ENDTRY as u8,
        12,
        OpCode::PUSH2 as u8,
        OpCode::THROW as u8,
        OpCode::ENDTRY as u8,
        8,
    ];
    script.extend(syscall(TRACE));
    script.extend([OpCode::ENDFINALLY as u8, OpCode::RET as u8]);
    let mut host = Host::default();
    assert_eq!(
        interpret_with_stack_and_syscalls_at_typed(&script, vec![], 0, &mut host),
        Err(VmException::Throw(StackValue::Integer(2)))
    );
    assert_eq!(host.traces, 1);
}

#[test]
fn throw_inside_finally_supersedes_the_deferred_exception() {
    let script = finalizing(
        &[OpCode::PUSH1 as u8, OpCode::THROW as u8],
        &[OpCode::PUSH2 as u8, OpCode::THROW as u8],
    );
    assert_eq!(
        interpret_with_stack_and_syscalls_at_typed(&script, vec![], 0, &mut Host::default()),
        Err(VmException::Throw(StackValue::Integer(2)))
    );
}

#[test]
fn nested_catch_inside_finally_matches_the_core_single_uncaught_exception_slot() {
    let mut finalizer = catching(&[OpCode::PUSH2 as u8, OpCode::THROW as u8]);
    finalizer.pop();
    finalizer.push(OpCode::DROP as u8);
    let script = finalizing(&[OpCode::PUSH1 as u8, OpCode::THROW as u8], &finalizer);
    // The inner catch clears Neo's global UncaughtException. The outer
    // finally then has no ENDTRY destination (native default -1), so it faults.
    assert!(matches!(
        interpret_with_stack_and_syscalls_at_typed(&script, vec![], 0, &mut Host::default()),
        Err(VmException::Fault(_))
    ));
}

#[test]
fn child_execution_propagates_ordinary_throw_and_native_boundary_can_classify_fault() {
    struct ChildHost {
        native: bool,
    }
    impl SyscallProvider for ChildHost {
        fn syscall(
            &mut self,
            _api: u32,
            _ip: usize,
            _stack: &mut Vec<StackValue>,
        ) -> Result<(), String> {
            unreachable!("typed execution must use the typed hook")
        }
        fn typed_syscall(
            &mut self,
            _api: u32,
            _ip: usize,
            _stack: &mut Vec<StackValue>,
        ) -> Result<(), VmException> {
            let result = interpret_with_stack_and_syscalls_at_typed(
                &[OpCode::PUSHT as u8, OpCode::THROW as u8],
                vec![],
                0,
                &mut Host::default(),
            );
            match result {
                Err(error) if self.native => Err(VmException::Fault(error.to_string())),
                Err(error) => Err(error),
                Ok(_) => unreachable!(),
            }
        }
    }
    let script = catching(&syscall(CALL));
    let result = interpret_with_stack_and_syscalls_at_typed(
        &script,
        vec![],
        0,
        &mut ChildHost { native: false },
    )
    .unwrap();
    assert_eq!(result.stack, vec![StackValue::Boolean(true)]);
    assert!(matches!(
        interpret_with_stack_and_syscalls_at_typed(
            &script,
            vec![],
            0,
            &mut ChildHost { native: true }
        ),
        Err(VmException::Fault(_))
    ));
}

#[test]
fn initializer_and_result_limit_entry_points_use_the_same_typed_contract() {
    let mut host = Host::default();
    let script = [OpCode::THROW as u8, OpCode::RET as u8];
    let value = StackValue::ByteString(vec![1, 2, 3]);
    assert_eq!(
        interpret_with_stack_and_syscalls_at_with_initializer_typed(
            &script,
            vec![value.clone()],
            0,
            1,
            &mut host
        ),
        Err(VmException::Throw(value.clone()))
    );
    assert_eq!(
        interpret_with_stack_and_syscalls_at_with_initializer_and_result_limit_typed(
            &script,
            vec![value.clone()],
            0,
            1,
            0,
            &mut host
        ),
        Err(VmException::Throw(value.clone()))
    );
    assert_eq!(host.initializers, 2);
    assert_eq!(
        interpret_with_stack_and_syscalls_at_with_result_limit_typed(
            &script,
            vec![value],
            0,
            0,
            &mut host
        )
        .unwrap_err(),
        VmException::Throw(StackValue::ByteString(vec![1, 2, 3]))
    );
    let result: ExecutionResult = interpret_with_stack_and_syscalls_at_with_result_limit_typed(
        &[OpCode::PUSH1 as u8, OpCode::PUSH2 as u8],
        vec![],
        0,
        1,
        &mut host,
    )
    .unwrap();
    assert_eq!(result.stack, vec![StackValue::Integer(2)]);
}

#[test]
fn caught_host_throw_keeps_argument_consumption_in_the_calling_stack() {
    struct ConsumingHost;
    impl SyscallProvider for ConsumingHost {
        fn syscall(
            &mut self,
            _api: u32,
            _ip: usize,
            _stack: &mut Vec<StackValue>,
        ) -> Result<(), String> {
            unreachable!()
        }
        fn typed_syscall(
            &mut self,
            _api: u32,
            _ip: usize,
            stack: &mut Vec<StackValue>,
        ) -> Result<(), VmException> {
            for _ in 0..4 {
                stack.pop().unwrap();
            }
            Err(VmException::Throw(StackValue::Boolean(false)))
        }
        fn typed_callt(
            &mut self,
            _token: u16,
            ip: usize,
            stack: &mut Vec<StackValue>,
        ) -> Result<(), VmException> {
            self.typed_syscall(CALL, ip, stack)
        }
    }
    for body in [
        syscall(neo_vm_rs::interop_hash("System.Contract.Call")),
        vec![OpCode::CALLT as u8, 0, 0],
    ] {
        let initial = vec![
            StackValue::Integer(99),
            StackValue::Integer(1),
            StackValue::Integer(2),
            StackValue::Integer(3),
            StackValue::Integer(4),
        ];
        let result = interpret_with_stack_and_syscalls_at_typed(
            &catching(&body),
            initial,
            0,
            &mut ConsumingHost,
        )
        .unwrap();
        assert_eq!(
            result.stack,
            vec![StackValue::Integer(99), StackValue::Boolean(false)]
        );
    }
}

#[test]
fn typed_exception_control_rejects_missing_handlers_and_endtry_inside_finally() {
    let invalid = [
        vec![OpCode::TRY as u8, 0, 0, OpCode::RET as u8],
        vec![
            OpCode::TRY_L as u8,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            OpCode::RET as u8,
        ],
        vec![OpCode::ENDTRY as u8, 2, OpCode::RET as u8],
        vec![OpCode::ENDTRY_L as u8, 5, 0, 0, 0, OpCode::RET as u8],
        finalizing(&[OpCode::NOP as u8], &[OpCode::ENDTRY as u8, 2]),
    ];
    for script in invalid {
        assert!(matches!(
            interpret_with_stack_and_syscalls_at_typed(&script, vec![], 0, &mut Host::default()),
            Err(VmException::Fault(_))
        ));
    }
}
