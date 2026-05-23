use neo_vm_rs::{
    interpret, interpret_with_stack_and_syscalls_at, ExecutionResult, OpCode, StackValue, VmState,
};

#[path = "interpreter_smoke/error_host.rs"]
mod error_host;
#[path = "interpreter_smoke/host.rs"]
mod host;

use error_host::ErrorHost;
use host::Host;

#[test]
fn executes_basic_arithmetic_script() {
    let result = interpret(&[0x12, 0x13, 0x9e, 0x40]).expect("script should execute");

    assert_eq!(result.state, VmState::Halt);
    assert_eq!(result.stack, vec![StackValue::Integer(5)]);
}

#[test]
fn executes_historical_size_and_pickitem_primitive_cases() {
    let size_int = interpret(&[
        OpCode::PUSHINT16.byte(),
        0x80,
        0x00,
        OpCode::SIZE.byte(),
        OpCode::RET.byte(),
    ])
    .expect("SIZE should accept Integer");
    assert_eq!(size_int.stack, vec![StackValue::Integer(2)]);

    let size_bool = interpret(&[
        OpCode::PUSHT.byte(),
        OpCode::SIZE.byte(),
        OpCode::RET.byte(),
    ])
    .expect("SIZE should accept Boolean");
    assert_eq!(size_bool.stack, vec![StackValue::Integer(1)]);

    let pick_int = interpret(&[
        OpCode::PUSHINT16.byte(),
        0x80,
        0x00,
        OpCode::PUSH0.byte(),
        OpCode::PICKITEM.byte(),
        OpCode::RET.byte(),
    ])
    .expect("PICKITEM should index Integer memory");
    assert_eq!(pick_int.stack, vec![StackValue::Integer(128)]);

    let pick_false = interpret(&[
        OpCode::PUSHF.byte(),
        OpCode::PUSH0.byte(),
        OpCode::PICKITEM.byte(),
        OpCode::RET.byte(),
    ])
    .expect("PICKITEM should index Boolean(false) memory");
    assert_eq!(pick_false.stack, vec![StackValue::Integer(0)]);
}

#[test]
fn equal_keeps_primitive_types_strict() {
    let equal = interpret(&[
        OpCode::PUSH1.byte(),
        OpCode::PUSHDATA1.byte(),
        0x01,
        0x01,
        OpCode::EQUAL.byte(),
        OpCode::RET.byte(),
    ])
    .expect("EQUAL should execute");
    assert_eq!(equal.stack, vec![StackValue::Boolean(false)]);

    let not_equal = interpret(&[
        OpCode::PUSH1.byte(),
        OpCode::PUSHDATA1.byte(),
        0x01,
        0x01,
        OpCode::NOTEQUAL.byte(),
        OpCode::RET.byte(),
    ])
    .expect("NOTEQUAL should execute");
    assert_eq!(not_equal.stack, vec![StackValue::Boolean(true)]);
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
