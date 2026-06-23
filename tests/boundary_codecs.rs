use neo_vm_rs::{ExecutionResult, StackValue, VmState, callback_codec, result_codec};

#[test]
fn callback_stack_result_codec_is_shared_at_vm_boundary() {
    let original = Ok(vec![
        StackValue::Integer(42),
        StackValue::ByteString(b"neo".to_vec()),
        StackValue::Buffer(0, vec![1, 2, 3]),
    ]);

    let encoded = callback_codec::encode_stack_result(&original);
    let decoded = callback_codec::decode_stack_result(&encoded)
        .expect("decode should succeed")
        .expect("inner decode should succeed");
    let orig = original.unwrap();

    assert_eq!(decoded.len(), orig.len());
    for (d, o) in decoded.iter().zip(orig.iter()) {
        assert!(d.structural_eq(o), "mismatch: decoded={d:?} expected={o:?}");
    }
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

#[test]
fn execution_result_codec_rejects_non_final_vm_state() {
    let original = Ok(ExecutionResult {
        fee_consumed_pico: 0,
        state: VmState::Break,
        stack: Vec::new(),
        fault_message: None,
        fault_ip: None,
        fault_locals: None,
    });

    let encoded = result_codec::encode_execution_result(&original);
    let decoded = result_codec::decode_execution_result(&encoded).expect("decode should succeed");

    assert_eq!(
        decoded,
        Err("Break is not a final execution result state".to_string())
    );
}
