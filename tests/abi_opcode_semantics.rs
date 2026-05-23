use neo_vm_rs::semantics::{arithmetic, collections, comparison, conversion};
use neo_vm_rs::{
    stack_value_span_bytes, StackValue, NEOVM_STACK_ITEM_TYPE_ARRAY,
    NEOVM_STACK_ITEM_TYPE_BYTESTRING, NEOVM_STACK_ITEM_TYPE_INTEGER, NEOVM_STACK_ITEM_TYPE_STRUCT,
};

#[test]
fn arithmetic_semantics_cover_stack_value_opcode_rules() {
    assert_eq!(
        arithmetic::add_values(StackValue::Integer(10), StackValue::Integer(3)),
        Ok(StackValue::Integer(13))
    );
    assert_eq!(
        arithmetic::sub_values(StackValue::Integer(10), StackValue::Integer(3)),
        Ok(StackValue::Integer(7))
    );
    assert_eq!(
        arithmetic::mul_values(StackValue::Integer(6), StackValue::Integer(7)),
        Ok(StackValue::Integer(42))
    );
    assert_eq!(
        arithmetic::div_values(StackValue::Integer(10), StackValue::Integer(3)),
        Ok(StackValue::Integer(3))
    );
    assert_eq!(
        arithmetic::modulo_values(StackValue::Integer(10), StackValue::Integer(3)),
        Ok(StackValue::Integer(1))
    );
    assert_eq!(
        arithmetic::negate_value(StackValue::Integer(5)),
        Ok(StackValue::Integer(-5))
    );
    assert_eq!(
        arithmetic::abs_value(StackValue::Integer(-7)),
        Ok(StackValue::Integer(7))
    );
    assert_eq!(
        arithmetic::sign_value(StackValue::Integer(-3)),
        Ok(StackValue::Integer(-1))
    );
    assert_eq!(
        arithmetic::max_values(StackValue::Integer(3), StackValue::Integer(7)),
        Ok(StackValue::Integer(7))
    );
    assert_eq!(
        arithmetic::min_values(StackValue::Integer(3), StackValue::Integer(7)),
        Ok(StackValue::Integer(3))
    );
    assert_eq!(
        arithmetic::pow_values(StackValue::Integer(2), StackValue::Integer(10)),
        Ok(StackValue::Integer(1024))
    );
    assert_eq!(
        arithmetic::sqrt_value(StackValue::Integer(49)),
        Ok(StackValue::Integer(7))
    );
    assert_eq!(
        arithmetic::modmul_values(
            StackValue::Integer(7),
            StackValue::Integer(8),
            StackValue::Integer(10)
        ),
        Ok(StackValue::Integer(6))
    );
    assert_eq!(
        arithmetic::modpow_values(
            StackValue::Integer(2),
            StackValue::Integer(10),
            StackValue::Integer(100)
        ),
        Ok(StackValue::Integer(24))
    );
    assert_eq!(
        arithmetic::shl_value(StackValue::Integer(1), 4),
        Ok(StackValue::Integer(16))
    );
    assert_eq!(
        arithmetic::shr_value(StackValue::Integer(16), 2),
        Ok(StackValue::Integer(4))
    );
    assert_eq!(
        arithmetic::bitwise_and_values(StackValue::Integer(0b1100), StackValue::Integer(0b1010)),
        Ok(StackValue::Integer(0b1000))
    );
    assert_eq!(
        arithmetic::bitwise_or_values(StackValue::Integer(0b1100), StackValue::Integer(0b1010)),
        Ok(StackValue::Integer(0b1110))
    );
    assert_eq!(
        arithmetic::bitwise_xor_values(StackValue::Integer(0b1100), StackValue::Integer(0b1010)),
        Ok(StackValue::Integer(0b0110))
    );
    assert_eq!(
        arithmetic::invert_value(StackValue::Integer(0)),
        Ok(StackValue::Integer(-1))
    );
    assert_eq!(
        arithmetic::inc_value(StackValue::Integer(41)),
        Ok(StackValue::Integer(42))
    );
    assert_eq!(
        arithmetic::dec_value(StackValue::Integer(43)),
        Ok(StackValue::Integer(42))
    );
    assert_eq!(
        arithmetic::within_values(
            StackValue::Integer(5),
            StackValue::Integer(3),
            StackValue::Integer(7)
        ),
        Ok(true)
    );
    assert_eq!(
        arithmetic::within_values(
            StackValue::Integer(7),
            StackValue::Integer(3),
            StackValue::Integer(7)
        ),
        Ok(false)
    );
}

#[test]
fn arithmetic_semantics_return_opcode_fault_messages() {
    assert_eq!(
        arithmetic::div_values(StackValue::Integer(10), StackValue::Integer(0)),
        Err("division by zero for DIV".to_string())
    );
    assert_eq!(
        arithmetic::modulo_values(StackValue::Integer(10), StackValue::Integer(0)),
        Err("division by zero for MOD".to_string())
    );
    assert_eq!(
        arithmetic::pow_values(StackValue::Integer(2), StackValue::Integer(-1)),
        Err("negative exponent for POW".to_string())
    );
    assert_eq!(
        arithmetic::sqrt_value(StackValue::Integer(-1)),
        Err("negative value for SQRT".to_string())
    );
    assert_eq!(
        arithmetic::modmul_values(
            StackValue::Integer(1),
            StackValue::Integer(2),
            StackValue::Integer(0)
        ),
        Err("division by zero for MODMUL".to_string())
    );
    assert_eq!(
        arithmetic::modpow_values(
            StackValue::Integer(2),
            StackValue::Integer(1),
            StackValue::Integer(0)
        ),
        Err("division by zero for MODPOW".to_string())
    );
    assert_eq!(
        arithmetic::shl_value(StackValue::Integer(1), -1),
        Err("shift count out of range for SHL".to_string())
    );
    assert_eq!(
        arithmetic::shr_value(StackValue::Integer(1), 257),
        Err("shift count out of range for SHR".to_string())
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
    assert_eq!(
        comparison::less_than_values(&StackValue::Integer(3), &StackValue::Integer(5)),
        Ok(true)
    );
    assert_eq!(
        comparison::less_or_equal_values(&StackValue::Integer(5), &StackValue::Integer(5)),
        Ok(true)
    );
    assert_eq!(
        comparison::greater_than_values(&StackValue::Integer(5), &StackValue::Integer(3)),
        Ok(true)
    );
    assert_eq!(
        comparison::greater_or_equal_values(&StackValue::Integer(5), &StackValue::Integer(5)),
        Ok(true)
    );
    assert_eq!(
        comparison::num_equal_values(&StackValue::Integer(10), &StackValue::Integer(10)),
        Ok(true)
    );
    assert_eq!(
        comparison::num_not_equal_values(&StackValue::Integer(10), &StackValue::Integer(11)),
        Ok(true)
    );
    assert!(comparison::bool_and(true, true));
    assert!(comparison::bool_or(false, true));
    assert_eq!(comparison::not_value(&StackValue::Boolean(true)), Ok(false));
    assert_eq!(comparison::nz_value(&StackValue::Integer(42)), Ok(true));
    assert!(comparison::is_null(&StackValue::Null));

    assert!(conversion::is_type(
        &StackValue::Integer(42),
        NEOVM_STACK_ITEM_TYPE_INTEGER
    ));
    assert!(conversion::is_type(
        &StackValue::BigInteger(vec![0xff, 0x00]),
        NEOVM_STACK_ITEM_TYPE_INTEGER
    ));
    assert!(!conversion::is_type(
        &StackValue::BigInteger(vec![0xff, 0x00]),
        NEOVM_STACK_ITEM_TYPE_BYTESTRING
    ));
    assert_eq!(
        conversion::convert_value(StackValue::Integer(256), NEOVM_STACK_ITEM_TYPE_BYTESTRING),
        Ok(StackValue::ByteString(vec![0, 1]))
    );
    assert_eq!(
        conversion::convert_value(
            StackValue::Array(vec![StackValue::Integer(1)]),
            NEOVM_STACK_ITEM_TYPE_STRUCT
        ),
        Ok(StackValue::Struct(vec![StackValue::Integer(1)]))
    );
    assert_eq!(
        conversion::convert_value(StackValue::Null, NEOVM_STACK_ITEM_TYPE_ARRAY),
        Ok(StackValue::Null)
    );
    assert_eq!(
        conversion::convert_value(
            StackValue::Map(Vec::new()),
            NEOVM_STACK_ITEM_TYPE_BYTESTRING
        ),
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
