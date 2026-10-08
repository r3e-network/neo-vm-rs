use neo_vm_rs::reference::{SharedHeap, SharedValue};
use neo_vm_rs::{
    OpCode, ReferenceSyscallProvider, SharedVmException, StackValue, VmState,
    interpret_with_stack_and_syscalls_at_reference as run,
    interpret_with_stack_and_syscalls_at_with_initializer_and_result_limit_reference as run_initialized,
    interpret_with_stack_and_syscalls_at_with_result_limit_reference as run_limited,
};

const CALL: u32 = 0xdead_beef;

fn primitive(value: i64) -> SharedValue {
    SharedValue::primitive(StackValue::Integer(value)).unwrap()
}

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

#[derive(Default)]
struct Host {
    heap: SharedHeap,
    child: Vec<u8>,
    depths: Vec<usize>,
    initializers: usize,
    outcome: Option<SharedVmException>,
    arguments: Option<usize>,
}

impl ReferenceSyscallProvider for Host {
    fn callt_arg_count(&self, _token: u16) -> Result<usize, String> {
        Ok(self.arguments.unwrap_or(usize::MAX))
    }
    fn syscall(
        &mut self,
        _api: u32,
        _ip: usize,
        depth: usize,
        stack: &mut Vec<SharedValue>,
    ) -> Result<(), SharedVmException> {
        self.depths.push(depth);
        if let Some(error) = &self.outcome {
            return Err(error.clone());
        }
        if !self.child.is_empty() {
            let child = self.child.clone();
            let args = core::mem::take(stack);
            let result = run(&child, args, 0, self.heap.clone(), depth + 1, self)?;
            *stack = result.stack;
        }
        Ok(())
    }
    fn initializer_complete(&mut self, _ip: usize) -> Result<(), String> {
        self.initializers += 1;
        Ok(())
    }
}

#[test]
fn input_output_and_duplicated_values_keep_exact_identity() {
    let heap = SharedHeap::new();
    for value in [
        heap.array(vec![primitive(1)]).unwrap(),
        heap.structure(vec![primitive(1)]).unwrap(),
        heap.map(vec![(primitive(1), primitive(2))]).unwrap(),
        heap.buffer(vec![1]).unwrap(),
    ] {
        let result = run(
            &[OpCode::DUP as u8],
            vec![value.clone()],
            0,
            heap.clone(),
            1,
            &mut Host::default(),
        )
        .unwrap();
        assert_eq!(result.state, VmState::Halt);
        assert_eq!(result.stack.len(), 2);
        assert!(result.stack.iter().all(|item| item.same_reference(&value)));
    }
}

#[test]
fn uncaught_and_caught_throw_keep_original_handle() {
    let heap = SharedHeap::new();
    let value = heap.array(vec![primitive(1)]).unwrap();
    let error = run(
        &[OpCode::THROW as u8],
        vec![value.clone()],
        0,
        heap.clone(),
        1,
        &mut Host::default(),
    )
    .unwrap_err();
    let SharedVmException::Throw(thrown) = error else {
        panic!("expected ordinary VM throw")
    };
    assert!(thrown.same_reference(&value));
    let result = run(
        &catching(&[OpCode::THROW as u8]),
        vec![value.clone()],
        0,
        heap,
        1,
        &mut Host::default(),
    )
    .unwrap();
    assert!(result.stack[0].same_reference(&value));
}

#[test]
fn syscall_and_callt_return_the_same_mutated_child_array() {
    for body in [syscall(CALL), vec![OpCode::CALLT as u8, 0, 0]] {
        let heap = SharedHeap::new();
        let value = heap.array(vec![primitive(1)]).unwrap();
        let mut host = Host {
            heap: heap.clone(),
            child: vec![
                OpCode::DUP as u8,
                OpCode::PUSH0 as u8,
                OpCode::PUSH9 as u8,
                OpCode::SETITEM as u8,
            ],
            ..Host::default()
        };
        let result = run(&body, vec![value.clone()], 0, heap, 1, &mut host).unwrap();
        assert!(result.stack[0].same_reference(&value));
        assert_eq!(
            value.snapshot(8).unwrap(),
            StackValue::Array(vec![StackValue::Integer(9)])
        );
    }
}

#[test]
fn syscall_and_callt_catch_child_throw_without_copying_the_object() {
    for body in [syscall(CALL), vec![OpCode::CALLT as u8, 0, 0]] {
        let heap = SharedHeap::new();
        let value = heap.array(vec![primitive(1)]).unwrap();
        let mut host = Host {
            heap: heap.clone(),
            child: vec![
                OpCode::DUP as u8,
                OpCode::PUSH0 as u8,
                OpCode::PUSH9 as u8,
                OpCode::SETITEM as u8,
                OpCode::THROW as u8,
            ],
            ..Host::default()
        };
        let result = run(&catching(&body), vec![value.clone()], 0, heap, 1, &mut host).unwrap();
        assert_eq!(result.stack.len(), 1);
        assert!(result.stack[0].same_reference(&value));
        assert_eq!(
            value.snapshot(8).unwrap(),
            StackValue::Array(vec![StackValue::Integer(9)])
        );
    }
}

#[test]
fn fatal_host_error_bypasses_reference_catch() {
    let mut host = Host {
        outcome: Some(SharedVmException::Fault("native failure".into())),
        ..Host::default()
    };
    let result = run(
        &catching(&syscall(CALL)),
        vec![],
        0,
        SharedHeap::new(),
        1,
        &mut host,
    );
    assert!(
        matches!(result, Err(SharedVmException::Fault(message)) if message == "native failure")
    );
}

#[test]
fn call_frame_restoration_keeps_mutation_visible_through_a_local() {
    let heap = SharedHeap::new();
    let value = heap.array(vec![primitive(1)]).unwrap();
    let script = [
        OpCode::INITSLOT as u8,
        1,
        0,
        OpCode::DUP as u8,
        OpCode::STLOC0 as u8,
        OpCode::CALL as u8,
        4,
        OpCode::LDLOC0 as u8,
        OpCode::RET as u8,
        OpCode::PUSH0 as u8,
        OpCode::PUSH9 as u8,
        OpCode::SETITEM as u8,
        OpCode::RET as u8,
    ];
    let result = run(
        &script,
        vec![value.clone()],
        0,
        heap,
        1,
        &mut Host::default(),
    )
    .unwrap();
    assert!(result.stack[0].same_reference(&value));
    assert_eq!(
        value.snapshot(8).unwrap(),
        StackValue::Array(vec![StackValue::Integer(9)])
    );
}

#[test]
fn initializer_and_result_limit_preserve_the_retained_argument() {
    let heap = SharedHeap::new();
    let value = heap.array(vec![primitive(1)]).unwrap();
    let script = [OpCode::RET as u8, OpCode::RET as u8];
    let mut host = Host::default();
    let result =
        run_initialized(&script, vec![value.clone()], 1, 0, 1, heap, 1, &mut host).unwrap();
    assert_eq!(host.initializers, 1);
    assert_eq!(result.stack.len(), 1);
    assert!(result.stack[0].same_reference(&value));
}

#[test]
fn zero_result_limit_releases_shared_result_handles() {
    let heap = SharedHeap::new();
    let value = heap.array(vec![primitive(1)]).unwrap();
    let result = run_limited(
        &[OpCode::DUP as u8, OpCode::DROP as u8, OpCode::DROP as u8],
        vec![value],
        0,
        0,
        heap.clone(),
        1,
        &mut Host::default(),
    )
    .unwrap();
    assert!(result.stack.is_empty());
    // The dropped return values must not be leaked by the legacy arena convention.
    heap.collect();
    assert_eq!(heap.subitem_count(), 0);
}

#[test]
fn entry_and_internal_calls_share_the_global_depth_limit() {
    for depth in [0, 1025, usize::MAX] {
        assert!(matches!(
            run(
                &[],
                vec![],
                0,
                SharedHeap::new(),
                depth,
                &mut Host::default()
            ),
            Err(SharedVmException::Fault(_))
        ));
    }
    let script = [OpCode::CALL as u8, 3, OpCode::RET as u8, OpCode::RET as u8];
    assert!(
        run(
            &script,
            vec![],
            0,
            SharedHeap::new(),
            1023,
            &mut Host::default()
        )
        .is_ok()
    );
    assert!(matches!(
        run(
            &script,
            vec![],
            0,
            SharedHeap::new(),
            1024,
            &mut Host::default()
        ),
        Err(SharedVmException::Fault(_))
    ));
}

#[test]
fn host_observes_internal_depth_and_child_call_cannot_reset_it() {
    let mut script = vec![OpCode::CALL as u8, 3, OpCode::RET as u8];
    script.extend(syscall(CALL));
    script.push(OpCode::RET as u8);
    let mut host = Host {
        child: vec![OpCode::RET as u8],
        ..Host::default()
    };
    let heap = host.heap.clone();
    let error = run(&script, vec![], 0, heap, 1023, &mut host).unwrap_err();
    assert_eq!(host.depths, vec![1024]);
    assert!(matches!(error, SharedVmException::Fault(_)));
}

#[test]
fn shared_heap_mismatch_is_rejected_at_the_entry_boundary() {
    let value = SharedHeap::new().array(vec![primitive(1)]).unwrap();
    assert!(matches!(
        run(
            &[],
            vec![value],
            0,
            SharedHeap::new(),
            1,
            &mut Host::default()
        ),
        Err(SharedVmException::Fault(_))
    ));
}

#[test]
fn reference_limit_counts_collection_edges_and_all_stack_roots() {
    let heap = SharedHeap::new();
    let value = heap.array(vec![primitive(0); 2047]).unwrap();
    assert!(
        run(
            &[],
            vec![value.clone()],
            0,
            heap.clone(),
            1,
            &mut Host::default()
        )
        .is_ok()
    );
    assert!(matches!(
        run(
            &[OpCode::DUP as u8],
            vec![value],
            0,
            heap,
            1,
            &mut Host::default()
        ),
        Err(SharedVmException::Fault(_))
    ));
    let script = [OpCode::INITSLOT as u8, 49, 0, OpCode::RET as u8];
    assert!(matches!(
        run(
            &script,
            vec![primitive(0); 2000],
            0,
            SharedHeap::new(),
            1,
            &mut Host::default()
        ),
        Err(SharedVmException::Fault(_))
    ));
}

#[test]
fn nested_call_counts_parent_prefix_once_and_never_resets_the_limit() {
    for (child, accepted) in [
        (vec![OpCode::RET as u8], true),
        (vec![OpCode::PUSH1 as u8], false),
    ] {
        let mut host = Host {
            child,
            arguments: Some(1),
            ..Host::default()
        };
        let result = run(
            &[OpCode::CALLT as u8, 0, 0],
            vec![primitive(0); 2048],
            0,
            host.heap.clone(),
            1,
            &mut host,
        );
        assert_eq!(result.is_ok(), accepted);
        if let Ok(result) = result {
            assert_eq!(result.stack.len(), 2048);
        }
    }
}

#[test]
fn malformed_argument_metadata_still_reaches_host_fee_and_validation_order() {
    struct ChargingHost {
        calls: usize,
        bad_metadata: bool,
    }
    impl ReferenceSyscallProvider for ChargingHost {
        fn syscall_arg_count(&self, _api: u32, _ip: usize) -> Result<usize, String> {
            if self.bad_metadata {
                Err("unknown method".into())
            } else {
                Ok(4)
            }
        }
        fn syscall(
            &mut self,
            _api: u32,
            _ip: usize,
            _depth: usize,
            stack: &mut Vec<SharedValue>,
        ) -> Result<(), SharedVmException> {
            self.calls += 1;
            assert_eq!(stack.len(), 1);
            Err(SharedVmException::Fault(
                "host charged then rejected".into(),
            ))
        }
    }
    for bad_metadata in [false, true] {
        let mut host = ChargingHost {
            calls: 0,
            bad_metadata,
        };
        let result = run(
            &catching(&syscall(CALL)),
            vec![primitive(1)],
            0,
            SharedHeap::new(),
            1,
            &mut host,
        );
        assert_eq!(host.calls, 1);
        assert!(
            matches!(result, Err(SharedVmException::Fault(message)) if message == "host charged then rejected")
        );
    }
}

#[test]
fn initializer_occupies_an_additional_invocation_frame_until_it_returns() {
    let mut script = syscall(CALL);
    script.extend([OpCode::RET as u8, OpCode::RET as u8]);
    let mut host = Host::default();
    run_initialized(&script, vec![], 6, 0, 0, SharedHeap::new(), 1023, &mut host).unwrap();
    assert_eq!(host.depths, vec![1024]);
    let mut host = Host::default();
    assert!(matches!(
        run_initialized(&script, vec![], 6, 0, 0, SharedHeap::new(), 1024, &mut host),
        Err(SharedVmException::Fault(_))
    ));
    assert!(host.depths.is_empty());
    // The method runs after the initializer frame has left the invocation stack.
    let script = [vec![OpCode::RET as u8], syscall(CALL)].concat();
    let mut host = Host::default();
    run_initialized(&script, vec![], 1, 0, 0, SharedHeap::new(), 1023, &mut host).unwrap();
    assert_eq!(host.depths, vec![1023]);
}

#[test]
fn initializer_shares_the_method_evaluation_stack_and_retains_its_mutations() {
    let heap = SharedHeap::new();
    let value = heap.array(vec![primitive(1)]).unwrap();
    let script = [
        OpCode::DUP as u8,
        OpCode::PUSH0 as u8,
        OpCode::PUSH9 as u8,
        OpCode::SETITEM as u8,
        OpCode::RET as u8,
        OpCode::PUSH0 as u8,
        OpCode::PICKITEM as u8,
        OpCode::RET as u8,
    ];
    let result = run_initialized(
        &script,
        vec![value.clone()],
        5,
        0,
        1,
        heap,
        1,
        &mut Host::default(),
    )
    .unwrap();
    assert_eq!(
        result.stack[0].primitive_ref(),
        Some(&StackValue::Integer(9))
    );
    assert_eq!(
        value.snapshot(8).unwrap(),
        StackValue::Array(vec![StackValue::Integer(9)])
    );
}

#[test]
fn implicit_return_resumes_the_method_and_internal_call_continuation() {
    // _initialize ends at script EOF; run() must still execute afterward.
    let script = [OpCode::PUSH9 as u8, OpCode::RET as u8, OpCode::NOP as u8];
    let result = run_initialized(
        &script,
        vec![],
        0,
        2,
        1,
        SharedHeap::new(),
        1,
        &mut Host::default(),
    )
    .unwrap();
    assert_eq!(
        result.stack[0].primitive_ref(),
        Some(&StackValue::Integer(9))
    );
    // CALL's target ends at EOF; the parent must resume after CALL.
    let script = [
        OpCode::CALL as u8,
        4,
        OpCode::PUSH9 as u8,
        OpCode::RET as u8,
        OpCode::NOP as u8,
    ];
    let result = run(
        &script,
        vec![],
        0,
        SharedHeap::new(),
        1,
        &mut Host::default(),
    )
    .unwrap();
    assert_eq!(
        result.stack[0].primitive_ref(),
        Some(&StackValue::Integer(9))
    );
}

#[test]
fn consumed_argument_container_is_collectible_before_the_child_starts() {
    struct UnpackingHost {
        heap: SharedHeap,
    }
    impl ReferenceSyscallProvider for UnpackingHost {
        fn syscall(
            &mut self,
            _api: u32,
            _ip: usize,
            depth: usize,
            stack: &mut Vec<SharedValue>,
        ) -> Result<(), SharedVmException> {
            let arguments = stack.pop().unwrap().items()?;
            let result = run(&[], arguments, 0, self.heap.clone(), depth + 1, self)?;
            *stack = result.stack;
            Ok(())
        }
    }
    let heap = SharedHeap::new();
    let argument_array = heap.array(vec![primitive(0); 2047]).unwrap();
    let result = run(
        &syscall(CALL),
        vec![argument_array],
        0,
        heap.clone(),
        1,
        &mut UnpackingHost { heap },
    )
    .unwrap();
    assert_eq!(result.stack.len(), 2047);
}

#[test]
fn reference_return_count_requires_exact_void_or_single_result() {
    for (script, count, accepted) in [
        (vec![OpCode::RET as u8], 0, true),
        (vec![OpCode::PUSH1 as u8], 0, false),
        (vec![OpCode::PUSH1 as u8], 1, true),
        (vec![OpCode::RET as u8], 1, false),
        (vec![OpCode::PUSH1 as u8, OpCode::PUSH2 as u8], 1, false),
    ] {
        assert_eq!(
            run_limited(
                &script,
                vec![],
                0,
                count,
                SharedHeap::new(),
                1,
                &mut Host::default()
            )
            .is_ok(),
            accepted
        );
    }
}

#[test]
fn equality_jumps_compare_integers_and_reject_compound_operands() {
    for (opcode, expected) in [(OpCode::JMPEQ, 9), (OpCode::JMPNE, 8)] {
        let script = [
            opcode as u8,
            4,
            OpCode::PUSH8 as u8,
            OpCode::RET as u8,
            OpCode::PUSH9 as u8,
            OpCode::RET as u8,
        ];
        let values = vec![
            SharedValue::primitive(StackValue::ByteString(vec![1])).unwrap(),
            primitive(1),
        ];
        let result = run(
            &script,
            values,
            0,
            SharedHeap::new(),
            1,
            &mut Host::default(),
        )
        .unwrap();
        assert_eq!(
            result.stack[0].primitive_ref(),
            Some(&StackValue::Integer(expected))
        );
        let heap = SharedHeap::new();
        let array = heap.array(vec![]).unwrap();
        assert!(matches!(
            run(
                &script,
                vec![array.clone(), array],
                0,
                heap,
                1,
                &mut Host::default()
            ),
            Err(SharedVmException::Fault(_))
        ));
    }
}

#[test]
fn pointer_scope_survives_throw_but_rejects_a_different_script_entry() {
    let same_script = [
        OpCode::PUSHA as u8,
        8,
        0,
        0,
        0,
        OpCode::CALLA as u8,
        OpCode::PUSH9 as u8,
        OpCode::RET as u8,
        OpCode::RET as u8,
    ];
    let result = run(
        &same_script,
        vec![],
        0,
        SharedHeap::new(),
        1,
        &mut Host::default(),
    )
    .unwrap();
    assert_eq!(
        result.stack[0].primitive_ref(),
        Some(&StackValue::Integer(9))
    );

    let producer = [OpCode::PUSHA as u8, 0, 0, 0, 0, OpCode::RET as u8];
    let heap = SharedHeap::new();
    let pointer = run(&producer, vec![], 0, heap.clone(), 1, &mut Host::default())
        .unwrap()
        .stack
        .pop()
        .unwrap();
    assert!(matches!(
        run(
            &[OpCode::CALLA as u8],
            vec![pointer],
            0,
            heap,
            1,
            &mut Host::default()
        ),
        Err(SharedVmException::Fault(_))
    ));

    let mut script = vec![OpCode::PUSHA as u8, 20, 0, 0, 0];
    let mut catch = catching(&syscall(CALL));
    *catch.last_mut().unwrap() = OpCode::CALLA as u8;
    script.extend(catch);
    script.extend([OpCode::PUSH9 as u8, OpCode::RET as u8, OpCode::RET as u8]);
    assert_eq!(script.len(), 21);
    let mut host = Host {
        child: vec![OpCode::THROW as u8],
        ..Host::default()
    };
    let result = run(&script, vec![], 0, host.heap.clone(), 1, &mut host).unwrap();
    assert_eq!(
        result.stack[0].primitive_ref(),
        Some(&StackValue::Integer(9))
    );
}

#[test]
fn pointers_compare_loaded_script_identity_even_when_the_script_bytes_match() {
    let heap = SharedHeap::new();
    let producer = [OpCode::PUSHA as u8, 0, 0, 0, 0, OpCode::RET as u8];
    let first = run(&producer, vec![], 0, heap.clone(), 1, &mut Host::default())
        .unwrap()
        .stack
        .pop()
        .unwrap();
    let second = run(&producer, vec![], 0, heap.clone(), 1, &mut Host::default())
        .unwrap()
        .stack
        .pop()
        .unwrap();
    let result = run(
        &[OpCode::EQUAL as u8],
        vec![first, second],
        0,
        heap.clone(),
        1,
        &mut Host::default(),
    )
    .unwrap();
    assert_eq!(
        result.stack[0].primitive_ref(),
        Some(&StackValue::Boolean(false))
    );
    let script = [
        OpCode::PUSHA as u8,
        0,
        0,
        0,
        0,
        OpCode::DUP as u8,
        OpCode::EQUAL as u8,
    ];
    let result = run(&script, vec![], 0, heap, 1, &mut Host::default()).unwrap();
    assert_eq!(
        result.stack[0].primitive_ref(),
        Some(&StackValue::Boolean(true))
    );
}

#[test]
fn pointer_scope_is_retained_inside_array_and_struct_fields() {
    for pack in [OpCode::PACK, OpCode::PACKSTRUCT] {
        let script = [
            OpCode::PUSHA as u8,
            12,
            0,
            0,
            0,
            OpCode::PUSH1 as u8,
            pack as u8,
            OpCode::PUSH0 as u8,
            OpCode::PICKITEM as u8,
            OpCode::CALLA as u8,
            OpCode::PUSH9 as u8,
            OpCode::RET as u8,
            OpCode::RET as u8,
        ];
        let result = run(
            &script,
            vec![],
            0,
            SharedHeap::new(),
            1,
            &mut Host::default(),
        )
        .unwrap();
        assert_eq!(
            result.stack[0].primitive_ref(),
            Some(&StackValue::Integer(9))
        );
    }
}
