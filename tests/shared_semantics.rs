use neo_vm_rs::{interop_hash, syscall_arg_count, ExecutionResult, OpCode, StackValue, VmState};

#[test]
fn opcode_slot_assignments_are_canonical() {
    assert_eq!(OpCode::LDSFLD6.byte(), 0x5e);
    assert_eq!(OpCode::LDSFLD.byte(), 0x5f);
    assert_eq!(OpCode::LDLOC6.byte(), 0x6e);
    assert_eq!(OpCode::STARG6.byte(), 0x86);
    assert_eq!(OpCode::STARG.byte(), 0x87);
    assert_eq!(OpCode::try_from(0x87), Ok(OpCode::STARG));
}

#[test]
fn opcode_metadata_handles_fixed_and_prefixed_operands() {
    assert_eq!(OpCode::PUSHINT64.operand_size(), 8);
    assert_eq!(OpCode::PUSHDATA4.operand_size(), 4);
    assert_eq!(OpCode::PUSHDATA4.operand_prefix(), 4);
    assert_eq!(OpCode::SYSCALL.operand_size(), 4);
    assert_eq!(OpCode::CALLT.operand_size(), 2);
    assert_eq!(OpCode::ADD.name(), "ADD");
}

#[test]
fn stack_value_integer_and_boolean_semantics_are_shared() {
    assert_eq!(StackValue::ByteString(vec![0xff]).to_i128(), Some(-1));
    assert_eq!(StackValue::ByteString(vec![0x01, 0x00]).to_i128(), Some(1));
    assert!(!StackValue::ByteString(vec![0, 0]).to_bool());
    assert!(StackValue::ByteString(vec![0, 1]).to_bool());
    assert!(!StackValue::Null.to_bool());
}

#[test]
fn syscall_hash_and_argument_counts_are_shared() {
    assert_eq!(interop_hash("System.Contract.Call"), 0x525b_7d62);
    assert_eq!(syscall_arg_count(interop_hash("System.Contract.Call")), 4);
    assert_eq!(
        syscall_arg_count(interop_hash("System.Contract.CallNative")),
        usize::MAX
    );
}

#[test]
fn execution_result_keeps_fault_metadata_optional() {
    let result = ExecutionResult {
        fee_consumed_pico: 42,
        state: VmState::Fault,
        stack: vec![StackValue::Integer(7)],
        fault_message: Some("fault".to_string()),
        fault_ip: Some(3),
        fault_locals: Some(vec![1, 2, 3]),
    };

    assert_eq!(result.state, VmState::Fault);
    assert_eq!(result.stack, vec![StackValue::Integer(7)]);
    assert_eq!(result.fault_ip, Some(3));
}
