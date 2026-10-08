use neo_vm_rs::reference::{SharedHeap, SharedValue};
use neo_vm_rs::{
    OpCode, ReferenceSyscallProvider, SharedVmException, StackItemType, StackValue,
    interpret_with_stack_and_syscalls_at_reference as run,
};
struct NoHost;
impl ReferenceSyscallProvider for NoHost {
    fn syscall(
        &mut self,
        _: u32,
        _: usize,
        _: usize,
        _: &mut Vec<SharedValue>,
    ) -> Result<(), SharedVmException> {
        Err(SharedVmException::Fault("unexpected host call".into()))
    }
}
fn value(v: StackValue) -> SharedValue {
    SharedValue::primitive(v).unwrap()
}
fn bytes(n: usize) -> SharedValue {
    value(StackValue::ByteString(vec![1; n]))
}
fn execute(
    code: &[u8],
    args: Vec<SharedValue>,
    heap: SharedHeap,
) -> Result<Vec<SharedValue>, SharedVmException> {
    run(code, args, 0, heap, 1, &mut NoHost).map(|r| r.stack)
}
#[test]
fn bytestring_comparison_limit_checks_both_bytestring_operands_and_left_dispatch() {
    for op in [OpCode::EQUAL, OpCode::NOTEQUAL] {
        for (a, b) in [
            (bytes(65536), bytes(1)),
            (bytes(1), bytes(65536)),
            (bytes(65536), value(StackValue::Null)),
        ] {
            assert!(execute(&[op as u8], vec![a, b], SharedHeap::new()).is_err());
        }
        let result = execute(
            &[op as u8],
            vec![bytes(65535), bytes(65535)],
            SharedHeap::new(),
        )
        .unwrap();
        assert_eq!(
            result[0].primitive_ref(),
            Some(&StackValue::Boolean(op == OpCode::EQUAL))
        );
        let result = execute(
            &[op as u8],
            vec![value(StackValue::Null), bytes(65536)],
            SharedHeap::new(),
        )
        .unwrap();
        assert_eq!(
            result[0].primitive_ref(),
            Some(&StackValue::Boolean(op == OpCode::NOTEQUAL))
        );
    }
}
#[test]
fn bytestring_boolean_limits_cover_control_assert_and_binary_operations() {
    for code in [
        vec![OpCode::JMPIF as u8, 2],
        vec![OpCode::JMPIFNOT as u8, 2],
        vec![OpCode::ASSERT as u8],
        vec![OpCode::NOT as u8],
        vec![OpCode::CONVERT as u8, StackItemType::Boolean as u8],
    ] {
        assert!(
            execute(&code, vec![bytes(33)], SharedHeap::new()).is_err(),
            "{code:?}"
        );
        assert!(
            execute(&code, vec![bytes(32)], SharedHeap::new()).is_ok(),
            "{code:?}"
        );
    }
    for op in [OpCode::BOOLAND, OpCode::BOOLOR] {
        assert!(
            execute(
                &[op as u8],
                vec![bytes(33), value(StackValue::Boolean(true))],
                SharedHeap::new()
            )
            .is_err()
        );
        assert!(
            execute(
                &[op as u8],
                vec![bytes(32), value(StackValue::Boolean(true))],
                SharedHeap::new()
            )
            .is_ok()
        );
    }
    assert!(
        execute(
            &[OpCode::ASSERTMSG as u8],
            vec![bytes(33), bytes(1)],
            SharedHeap::new()
        )
        .is_err()
    );
    assert!(
        execute(
            &[OpCode::ASSERTMSG as u8],
            vec![bytes(32), bytes(33)],
            SharedHeap::new()
        )
        .is_ok()
    );
}
#[test]
fn integer_buffer_faults_preserve_explicit_conversion_and_zero_shift_short_circuit() {
    let heap = SharedHeap::new();
    let buffer = heap.buffer(vec![1]).unwrap();
    let result = execute(
        &[OpCode::SHL as u8],
        vec![buffer.clone(), value(StackValue::Integer(0))],
        heap.clone(),
    )
    .unwrap();
    assert!(result[0].same_reference(&buffer));
    assert!(
        execute(
            &[OpCode::SHL as u8],
            vec![buffer.clone(), value(StackValue::Integer(1))],
            heap.clone()
        )
        .is_err()
    );
    assert!(
        execute(
            &[OpCode::ADD as u8],
            vec![buffer.clone(), value(StackValue::Integer(1))],
            heap.clone()
        )
        .is_err()
    );
    assert!(
        execute(
            &[OpCode::CONVERT as u8, StackItemType::Integer as u8],
            vec![buffer],
            heap
        )
        .is_ok()
    );
    for n in [0, 1, 32] {
        assert!(execute(&[OpCode::INC as u8], vec![bytes(n)], SharedHeap::new()).is_ok());
    }
    assert!(execute(&[OpCode::INC as u8], vec![bytes(33)], SharedHeap::new()).is_err());
}

#[test]
fn ordering_comparisons_short_circuit_null_without_coercing_the_other_operand() {
    for op in [OpCode::LT, OpCode::LE, OpCode::GT, OpCode::GE] {
        for operands in [
            vec![value(StackValue::Null), bytes(33)],
            vec![bytes(33), value(StackValue::Null)],
        ] {
            let result = execute(&[op as u8], operands, SharedHeap::new()).unwrap();
            assert_eq!(result[0].primitive_ref(), Some(&StackValue::Boolean(false)));
        }
    }
}
