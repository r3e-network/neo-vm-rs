use neo_vm_rs::semantics::{arithmetic, collections, comparison, conversion};
use neo_vm_rs::{stack_value_span_bytes, StackValue};

#[test]
fn arithmetic_semantics_cover_riscv_runtime_integer_ops() {
    assert_eq!(arithmetic::add_i64(10, 3), 13);
    assert_eq!(arithmetic::sub_i64(10, 3), 7);
    assert_eq!(arithmetic::mul_i64(6, 7), 42);
    assert_eq!(arithmetic::div_i64(10, 3), Ok(3));
    assert_eq!(arithmetic::modulo_i64(10, 3), Ok(1));
    assert_eq!(arithmetic::negate_i64(5), -5);
    assert_eq!(arithmetic::abs_i64(-7), 7);
    assert_eq!(arithmetic::sign_i64(-3), -1);
    assert_eq!(arithmetic::max_i64(3, 7), 7);
    assert_eq!(arithmetic::min_i64(3, 7), 3);
    assert_eq!(arithmetic::pow_i64(2, 10), Ok(1024));
    assert_eq!(arithmetic::sqrt_i64(49), Ok(7));
    assert_eq!(arithmetic::modmul_i64(7, 8, 10), Ok(6));
    assert_eq!(arithmetic::modpow_i64(2, 10, 100), Ok(24));
    assert_eq!(arithmetic::shl_i64(1, 4), Ok(16));
    assert_eq!(arithmetic::shr_i64(16, 2), Ok(4));
    assert_eq!(arithmetic::bitwise_and_i64(0b1100, 0b1010), 0b1000);
    assert_eq!(arithmetic::bitwise_or_i64(0b1100, 0b1010), 0b1110);
    assert_eq!(arithmetic::bitwise_xor_i64(0b1100, 0b1010), 0b0110);
    assert_eq!(arithmetic::bitwise_not_i64(0), -1);
    assert_eq!(arithmetic::inc_i64(41), 42);
    assert_eq!(arithmetic::dec_i64(43), 42);
    assert!(arithmetic::within_i64(5, 3, 7));
    assert!(!arithmetic::within_i64(7, 3, 7));
}

#[test]
fn arithmetic_semantics_return_opcode_fault_messages() {
    assert_eq!(arithmetic::div_i64(10, 0), Err("DIV: division by zero"));
    assert_eq!(arithmetic::modulo_i64(10, 0), Err("MOD: division by zero"));
    assert_eq!(arithmetic::pow_i64(2, -1), Err("POW: negative exponent"));
    assert_eq!(
        arithmetic::pow_i64(2, 64),
        Err("POW: exponent too large for i64 fast path")
    );
    assert_eq!(arithmetic::sqrt_i64(-1), Err("SQRT: negative value"));
    assert_eq!(
        arithmetic::modmul_i64(1, 2, 0),
        Err("MODMUL: division by zero")
    );
    assert_eq!(
        arithmetic::modpow_i64(2, -1, 7),
        Err("MODPOW: negative exponent")
    );
    assert_eq!(
        arithmetic::modpow_i64(2, 1, 0),
        Err("MODPOW: division by zero")
    );
    assert_eq!(
        arithmetic::shl_i64(1, -1),
        Err("SHL: shift amount out of range")
    );
    assert_eq!(
        arithmetic::shr_i64(1, 64),
        Err("SHR: shift amount out of range")
    );
}

#[test]
fn comparison_and_conversion_semantics_use_shared_stack_value_rules() {
    assert!(comparison::equal_values(
        &StackValue::ByteString(b"neo".to_vec()),
        &StackValue::ByteString(b"neo".to_vec())
    ));
    assert!(comparison::not_equal_values(
        &StackValue::Integer(1),
        &StackValue::Integer(2)
    ));
    assert!(comparison::less_than_i64(3, 5));
    assert!(comparison::less_or_equal_i64(5, 5));
    assert!(comparison::greater_than_i64(5, 3));
    assert!(comparison::greater_or_equal_i64(5, 5));
    assert!(comparison::num_equal_i64(10, 10));
    assert!(comparison::num_not_equal_i64(10, 11));
    assert!(comparison::bool_and(true, true));
    assert!(comparison::bool_or(false, true));
    assert!(!comparison::bool_not(true));
    assert!(comparison::nz(&StackValue::Integer(42)));
    assert!(comparison::is_null(&StackValue::Null));

    assert!(conversion::is_type(&StackValue::Integer(42), 0x21));
    assert_eq!(
        conversion::convert_value(StackValue::Integer(256), 0x28),
        Ok(StackValue::ByteString(vec![0, 1]))
    );
    assert_eq!(
        conversion::convert_value(StackValue::Array(vec![StackValue::Integer(1)]), 0x41),
        Ok(StackValue::Struct(vec![StackValue::Integer(1)]))
    );
    assert_eq!(
        conversion::convert_value(StackValue::Null, 0x30),
        Ok(StackValue::Null)
    );
    assert_eq!(
        conversion::convert_value(StackValue::Map(Vec::new()), 0x28),
        Err("CONVERT: cannot convert to ByteString".to_string())
    );
}

#[test]
fn stack_value_span_bytes_are_shared_splice_inputs() {
    assert_eq!(
        stack_value_span_bytes(&StackValue::Integer(128)),
        Some(vec![0x80, 0x00])
    );
    assert_eq!(
        stack_value_span_bytes(&StackValue::Boolean(true)),
        Some(vec![0x01])
    );
    assert_eq!(
        stack_value_span_bytes(&StackValue::ByteString(b"neo".to_vec())),
        Some(b"neo".to_vec())
    );
    assert_eq!(
        stack_value_span_bytes(&StackValue::Buffer(b"n4".to_vec())),
        Some(b"n4".to_vec())
    );
    assert_eq!(stack_value_span_bytes(&StackValue::Null), None);
}

#[test]
fn collection_semantics_cover_value_construction_and_queries() {
    assert_eq!(
        collections::new_array(3),
        Ok(StackValue::Array(vec![StackValue::Null; 3]))
    );
    assert_eq!(
        collections::new_array_t(2, 0x28),
        Ok(StackValue::Array(vec![
            StackValue::ByteString(Vec::new()),
            StackValue::ByteString(Vec::new()),
        ]))
    );
    assert_eq!(
        collections::new_struct(2),
        Ok(StackValue::Struct(vec![StackValue::Null; 2]))
    );
    assert_eq!(
        collections::new_buffer(4),
        Ok(StackValue::Buffer(vec![0; 4]))
    );

    let mut array = StackValue::Array(Vec::new());
    collections::append(&mut array, StackValue::Integer(42)).unwrap();
    collections::append(&mut array, StackValue::Integer(99)).unwrap();
    assert_eq!(collections::size(&array), Ok(2));
    assert_eq!(
        collections::pick_item(&array, &StackValue::Integer(1)),
        Ok(StackValue::Integer(99))
    );
    assert_eq!(
        collections::has_key(&array, &StackValue::Integer(2)),
        Ok(false)
    );

    let mut map = StackValue::Map(Vec::new());
    collections::set_item(
        &mut map,
        StackValue::ByteString(b"key".to_vec()),
        StackValue::Integer(7),
    )
    .unwrap();
    assert_eq!(
        collections::pick_item(&map, &StackValue::ByteString(b"key".to_vec())),
        Ok(StackValue::Integer(7))
    );
    assert_eq!(
        collections::keys(map.clone()),
        Ok(StackValue::Array(vec![StackValue::ByteString(
            b"key".to_vec()
        )]))
    );
    assert_eq!(
        collections::values(map),
        Ok(StackValue::Array(vec![StackValue::Integer(7)]))
    );
}

#[test]
fn collection_semantics_cover_mutation_and_stack_shaping() {
    let packed = collections::pack(vec![
        StackValue::Integer(10),
        StackValue::Integer(20),
        StackValue::Integer(30),
    ]);
    assert_eq!(
        packed,
        StackValue::Array(vec![
            StackValue::Integer(10),
            StackValue::Integer(20),
            StackValue::Integer(30),
        ])
    );
    assert_eq!(
        collections::unpack(packed),
        Ok(vec![
            StackValue::Integer(10),
            StackValue::Integer(20),
            StackValue::Integer(30),
            StackValue::Integer(3),
        ])
    );

    let mut buffer = StackValue::Buffer(vec![1, 2, 3]);
    collections::set_item(
        &mut buffer,
        StackValue::Integer(1),
        StackValue::Integer(255),
    )
    .unwrap();
    assert_eq!(buffer, StackValue::Buffer(vec![1, 255, 3]));
    assert_eq!(
        collections::pop_item(buffer),
        Ok(vec![StackValue::Integer(3)])
    );

    let mut array = StackValue::Array(vec![
        StackValue::Integer(1),
        StackValue::Integer(2),
        StackValue::Integer(3),
    ]);
    collections::reverse_items(&mut array).unwrap();
    assert_eq!(
        array,
        StackValue::Array(vec![
            StackValue::Integer(3),
            StackValue::Integer(2),
            StackValue::Integer(1),
        ])
    );
    collections::clear_items(&mut array).unwrap();
    assert_eq!(array, StackValue::Array(Vec::new()));
}
