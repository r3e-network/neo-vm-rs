//! Structural parity with Neo.VM v3.9.0 Script.cs, independent of execution.
//! https://github.com/neo-project/neo-vm/blob/v3.9.0/src/Neo.VM/Script.cs

use neo_vm_rs::{
    Instruction, OpCode, StackItemType, instruction_jump_target, instruction_try_targets,
    validate_script, validate_script_reference, validate_strict_script,
    validate_strict_script_reference,
};

#[test]
fn reference_accepts_actual_core_call_to_final_nop() {
    // Actual C# native-phases.json case call_implicit_return_continues_caller:
    // CALL +4, PUSH9, RET, NOP. The callee at byte 4 falls through EOF.
    let script = [0x34, 0x04, 0x19, 0x40, 0x21];
    let validated = validate_script_reference(&script, true).expect("Neo current-IP CALL target");
    for offset in [0, 2, 3, 4] {
        assert!(validated.has_instruction_at(offset));
    }
    assert!(!validated.has_instruction_at(1));
    assert!(!validated.has_instruction_at(script.len()));
    assert!(validate_strict_script(&script).is_err());
}

#[test]
fn reference_rejects_operand_target_accepted_by_legacy_origin() {
    let script = [
        OpCode::JMP.byte(),
        1,
        OpCode::PUSH0.byte(),
        OpCode::RET.byte(),
    ];
    // Legacy resolves 0 + 2 + 1 to RET; Neo resolves 0 + 1 into JMP's operand.
    assert!(validate_strict_script(&script).is_ok());
    assert!(validate_strict_script_reference(&script).is_err());
}

#[test]
fn reference_checks_every_short_relative_opcode_at_current_ip() {
    for opcode in [
        OpCode::JMP,
        OpCode::JMPIF,
        OpCode::JMPIFNOT,
        OpCode::JMPEQ,
        OpCode::JMPNE,
        OpCode::JMPGT,
        OpCode::JMPGE,
        OpCode::JMPLT,
        OpCode::JMPLE,
        OpCode::CALL,
        OpCode::ENDTRY,
    ] {
        let forward = [opcode.byte(), 2, OpCode::RET.byte()];
        assert!(
            validate_strict_script_reference(&forward).is_ok(),
            "{opcode:?}"
        );
        assert!(validate_strict_script(&forward).is_err(), "{opcode:?}");

        let backward = [OpCode::NOP.byte(), opcode.byte(), 0xff, OpCode::RET.byte()];
        assert!(
            validate_strict_script_reference(&backward).is_ok(),
            "{opcode:?}"
        );
        let before_script = [opcode.byte(), 0xff, OpCode::RET.byte()];
        assert!(
            validate_strict_script_reference(&before_script).is_err(),
            "{opcode:?}"
        );
    }
}

#[test]
fn reference_checks_pusha_and_every_long_relative_opcode_at_current_ip() {
    for opcode in [
        OpCode::PUSHA,
        OpCode::JMP_L,
        OpCode::JMPIF_L,
        OpCode::JMPIFNOT_L,
        OpCode::JMPEQ_L,
        OpCode::JMPNE_L,
        OpCode::JMPGT_L,
        OpCode::JMPGE_L,
        OpCode::JMPLT_L,
        OpCode::JMPLE_L,
        OpCode::CALL_L,
        OpCode::ENDTRY_L,
    ] {
        let mut forward = vec![opcode.byte()];
        forward.extend_from_slice(&5_i32.to_le_bytes());
        forward.push(OpCode::RET.byte());
        assert!(
            validate_strict_script_reference(&forward).is_ok(),
            "{opcode:?}"
        );
        assert!(validate_strict_script(&forward).is_err(), "{opcode:?}");

        let mut backward = vec![OpCode::NOP.byte(), opcode.byte()];
        backward.extend_from_slice(&(-1_i32).to_le_bytes());
        backward.push(OpCode::RET.byte());
        assert!(
            validate_strict_script_reference(&backward).is_ok(),
            "{opcode:?}"
        );
    }
}

#[test]
fn reference_checks_both_short_try_targets_from_current_ip() {
    let valid = [
        OpCode::TRY.byte(),
        3,
        4,
        OpCode::NOP.byte(),
        OpCode::RET.byte(),
    ];
    assert!(validate_strict_script_reference(&valid).is_ok());
    assert!(validate_strict_script(&valid).is_err());

    for (catch_offset, finally_offset) in [(1, 4), (3, 2), (5, 4), (3, 5), (0xff, 4)] {
        let script = [
            OpCode::TRY.byte(),
            catch_offset,
            finally_offset,
            OpCode::NOP.byte(),
            OpCode::RET.byte(),
        ];
        assert!(
            validate_strict_script_reference(&script).is_err(),
            "{script:?}"
        );
    }

    let backward = [
        OpCode::NOP.byte(),
        OpCode::TRY.byte(),
        0xff,
        3,
        OpCode::RET.byte(),
    ];
    assert!(validate_strict_script_reference(&backward).is_ok());
}

#[test]
fn reference_checks_both_long_try_targets_from_current_ip() {
    let script = |catch_offset: i32, finally_offset: i32| {
        let mut bytes = vec![OpCode::TRY_L.byte()];
        bytes.extend_from_slice(&catch_offset.to_le_bytes());
        bytes.extend_from_slice(&finally_offset.to_le_bytes());
        bytes.extend_from_slice(&[OpCode::NOP.byte(), OpCode::RET.byte()]);
        bytes
    };
    assert!(validate_strict_script_reference(&script(9, 10)).is_ok());
    assert!(validate_strict_script(&script(9, 10)).is_err());
    for (catch_offset, finally_offset) in [(1, 10), (9, 5), (11, 10), (9, 11), (-1, 10)] {
        assert!(validate_strict_script_reference(&script(catch_offset, finally_offset)).is_err());
    }

    let mut backward = vec![OpCode::NOP.byte(), OpCode::TRY_L.byte()];
    backward.extend_from_slice(&(-1_i32).to_le_bytes());
    backward.extend_from_slice(&9_i32.to_le_bytes());
    backward.push(OpCode::RET.byte());
    assert!(validate_strict_script_reference(&backward).is_ok());
}

#[test]
fn reference_zero_try_offsets_remain_structurally_valid() {
    // Handler presence is a runtime check, not the strict Script constructor's rule.
    for (catch_offset, finally_offset) in [(0, 0), (0, 3), (3, 0)] {
        let script = [
            OpCode::TRY.byte(),
            catch_offset,
            finally_offset,
            OpCode::RET.byte(),
        ];
        assert!(validate_strict_script_reference(&script).is_ok());
    }
    let mut script = vec![OpCode::TRY_L.byte()];
    script.extend_from_slice(&[0; 8]);
    assert!(validate_strict_script_reference(&script).is_ok());
}

#[test]
fn reference_rejects_explicit_eof_targets_but_allows_fallthrough() {
    assert!(validate_strict_script_reference(&[OpCode::NOP.byte()]).is_ok());
    assert!(validate_strict_script_reference(&[OpCode::JMP.byte(), 2]).is_err());
    assert!(validate_strict_script_reference(&[OpCode::CALL.byte(), 2]).is_err());
    let pusha_eof = [OpCode::PUSHA.byte(), 5, 0, 0, 0];
    assert!(validate_strict_script_reference(&pusha_eof).is_err());
}

#[test]
fn reference_rejects_long_target_overflow_and_negative_offsets() {
    for offset in [i32::MAX, i32::MIN] {
        let mut script = vec![OpCode::NOP.byte(), OpCode::CALL_L.byte()];
        script.extend_from_slice(&offset.to_le_bytes());
        script.push(OpCode::RET.byte());
        assert!(validate_strict_script_reference(&script).is_err());
    }
    for overflow_catch in [true, false] {
        let mut script = vec![OpCode::NOP.byte(), OpCode::TRY_L.byte()];
        script.extend_from_slice(&(if overflow_catch { i32::MAX } else { 0 }).to_le_bytes());
        script.extend_from_slice(&(if overflow_catch { 0 } else { i32::MAX }).to_le_bytes());
        script.push(OpCode::RET.byte());
        assert!(validate_strict_script_reference(&script).is_err());
    }
}

#[test]
fn reference_preserves_shared_parser_and_type_validation() {
    assert!(validate_strict_script_reference(&[]).is_ok());
    for script in [
        vec![0xff],
        vec![OpCode::PUSHDATA1.byte(), 2, 1],
        vec![OpCode::TRY_L.byte(), 0],
    ] {
        assert!(validate_script_reference(&script, true).is_err());
        assert!(validate_script_reference(&script, false).is_err());
    }
    for opcode in [OpCode::NEWARRAY_T, OpCode::ISTYPE, OpCode::CONVERT] {
        assert!(validate_strict_script_reference(&[opcode.byte(), 0xff]).is_err());
    }
    for opcode in [OpCode::ISTYPE, OpCode::CONVERT] {
        assert!(
            validate_strict_script_reference(&[opcode.byte(), StackItemType::Any.to_byte()])
                .is_err()
        );
    }
    assert!(
        validate_strict_script_reference(&[
            OpCode::NEWARRAY_T.byte(),
            StackItemType::Any.to_byte()
        ])
        .is_ok()
    );
}

#[test]
fn reference_relaxed_mode_only_parses_instruction_boundaries() {
    let script = [OpCode::JMP.byte(), 1, OpCode::RET.byte()];
    assert!(validate_script_reference(&script, true).is_err());
    let reference = validate_script_reference(&script, false).expect("relaxed reference parsing");
    let legacy = validate_script(&script, false).expect("relaxed legacy parsing");
    for offset in 0..=script.len() {
        assert_eq!(
            reference.has_instruction_at(offset),
            legacy.has_instruction_at(offset)
        );
    }
}

#[test]
fn legacy_public_target_helpers_keep_following_instruction_origin() {
    let jump = Instruction::parse(&[OpCode::JMP.byte(), 0, OpCode::RET.byte()], 0).unwrap();
    assert_eq!(instruction_jump_target(&jump), Ok(2));
    let try_instruction =
        Instruction::parse(&[OpCode::TRY.byte(), 0, 0, OpCode::RET.byte()], 0).unwrap();
    assert_eq!(instruction_try_targets(&try_instruction), Ok((3, 3)));
}
