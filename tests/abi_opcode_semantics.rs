use neo_vm_rs::semantics::{arithmetic, collections, comparison, conversion};
use neo_vm_rs::{
    NEOVM_STACK_ITEM_TYPE_ARRAY, NEOVM_STACK_ITEM_TYPE_BYTESTRING, NEOVM_STACK_ITEM_TYPE_INTEGER,
    NEOVM_STACK_ITEM_TYPE_STRUCT, StackValue, stack_value_span_bytes,
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
    assert!(
        conversion::convert_value(
            StackValue::Array(0, vec![StackValue::Integer(1)]),
            NEOVM_STACK_ITEM_TYPE_STRUCT
        )
        .as_ref()
        .map(|v| v.structural_eq(&StackValue::Struct(0, vec![StackValue::Integer(1)])))
        .unwrap_or(false),
        "convert_value Array->Struct mismatch"
    );
    assert_eq!(
        conversion::convert_value(StackValue::Null, NEOVM_STACK_ITEM_TYPE_ARRAY),
        Ok(StackValue::Null)
    );
    assert_eq!(
        conversion::convert_value(
            StackValue::Map(0, Vec::new()),
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
        stack_value_span_bytes(&StackValue::Buffer(0, b"n4".to_vec())),
        Some(b"n4".to_vec())
    );
    assert_eq!(stack_value_span_bytes(&StackValue::Null), None);
}

#[test]
fn collection_semantics_cover_value_construction_and_queries() {
    assert!(
        collections::new_array(3)
            .as_ref()
            .map(|v| v.structural_eq(&StackValue::Array(0, vec![StackValue::Null; 3])))
            .unwrap_or(false),
        "new_array mismatch"
    );
    assert!(
        collections::new_array_t(2, 0x28)
            .as_ref()
            .map(|v| v.structural_eq(&StackValue::Array(
                0,
                vec![
                    StackValue::ByteString(Vec::new()),
                    StackValue::ByteString(Vec::new()),
                ]
            )))
            .unwrap_or(false),
        "new_array_t mismatch"
    );
    assert!(
        collections::new_struct(2)
            .as_ref()
            .map(|v| v.structural_eq(&StackValue::Struct(0, vec![StackValue::Null; 2])))
            .unwrap_or(false),
        "new_struct mismatch"
    );
    assert!(
        collections::new_buffer(4)
            .as_ref()
            .map(|v| v.structural_eq(&StackValue::Buffer(0, vec![0; 4])))
            .unwrap_or(false),
        "new_buffer mismatch"
    );

    let mut array = StackValue::Array(0, Vec::new());
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

    let mut map = StackValue::Map(0, Vec::new());
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
    assert!(
        collections::keys(map.clone())
            .as_ref()
            .map(|v| v.structural_eq(&StackValue::Array(
                0,
                vec![StackValue::ByteString(b"key".to_vec())]
            )))
            .unwrap_or(false),
        "keys mismatch"
    );
    assert!(
        collections::values(map)
            .as_ref()
            .map(|v| v.structural_eq(&StackValue::Array(0, vec![StackValue::Integer(7)])))
            .unwrap_or(false),
        "values mismatch"
    );
}

#[test]
fn collection_semantics_cover_mutation_and_stack_shaping() {
    let packed = collections::pack(vec![
        StackValue::Integer(10),
        StackValue::Integer(20),
        StackValue::Integer(30),
    ]);
    assert!(
        packed.structural_eq(&StackValue::Array(
            0,
            vec![
                StackValue::Integer(10),
                StackValue::Integer(20),
                StackValue::Integer(30),
            ]
        )),
        "pack mismatch"
    );
    {
        let unpacked = collections::unpack(packed).expect("unpack failed");
        let expected = vec![
            StackValue::Integer(10),
            StackValue::Integer(20),
            StackValue::Integer(30),
            StackValue::Integer(3),
        ];
        assert_eq!(unpacked.len(), expected.len(), "unpack length mismatch");
        for (a, e) in unpacked.iter().zip(expected.iter()) {
            assert!(
                a.structural_eq(e),
                "unpack element mismatch: {:?} vs {:?}",
                a,
                e
            );
        }
    }

    let mut buffer = StackValue::Buffer(0, vec![1, 2, 3]);
    collections::set_item(
        &mut buffer,
        StackValue::Integer(1),
        StackValue::Integer(255),
    )
    .unwrap();
    assert!(
        buffer.structural_eq(&StackValue::Buffer(0, vec![1, 255, 3])),
        "set_item on buffer mismatch"
    );
    {
        let popped = collections::pop_item(buffer).expect("pop_item failed");
        assert_eq!(popped.len(), 1, "pop_item length mismatch");
        assert!(
            popped[0].structural_eq(&StackValue::Integer(3)),
            "pop_item element mismatch"
        );
    }

    let mut array = StackValue::Array(
        0,
        vec![
            StackValue::Integer(1),
            StackValue::Integer(2),
            StackValue::Integer(3),
        ],
    );
    collections::reverse_items(&mut array).unwrap();
    assert!(
        array.structural_eq(&StackValue::Array(
            0,
            vec![
                StackValue::Integer(3),
                StackValue::Integer(2),
                StackValue::Integer(1),
            ]
        )),
        "reverse_items mismatch"
    );
    collections::clear_items(&mut array).unwrap();
    assert!(
        array.structural_eq(&StackValue::Array(0, Vec::new())),
        "clear_items mismatch"
    );
}
