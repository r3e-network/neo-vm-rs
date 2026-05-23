use neo_vm_rs::{
    instruction_jump_target, instruction_try_targets, interop_hash, parse_script_instructions,
    syscall_arg_count, validate_script, validate_strict_script, ExceptionHandlingContext,
    ExceptionHandlingState, ExecutionEngineLimits, ExecutionResult, Instruction, OpCode,
    StackItemType, StackValue, VmOrderedDictionary, VmState,
};

#[test]
fn historical_primitive_collection_semantics_are_shared() {
    use neo_vm_rs::semantics::collections;

    assert_eq!(collections::size(&StackValue::Integer(0)), Ok(0));
    assert_eq!(collections::size(&StackValue::Integer(128)), Ok(2));
    assert_eq!(collections::size(&StackValue::Boolean(false)), Ok(1));
    assert_eq!(collections::size(&StackValue::Boolean(true)), Ok(1));

    assert_eq!(
        collections::pick_item(&StackValue::Integer(128), &StackValue::Integer(0)),
        Ok(StackValue::Integer(128))
    );
    assert_eq!(
        collections::pick_item(&StackValue::Integer(128), &StackValue::Integer(1)),
        Ok(StackValue::Integer(0))
    );
    assert_eq!(
        collections::pick_item(&StackValue::Integer(128), &StackValue::Integer(-1)),
        Err("PICKITEM: byte index out of range".to_string())
    );
    assert_eq!(
        collections::pick_item(&StackValue::Integer(128), &StackValue::Integer(2)),
        Err("PICKITEM: byte index out of range".to_string())
    );
    assert_eq!(
        collections::pick_item(&StackValue::Boolean(true), &StackValue::Integer(0)),
        Ok(StackValue::Integer(1))
    );
    assert_eq!(
        collections::pick_item(&StackValue::Boolean(false), &StackValue::Integer(0)),
        Ok(StackValue::Integer(0))
    );
    assert_eq!(
        collections::pick_item(&StackValue::Boolean(false), &StackValue::Integer(-1)),
        Err("PICKITEM: byte index out of range".to_string())
    );
    assert_eq!(
        collections::pick_item(&StackValue::Boolean(false), &StackValue::Integer(1)),
        Err("PICKITEM: byte index out of range".to_string())
    );
}

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
fn opcode_names_roundtrip_through_canonical_metadata() {
    for opcode in OpCode::ALL {
        assert_eq!(OpCode::from_name(opcode.name()), Some(opcode));
        assert_eq!(
            OpCode::from_name(&opcode.name().to_ascii_lowercase()),
            Some(opcode)
        );
    }

    assert_eq!(OpCode::from_name("NOT_A_REAL_OPCODE"), None);
}

#[test]
fn instruction_parsing_uses_shared_opcode_metadata() {
    let script = vec![
        OpCode::PUSH1.byte(),
        OpCode::JMP.byte(),
        0x10,
        OpCode::PUSHDATA1.byte(),
        0x03,
        0x01,
        0x02,
        0x03,
    ];

    let push = Instruction::parse(&script, 0).expect("PUSH1 should parse");
    assert_eq!(push.opcode(), OpCode::PUSH1);
    assert_eq!(push.pointer(), 0);
    assert_eq!(push.size(), 1);
    assert!(push.operand().is_empty());

    let jump = Instruction::parse(&script, 1).expect("JMP should parse");
    assert_eq!(jump.opcode(), OpCode::JMP);
    assert_eq!(jump.size(), 2);
    assert_eq!(jump.operand_data(), &[0x10]);
    assert_eq!(jump.operand_as::<i8>(), Ok(16));

    let data = Instruction::parse(&script, 3).expect("PUSHDATA1 should parse");
    assert_eq!(data.opcode(), OpCode::PUSHDATA1);
    assert_eq!(data.size(), 5);
    assert_eq!(data.operand(), &[0x01, 0x02, 0x03]);

    let token = Instruction::new(OpCode::SYSCALL, &[1, 2, 3, 4]);
    assert_eq!(token.token_u32(), 0x0403_0201);
    assert_eq!(
        Instruction::parse(&[OpCode::PUSHDATA1.byte(), 3, 1], 0)
            .expect_err("truncated PUSHDATA should fail")
            .message(),
        "PUSHDATA1 operand size exceeds script bounds: 2 + 3 > 3"
    );
}

#[test]
fn script_validation_uses_shared_instruction_parser() {
    assert!(validate_strict_script(&[]).is_ok());
    assert!(validate_strict_script(&[OpCode::PUSH1.byte(), OpCode::RET.byte()]).is_ok());
    assert!(validate_strict_script(&[0xff]).is_err());
    assert!(validate_strict_script(&[OpCode::PUSHDATA1.byte(), 2, 1]).is_err());
    assert!(validate_strict_script(&[OpCode::JMP.byte(), 10, OpCode::RET.byte()]).is_err());
    assert!(validate_strict_script(&[
        OpCode::CONVERT.byte(),
        StackItemType::Any.to_byte(),
        OpCode::RET.byte(),
    ])
    .is_err());

    let relaxed = validate_script(&[OpCode::JMP.byte(), 10, OpCode::RET.byte()], false)
        .expect("relaxed validation should only parse instruction offsets");
    assert!(relaxed.has_instruction_at(0));
    assert!(relaxed.has_instruction_at(2));
    assert!(!relaxed.has_instruction_at(1));

    let script = [
        OpCode::PUSHDATA1.byte(),
        3,
        b'n',
        b'e',
        b'o',
        OpCode::SYSCALL.byte(),
        1,
        2,
        3,
        4,
        OpCode::RET.byte(),
    ];
    let instructions = parse_script_instructions(&script).expect("script should parse");
    assert_eq!(instructions.len(), 3);
    assert_eq!(instructions[0].pointer(), 0);
    assert_eq!(instructions[0].opcode(), OpCode::PUSHDATA1);
    assert_eq!(instructions[0].operand(), b"neo");
    assert_eq!(instructions[0].size(), 5);
    assert_eq!(instructions[1].token_u32(), 0x0403_0201);

    let jump = Instruction::parse(&[OpCode::JMP.byte(), 0, OpCode::RET.byte()], 0)
        .expect("jump should parse");
    assert_eq!(instruction_jump_target(&jump), Ok(2));

    let try_instruction = Instruction::parse(&[OpCode::TRY.byte(), 0, 0, OpCode::RET.byte()], 0)
        .expect("TRY should parse");
    assert_eq!(instruction_try_targets(&try_instruction), Ok((3, 3)));
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
fn vm_state_byte_mapping_matches_neo_vm() {
    assert_eq!(VmState::NONE.to_byte(), 0);
    assert_eq!(VmState::HALT.to_byte(), 1);
    assert_eq!(VmState::FAULT.to_byte(), 2);
    assert_eq!(VmState::BREAK.to_byte(), 4);

    assert_eq!(VmState::from_byte(0), VmState::None);
    assert_eq!(VmState::from_byte(1), VmState::Halt);
    assert_eq!(VmState::from_byte(2), VmState::Fault);
    assert_eq!(VmState::from_byte(4), VmState::Break);
    assert_eq!(VmState::from_byte(3), VmState::None);

    assert!(VmState::HALT.contains(VmState::HALT));
    assert!(VmState::FAULT.is_fault());
    assert!(VmState::BREAK.is_break());
    assert!(VmState::NONE.is_none());
    assert_eq!(VmState::HALT.final_name(), Some("HALT"));
    assert_eq!(VmState::FAULT.final_name(), Some("FAULT"));
    assert_eq!(VmState::BREAK.final_name(), None);
}

#[test]
fn stack_item_type_byte_mapping_matches_neo_vm() {
    assert_eq!(StackItemType::Any.to_byte(), 0x00);
    assert_eq!(StackItemType::Pointer.to_byte(), 0x10);
    assert_eq!(StackItemType::Boolean.to_byte(), 0x20);
    assert_eq!(StackItemType::Integer.to_byte(), 0x21);
    assert_eq!(StackItemType::ByteString.to_byte(), 0x28);
    assert_eq!(StackItemType::Buffer.to_byte(), 0x30);
    assert_eq!(StackItemType::Array.to_byte(), 0x40);
    assert_eq!(StackItemType::Struct.to_byte(), 0x41);
    assert_eq!(StackItemType::Map.to_byte(), 0x48);
    assert_eq!(StackItemType::InteropInterface.to_byte(), 0x60);

    assert_eq!(StackItemType::from_byte(0x21), Some(StackItemType::Integer));
    assert_eq!(
        StackItemType::from_byte(0x60),
        Some(StackItemType::InteropInterface)
    );
    assert_eq!(StackItemType::from_byte(0xff), None);
}

#[test]
fn execution_engine_limits_match_neo_vm_defaults() {
    let limits = ExecutionEngineLimits::default();

    assert_eq!(limits.max_shift, 256);
    assert_eq!(
        limits.max_stack_size,
        neo_vm_rs::DEFAULT_MAX_STACK_DEPTH as u32
    );
    assert_eq!(limits.max_item_size, u16::MAX as u32);
    assert_eq!(limits.max_comparable_size, u16::MAX as u32);
    assert_eq!(
        limits.max_invocation_stack_size,
        neo_vm_rs::DEFAULT_MAX_INVOCATION_DEPTH as u32
    );
    assert_eq!(limits.max_try_nesting_depth, 16);
    assert!(limits.catch_engine_exceptions);
    assert_eq!(limits.max_instructions, 1_000_000);

    assert_eq!(limits.assert_shift(0), Ok(()));
    assert_eq!(limits.assert_shift(256), Ok(()));
    assert_eq!(
        limits.assert_shift(257),
        Err("Invalid shift value: 257/256".to_string())
    );
    assert_eq!(
        limits.assert_max_item_size(u16::MAX as usize + 1),
        Err("MaxItemSize exceed: 65536/65535".to_string())
    );
}

#[test]
fn exception_handling_context_matches_neo_vm_shape() {
    let mut context = ExceptionHandlingContext::new(-1, 30);

    assert_eq!(context.catch_pointer(), -1);
    assert_eq!(context.finally_pointer(), 30);
    assert_eq!(context.end_pointer(), -1);
    assert_eq!(context.state(), ExceptionHandlingState::Try);
    assert!(!context.has_catch());
    assert!(context.has_finally());
    assert!(!context.is_in_exception());

    context.set_end_pointer(100);
    context.set_state(ExceptionHandlingState::Catch);
    assert_eq!(context.end_pointer(), 100);
    assert_eq!(context.state(), ExceptionHandlingState::Catch);
    assert!(context.is_in_exception());

    context.set_state(ExceptionHandlingState::Finally);
    assert_eq!(context.state(), ExceptionHandlingState::Finally);
}

#[test]
fn ordered_dictionary_preserves_insertion_order() {
    let mut items = VmOrderedDictionary::new();

    assert_eq!(items.insert(3, 30), None);
    assert_eq!(items.insert(1, 10), None);
    assert_eq!(items.insert(2, 20), None);
    assert_eq!(items.insert(1, 11), Some(10));

    let entries = items
        .iter()
        .map(|(key, value)| (*key, *value))
        .collect::<Vec<_>>();
    assert_eq!(entries, vec![(3, 30), (1, 11), (2, 20)]);
    assert_eq!(items.get(&1), Some(&11));
    assert!(items.contains_key(&3));
    assert_eq!(items.remove(&3), Some(30));

    let entries = items.into_iter().collect::<Vec<_>>();
    assert_eq!(entries, vec![(1, 11), (2, 20)]);
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
