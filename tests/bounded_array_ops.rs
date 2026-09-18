use neo_vm_rs::{OpCode, StackValue, VmState, interpret};

const MAX_ITEMS: usize = 16;

fn push_integer(script: &mut Vec<u8>, value: i8) {
    script.extend_from_slice(&[OpCode::PUSHINT8.byte(), value as u8]);
}

fn packed_script(values: &[i8]) -> Vec<u8> {
    assert!(values.len() <= MAX_ITEMS);
    let mut script = Vec::new();
    for &value in values {
        push_integer(&mut script, value);
    }
    push_integer(&mut script, values.len() as i8);
    script.push(OpCode::PACK.byte());
    script
}

fn assert_halt_stack(mut script: Vec<u8>, expected: Vec<StackValue>) {
    script.push(OpCode::RET.byte());
    let result = interpret(&script).expect("bounded collection script should execute");
    assert_eq!(result.state, VmState::Halt, "script: {script:02x?}");
    assert_eq!(result.stack, expected, "script: {script:02x?}");
}

fn values_for_length(length: usize) -> Vec<i8> {
    (0..length).map(|index| index as i8 - 8).collect()
}

#[test]
fn pack_preserves_top_first_array_order_for_bounded_lengths() {
    for length in 0..=MAX_ITEMS {
        let values = values_for_length(length);
        let expected = values
            .iter()
            .rev()
            .map(|&value| StackValue::Integer(i64::from(value)))
            .collect();
        assert_halt_stack(packed_script(&values), vec![StackValue::Array(expected)]);
    }
}

#[test]
fn pack_then_unpack_restores_values_and_appends_count() {
    for length in 0..=MAX_ITEMS {
        let values = values_for_length(length);
        let mut script = packed_script(&values);
        script.push(OpCode::UNPACK.byte());
        let mut expected: Vec<_> = values
            .iter()
            .map(|&value| StackValue::Integer(i64::from(value)))
            .collect();
        expected.push(StackValue::Integer(length as i64));
        assert_halt_stack(script, expected);
    }
}

#[test]
fn pickitem_reads_each_packed_index_without_changing_the_array() {
    for length in 1..=MAX_ITEMS {
        let values = values_for_length(length);
        for index in 0..length {
            let mut script = packed_script(&values);
            script.push(OpCode::DUP.byte());
            push_integer(&mut script, index as i8);
            script.push(OpCode::PICKITEM.byte());
            script.push(OpCode::SWAP.byte());
            script.push(OpCode::UNPACK.byte());

            let mut expected = vec![StackValue::Integer(i64::from(values[length - 1 - index]))];
            expected.extend(
                values
                    .iter()
                    .map(|&value| StackValue::Integer(i64::from(value))),
            );
            expected.push(StackValue::Integer(length as i64));
            assert_halt_stack(script, expected);
        }
    }
}
