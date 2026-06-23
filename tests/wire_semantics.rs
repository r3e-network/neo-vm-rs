use neo_vm_rs::{ExecutionResult, StackValue, VmState};

#[test]
fn stack_values_round_trip_through_serde() {
    let value = StackValue::Array(
        0,
        vec![
            StackValue::Integer(42),
            StackValue::ByteString(vec![0xde, 0xad, 0xbe, 0xef]),
            StackValue::Map(
                0,
                vec![(StackValue::ByteString(vec![1]), StackValue::Boolean(true))],
            ),
        ],
    );

    let json = serde_json::to_string(&value).expect("stack value should serialize");
    let decoded: StackValue = serde_json::from_str(&json).expect("stack value should deserialize");
    assert!(
        decoded.structural_eq(&value),
        "decoded={decoded:?} expected={value:?}"
    );
}

#[test]
fn execution_result_accepts_backward_compatible_minimal_json() {
    let json = r#"{"fee_consumed_pico":7,"state":"Halt","stack":[{"Integer":42}]}"#;
    let decoded: ExecutionResult =
        serde_json::from_str(json).expect("minimal execution result should decode");

    assert_eq!(decoded.fee_consumed_pico, 7);
    assert_eq!(decoded.state, VmState::Halt);
    assert_eq!(decoded.stack, vec![StackValue::Integer(42)]);
    assert_eq!(decoded.fault_message, None);
    assert_eq!(decoded.fault_ip, None);
    assert_eq!(decoded.fault_locals, None);
}
