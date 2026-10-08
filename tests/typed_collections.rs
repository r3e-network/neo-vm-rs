use neo_vm_rs::{
    ExecutionResult, OpCode, StackValue, SyscallProvider, VmException,
    interpret_with_stack_and_syscalls_at, interpret_with_stack_and_syscalls_at_typed,
};

struct NoCalls;

impl SyscallProvider for NoCalls {
    fn syscall(&mut self, _: u32, _: usize, _: &mut Vec<StackValue>) -> Result<(), String> {
        Err("unexpected syscall".into())
    }
}

fn catch(opcode: OpCode) -> Vec<u8> {
    vec![
        OpCode::TRY.byte(),
        6,
        0,
        opcode.byte(),
        OpCode::ENDTRY.byte(),
        4,
        OpCode::ENDTRY.byte(),
        2,
        OpCode::RET.byte(),
    ]
}

fn typed(opcode: OpCode, stack: Vec<StackValue>) -> Result<ExecutionResult, VmException> {
    interpret_with_stack_and_syscalls_at_typed(&catch(opcode), stack, 0, &mut NoCalls)
}

fn expect_caught(opcode: OpCode, stack: Vec<StackValue>, message: &str) {
    let result = typed(opcode, stack).unwrap();
    assert_eq!(
        result.stack,
        vec![StackValue::ByteString(message.as_bytes().to_vec())]
    );
}

#[test]
fn pickitem_catches_only_native_collection_bounds_with_exact_payload() {
    let cases = [
        (StackValue::Array(vec![]), -1, "VMArray", 0),
        (StackValue::Array(vec![StackValue::Null]), 1, "VMArray", 1),
        (StackValue::Struct(vec![]), 0, "VMArray", 0),
        (StackValue::Buffer(vec![0]), 1, "Buffer", 1),
        (StackValue::ByteString(vec![0]), -1, "PrimitiveType", 1),
        (StackValue::Boolean(false), 1, "PrimitiveType", 1),
        (StackValue::Integer(0), 0, "PrimitiveType", 0),
        (StackValue::BigInteger(vec![0]), 0, "PrimitiveType", 0),
    ];
    for (collection, index, kind, count) in cases {
        expect_caught(
            OpCode::PICKITEM,
            vec![collection, StackValue::Integer(index)],
            &format!("The index of {kind} is out of range, {index}/[0, {count})."),
        );
    }
}

#[test]
fn missing_map_keys_use_native_primitive_display_in_the_catch_payload() {
    let cases = [
        (StackValue::Integer(-128), "-128"),
        (
            StackValue::BigInteger(vec![0, 0, 0, 0, 0, 0, 0, 0, 1]),
            "18446744073709551616",
        ),
        (StackValue::Boolean(true), "True"),
        (StackValue::Boolean(false), "False"),
        (StackValue::ByteString(b"missing".to_vec()), "\"missing\""),
        (StackValue::ByteString(b"\"\n\0".to_vec()), "\"\"\n\0\""),
        (StackValue::ByteString(vec![0xff]), "\"Base64: /w==\""),
        (StackValue::ByteString(vec![0xff, 0x80]), "\"Base64: /4A=\""),
        (
            StackValue::ByteString(vec![0xff, 0x80, 0]),
            "\"Base64: /4AA\"",
        ),
    ];
    for (key, display) in cases {
        expect_caught(
            OpCode::PICKITEM,
            vec![StackValue::Map(vec![]), key],
            &format!("Key {display} not found in Map."),
        );
    }
}

#[test]
fn collection_key_type_int32_overflow_and_map_key_size_are_fatal() {
    for key in [
        StackValue::Null,
        StackValue::Array(vec![]),
        StackValue::Buffer(vec![0]),
        StackValue::Interop(0),
        StackValue::Pointer(0),
        StackValue::Integer(i64::MAX),
        StackValue::Integer(i64::from(i32::MIN) - 1),
        StackValue::ByteString(vec![0; 33]),
    ] {
        assert!(
            matches!(
                typed(
                    OpCode::PICKITEM,
                    vec![StackValue::Array(vec![]), key.clone()]
                ),
                Err(VmException::Fault(_))
            ),
            "{key:?}"
        );
        assert!(
            matches!(
                typed(
                    OpCode::SETITEM,
                    vec![StackValue::Array(vec![]), key.clone(), StackValue::Null]
                ),
                Err(VmException::Fault(_))
            ),
            "{key:?}"
        );
    }
    for opcode in [OpCode::PICKITEM, OpCode::SETITEM] {
        let mut stack = vec![StackValue::Map(vec![]), StackValue::ByteString(vec![0; 65])];
        if opcode == OpCode::SETITEM {
            stack.push(StackValue::Null);
        }
        assert!(matches!(typed(opcode, stack), Err(VmException::Fault(_))));
        assert!(matches!(typed(opcode, vec![]), Err(VmException::Fault(_))));
    }
    assert!(matches!(
        typed(
            OpCode::PICKITEM,
            vec![StackValue::Null, StackValue::Integer(0)]
        ),
        Err(VmException::Fault(_))
    ));
}

#[test]
fn collection_indices_use_signed_primitive_int32_conversion() {
    for key in [
        StackValue::Boolean(false),
        StackValue::ByteString(vec![]),
        StackValue::ByteString(vec![0; 32]),
        StackValue::BigInteger(vec![0]),
    ] {
        let result = typed(
            OpCode::PICKITEM,
            vec![StackValue::Array(vec![StackValue::Boolean(true)]), key],
        )
        .unwrap();
        assert_eq!(result.stack, vec![StackValue::Boolean(true)]);
    }
    expect_caught(
        OpCode::PICKITEM,
        vec![
            StackValue::Array(vec![]),
            StackValue::ByteString(vec![0xff]),
        ],
        "The index of VMArray is out of range, -1/[0, 0).",
    );
}

#[test]
fn setitem_bounds_are_catchable_before_buffer_value_conversion() {
    for (collection, kind) in [
        (StackValue::Array(vec![]), "VMArray"),
        (StackValue::Struct(vec![]), "VMArray"),
        (StackValue::Buffer(vec![]), "Buffer"),
    ] {
        expect_caught(
            OpCode::SETITEM,
            vec![
                collection,
                StackValue::Integer(-1),
                StackValue::Array(vec![]),
            ],
            &format!("The index of {kind} is out of range, -1/[0, 0)."),
        );
    }
}

#[test]
fn setitem_buffer_nonprimitive_values_and_byte_overflow_are_fatal() {
    for value in [
        StackValue::Null,
        StackValue::Array(vec![]),
        StackValue::Buffer(vec![1]),
        StackValue::Integer(-129),
        StackValue::Integer(256),
        StackValue::Integer(i64::MAX),
        StackValue::ByteString(vec![0; 33]),
    ] {
        assert!(
            matches!(
                typed(
                    OpCode::SETITEM,
                    vec![
                        StackValue::Buffer(vec![0]),
                        StackValue::Integer(0),
                        value.clone()
                    ]
                ),
                Err(VmException::Fault(_))
            ),
            "{value:?}"
        );
    }
    assert!(matches!(
        typed(
            OpCode::SETITEM,
            vec![
                StackValue::ByteString(vec![0]),
                StackValue::Integer(0),
                StackValue::Integer(1)
            ]
        ),
        Err(VmException::Fault(_))
    ));
}

#[test]
fn setitem_buffer_accepts_native_boolean_and_signed_byte_values() {
    for value in [
        StackValue::Boolean(true),
        StackValue::Integer(-128),
        StackValue::Integer(255),
        StackValue::ByteString(vec![0xff, 0]),
        StackValue::ByteString(vec![]),
        StackValue::BigInteger(vec![0xff, 0]),
    ] {
        let result = typed(
            OpCode::SETITEM,
            vec![StackValue::Buffer(vec![0]), StackValue::Integer(0), value],
        )
        .unwrap();
        assert!(result.stack.is_empty());
    }
    let script = [
        OpCode::DUP.byte(),
        OpCode::PUSH0.byte(),
        OpCode::PUSHT.byte(),
        OpCode::SETITEM.byte(),
        OpCode::RET.byte(),
    ];
    let result = interpret_with_stack_and_syscalls_at_typed(
        &script,
        vec![StackValue::Buffer(vec![0])],
        0,
        &mut NoCalls,
    )
    .unwrap();
    assert_eq!(result.stack, vec![StackValue::Buffer(vec![1])]);
}

#[test]
fn map_integer_representations_share_identity_but_booleans_do_not() {
    let entries = vec![
        (StackValue::Integer(7), StackValue::Integer(1)),
        (StackValue::Boolean(true), StackValue::Integer(2)),
    ];
    let result = typed(
        OpCode::PICKITEM,
        vec![StackValue::Map(entries), StackValue::BigInteger(vec![7])],
    )
    .unwrap();
    assert_eq!(result.stack, vec![StackValue::Integer(1)]);
    expect_caught(
        OpCode::PICKITEM,
        vec![
            StackValue::Map(vec![(StackValue::Integer(1), StackValue::Null)]),
            StackValue::Boolean(true),
        ],
        "Key True not found in Map.",
    );
}

#[test]
fn uncaught_collection_failure_retains_the_native_vm_exception_value() {
    let result = interpret_with_stack_and_syscalls_at_typed(
        &[OpCode::PICKITEM.byte()],
        vec![StackValue::Array(vec![]), StackValue::Integer(0)],
        0,
        &mut NoCalls,
    );
    assert_eq!(
        result.unwrap_err(),
        VmException::Throw(StackValue::ByteString(
            b"The index of VMArray is out of range, 0/[0, 0).".to_vec()
        ))
    );
}

#[test]
fn legacy_collection_exception_behavior_is_unchanged() {
    let legacy = interpret_with_stack_and_syscalls_at(
        &catch(OpCode::PICKITEM),
        vec![StackValue::Array(vec![]), StackValue::Integer(0)],
        0,
        &mut NoCalls,
    )
    .unwrap();
    assert_eq!(
        legacy.stack,
        vec![StackValue::ByteString(
            b"index out of range for PICKITEM".to_vec()
        )]
    );
    let legacy = interpret_with_stack_and_syscalls_at(
        &catch(OpCode::PICKITEM),
        vec![
            StackValue::Array(vec![StackValue::Integer(7)]),
            StackValue::Null,
        ],
        0,
        &mut NoCalls,
    )
    .unwrap();
    assert_eq!(legacy.stack, vec![StackValue::Integer(7)]);
    let legacy = interpret_with_stack_and_syscalls_at(
        &catch(OpCode::SETITEM),
        vec![
            StackValue::Array(vec![]),
            StackValue::Integer(0),
            StackValue::Null,
        ],
        0,
        &mut NoCalls,
    );
    assert_eq!(legacy.unwrap_err(), "index out of range for SETITEM");
}
