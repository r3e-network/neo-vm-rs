use neo_vm_rs::{callback_codec, result_codec, ExecutionResult, StackValue, VmState};

#[test]
fn callback_stack_result_codec_is_shared_at_vm_boundary() {
    let original = Ok(vec![
        StackValue::Integer(42),
        StackValue::ByteString(b"neo".to_vec()),
        StackValue::Buffer(vec![1, 2, 3]),
    ]);

    let encoded = callback_codec::encode_stack_result(&original);
    let decoded = callback_codec::decode_stack_result(&encoded).expect("decode should succeed");

    assert_eq!(decoded, original);
}

#[test]
fn execution_result_codec_is_shared_at_vm_boundary() {
    let original = Ok(ExecutionResult {
        fee_consumed_pico: 123,
        state: VmState::Fault,
        stack: vec![StackValue::Boolean(true)],
        fault_message: Some("boom".to_string()),
        fault_ip: Some(7),
        fault_locals: Some(vec![4, 5, 6]),
    });

    let encoded = result_codec::encode_execution_result(&original);
    let decoded = result_codec::decode_execution_result(&encoded).expect("decode should succeed");

    assert_eq!(decoded, original);
}
