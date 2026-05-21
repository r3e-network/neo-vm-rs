use neo_vm_rs::OpCode;

fn canonical_opcode_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(0x00..=0x05);
    bytes.extend(0x08..=0x41);
    bytes.extend([0x43, 0x45, 0x46]);
    bytes.extend(0x48..=0x4b);
    bytes.extend([0x4d, 0x4e]);
    bytes.extend(0x50..=0x55);
    bytes.extend(0x56..=0x89);
    bytes.extend(0x8b..=0x8e);
    bytes.extend(0x90..=0x93);
    bytes.extend(0x97..=0xa6);
    bytes.extend(0xa8..=0xac);
    bytes.push(0xb1);
    bytes.extend(0xb3..=0xbb);
    bytes.extend(0xbe..=0xc6);
    bytes.push(0xc8);
    bytes.extend(0xca..=0xd4);
    bytes.extend([0xd8, 0xd9, 0xdb]);
    bytes.extend(0xe0..=0xe1);
    bytes
}

#[test]
fn canonical_opcode_byte_table_is_complete_and_rejects_gaps() {
    let canonical = canonical_opcode_bytes();
    assert_eq!(canonical.len(), OpCode::ALL.len());

    let mut accepted = [false; 256];
    for byte in canonical {
        assert!(
            !accepted[byte as usize],
            "duplicate opcode byte {byte:#04x}"
        );
        accepted[byte as usize] = true;

        let opcode = OpCode::try_from(byte).expect("canonical opcode should decode");
        assert_eq!(opcode.byte(), byte);
        assert_eq!(OpCode::from_u8(byte), Some(opcode));
    }

    for byte in 0u8..=u8::MAX {
        assert_eq!(
            OpCode::try_from(byte).is_ok(),
            accepted[byte as usize],
            "unexpected decode result for byte {byte:#04x}"
        );
    }
}

#[test]
fn legacy_and_non_canonical_opcode_gaps_are_rejected() {
    for byte in [0x06, 0x07, 0xda, 0xf1] {
        assert!(
            OpCode::try_from(byte).is_err(),
            "byte {byte:#04x} is not a NeoVM 3.9.x opcode"
        );
        assert_eq!(OpCode::from_u8(byte), None);
    }
}

#[test]
fn all_opcode_metadata_is_total_and_consistent() {
    for opcode in OpCode::ALL {
        assert_eq!(OpCode::from_u8(opcode.byte()), Some(opcode));
        assert!(!opcode.name().is_empty());
        assert_eq!(opcode.to_string(), opcode.name());
    }
}

#[test]
fn variable_length_opcode_prefixes_are_explicit() {
    assert_eq!(OpCode::PUSHDATA1.operand_prefix(), 1);
    assert_eq!(OpCode::PUSHDATA2.operand_prefix(), 2);
    assert_eq!(OpCode::PUSHDATA4.operand_prefix(), 4);

    for opcode in OpCode::ALL {
        if !matches!(
            opcode,
            OpCode::PUSHDATA1 | OpCode::PUSHDATA2 | OpCode::PUSHDATA4
        ) {
            assert_eq!(
                opcode.operand_prefix(),
                0,
                "{opcode} should not have a variable prefix"
            );
        }
    }
}
