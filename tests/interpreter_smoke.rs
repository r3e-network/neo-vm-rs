use neo_vm_rs::{
    ExecutionResult, OpCode, StackValue, VmState, interpret, interpret_with_stack_and_syscalls_at,
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

#[test]
fn jump_comparison_faults_on_null_operand() {
    // Canonical JMP* comparisons (JmpEq/Ne/Gt/Ge/Lt/Le) call GetInteger on BOTH
    // operands with no null guard, so a null operand faults uncatchably rather
    // than being treated as no-jump (the pre-fix behavior for JMPGT/etc.).
    let result = interpret(&[
        OpCode::PUSH5.byte(),
        OpCode::PUSHNULL.byte(),
        OpCode::JMPGT.byte(),
        0x03,
        OpCode::RET.byte(),
    ]);
    match result {
        Err(_) => {}
        Ok(r) => assert_eq!(
            r.state,
            VmState::Fault,
            "JMPGT with a null operand must fault, got {r:?}"
        ),
    }
}

#[test]
fn jmpeq_compares_by_integer_value_not_bytes() {
    // ByteString [0x01,0x00] (=1) and PUSH1 (=1) are integer-equal; canonical
    // JMPEQ uses GetInteger, so it jumps even though the byte representations
    // differ. The pre-fix structural vm_equal did NOT jump (bytes unequal).
    //   ip0 PUSHDATA1 02 01 00   (ByteString {01,00} = 1)
    //   ip4 PUSH1
    //   ip5 JMPEQ +4             -> ip9 on integer-equal
    //   ip7 PUSH3; ip8 RET       (not-equal path)
    //   ip9 PUSH7; ip10 RET      (equal path)
    let result = interpret(&[
        OpCode::PUSHDATA1.byte(),
        0x02,
        0x01,
        0x00,
        OpCode::PUSH1.byte(),
        OpCode::JMPEQ.byte(),
        0x04,
        OpCode::PUSH3.byte(),
        OpCode::RET.byte(),
        OpCode::PUSH7.byte(),
        OpCode::RET.byte(),
    ])
    .expect("JMPEQ script should execute");
    assert_eq!(result.state, VmState::Halt);
    assert!(
        result.stack.contains(&StackValue::Integer(7)),
        "JMPEQ must compare by integer value (jump taken -> 7); stack={:?}",
        result.stack
    );
}

#[test]
fn haskey_faults_on_negative_and_null_index() {
    // Canonical HASKEY on Array/Buffer/ByteString does (int)key.GetInteger() then
    // `if (index < 0) throw` — so a negative index, a null key, or an Int32
    // overflow faults uncatchably (pre-fix returned false / coerced null to 0).
    for key in [OpCode::PUSHM1.byte(), OpCode::PUSHNULL.byte()] {
        let result = interpret(&[
            OpCode::NEWARRAY0.byte(),
            key,
            OpCode::HASKEY.byte(),
            OpCode::RET.byte(),
        ]);
        match result {
            Err(_) => {}
            Ok(r) => assert_eq!(
                r.state,
                VmState::Fault,
                "HASKEY with negative/null index must fault, got {r:?}"
            ),
        }
    }
}

#[test]
fn remove_faults_on_null_index() {
    // REMOVE pops its key as PrimitiveType + (int)GetInteger; a null key faults
    // uncatchably (pre-fix coerced null to index 0 and removed element 0).
    //   PUSH1; NEWARRAY  -> Array[Null] (len 1)
    //   PUSHNULL; REMOVE -> fault (null index)
    let result = interpret(&[
        OpCode::PUSH1.byte(),
        OpCode::NEWARRAY.byte(),
        OpCode::PUSHNULL.byte(),
        OpCode::REMOVE.byte(),
        OpCode::RET.byte(),
    ]);
    match result {
        Err(_) => {}
        Ok(r) => assert_eq!(
            r.state,
            VmState::Fault,
            "REMOVE with a null index must fault, got {r:?}"
        ),
    }
}

#[test]
fn remove_missing_map_key_is_noop() {
    // Canonical REMOVE on a map is `map.Remove(key)` — a no-op when the key is
    // absent, not a fault (pre-fix neo-vm-rs faulted "key not found").
    let result = interpret(&[
        OpCode::NEWMAP.byte(),
        OpCode::PUSH9.byte(),
        OpCode::REMOVE.byte(),
        OpCode::RET.byte(),
    ])
    .expect("REMOVE of an absent map key should not fault");
    assert_eq!(result.state, VmState::Halt);
}

#[test]
fn equal_compares_nested_compounds_in_struct_by_reference() {
    // Canonical Struct.Equals compares nested Array/Map/Buffer by REFERENCE, not
    // content. Two structs each holding a DISTINCT (fresh) empty array are NOT
    // equal, even though the arrays have identical content (pre-fix
    // structurally_equal compared them by content -> wrongly equal). D14.
    //   NEWARRAY0; PUSH1; PACKSTRUCT  -> struct{ [] }   (array A)
    //   NEWARRAY0; PUSH1; PACKSTRUCT  -> struct{ [] }   (array B, distinct id)
    //   EQUAL
    let result = interpret(&[
        OpCode::NEWARRAY0.byte(),
        OpCode::PUSH1.byte(),
        OpCode::PACKSTRUCT.byte(),
        OpCode::NEWARRAY0.byte(),
        OpCode::PUSH1.byte(),
        OpCode::PACKSTRUCT.byte(),
        OpCode::EQUAL.byte(),
        OpCode::RET.byte(),
    ])
    .expect("EQUAL of two structs should execute");
    assert_eq!(result.state, VmState::Halt);
    assert_eq!(
        result.stack,
        vec![StackValue::Boolean(false)],
        "structs with distinct nested arrays must compare unequal (reference equality)"
    );
}

#[test]
fn map_accepts_large_integer_key() {
    // A BigInteger (large-integer representation, e.g. from PUSHINT128) is a
    // valid PrimitiveType integer map key in canonical NeoVM; SETITEM must not
    // reject it (pre-fix validate_map_key faulted "map key must be primitive"). D15.
    //   NEWMAP; PUSHINT128 1; PUSH5; SETITEM
    let mut script = vec![OpCode::NEWMAP.byte(), OpCode::PUSHINT128.byte(), 0x01];
    script.extend_from_slice(&[0u8; 15]); // 16-byte little-endian 1
    script.extend_from_slice(&[
        OpCode::PUSH5.byte(),
        OpCode::SETITEM.byte(),
        OpCode::RET.byte(),
    ]);
    let result =
        interpret(&script).expect("SETITEM with a large-integer map key should not fault");
    assert_eq!(result.state, VmState::Halt);
}

#[test]
fn setitem_out_of_range_is_catchable() {
    // Canonical SETITEM array/buffer out-of-range is a CatchableException, so a
    // surrounding TRY/CATCH handles it and the script HALTs (pre-fix it was an
    // uncatchable fault).
    //   IP0  TRY catch=+9 finally=0
    //   IP3  NEWARRAY0; IP4 PUSH5; IP5 PUSH9; IP6 SETITEM  (index 5 vs len 0 -> OOR)
    //   IP7  ENDTRY +5 -> IP12
    //   IP9  PUSH1 (catch body); IP10 ENDTRY +2 -> IP12
    //   IP12 RET
    let result = interpret(&[
        OpCode::TRY.byte(),
        0x09,
        0x00,
        OpCode::NEWARRAY0.byte(),
        OpCode::PUSH5.byte(),
        OpCode::PUSH9.byte(),
        OpCode::SETITEM.byte(),
        OpCode::ENDTRY.byte(),
        0x05,
        OpCode::PUSH1.byte(),
        OpCode::ENDTRY.byte(),
        0x02,
        OpCode::RET.byte(),
    ]);
    match result {
        Ok(r) => assert_eq!(
            r.state,
            VmState::Halt,
            "SETITEM out-of-range inside a TRY must be caught (HALT), got {r:?}"
        ),
        Err(e) => panic!("SETITEM out-of-range should be catchable, but faulted uncatchably: {e}"),
    }
}

#[test]
fn booland_faults_on_oversized_bytestring_right_operand() {
    // BOOLAND coerces BOTH operands via canonical GetBoolean, which faults
    // (uncatchably) on a ByteString wider than Integer.MaxSize (32 bytes).
    // Critically EAGER (no short-circuit): a FALSY left must NOT mask the
    // oversized right's fault (a lax `false && right` would wrongly HALT).
    // Pre-fix BOOLAND used to_bool, which never faulted -> consensus divergence.
    //   PUSH0                       (left = 0, falsy)
    //   PUSHDATA1 0x21 <33 bytes>   (right = 33-byte ByteString)
    //   BOOLAND; RET
    let mut script = vec![OpCode::PUSH0.byte(), OpCode::PUSHDATA1.byte(), 0x21];
    script.extend_from_slice(&[0u8; 33]);
    script.extend_from_slice(&[OpCode::BOOLAND.byte(), OpCode::RET.byte()]);
    match interpret(&script) {
        Err(_) => {}
        Ok(r) => assert_eq!(
            r.state,
            VmState::Fault,
            "BOOLAND with a >32-byte ByteString operand must fault even when the \
             other operand is falsy (eager GetBoolean), got {r:?}"
        ),
    }
}

#[test]
fn booland_faults_on_oversized_bytestring_left_operand() {
    // Symmetric guard: oversized LEFT operand, falsy right — must still fault
    // (protects against a future left-first short-circuit refactor).
    //   PUSHDATA1 0x21 <33 bytes>   (left = 33-byte ByteString)
    //   PUSH0                       (right = 0, falsy)
    //   BOOLAND; RET
    let mut script = vec![OpCode::PUSHDATA1.byte(), 0x21];
    script.extend_from_slice(&[0u8; 33]);
    script.extend_from_slice(&[OpCode::PUSH0.byte(), OpCode::BOOLAND.byte(), OpCode::RET.byte()]);
    match interpret(&script) {
        Err(_) => {}
        Ok(r) => assert_eq!(
            r.state,
            VmState::Fault,
            "BOOLAND with a >32-byte ByteString left operand must fault, got {r:?}"
        ),
    }
}

#[test]
fn boolor_faults_on_oversized_bytestring_operand() {
    // BOOLOR likewise evaluates both operands' GetBoolean eagerly: a TRUTHY left
    // must NOT mask the oversized right's fault (a lax `true || right` HALTs).
    //   PUSH1                       (left = 1, truthy)
    //   PUSHDATA1 0x21 <33 bytes>   (right = 33-byte ByteString)
    //   BOOLOR; RET
    let mut script = vec![OpCode::PUSH1.byte(), OpCode::PUSHDATA1.byte(), 0x21];
    script.extend_from_slice(&[0u8; 33]);
    script.extend_from_slice(&[OpCode::BOOLOR.byte(), OpCode::RET.byte()]);
    match interpret(&script) {
        Err(_) => {}
        Ok(r) => assert_eq!(
            r.state,
            VmState::Fault,
            "BOOLOR with a >32-byte ByteString operand must fault even when the \
             other operand is truthy (eager GetBoolean), got {r:?}"
        ),
    }
}

#[test]
fn booland_allows_32_byte_bytestring_operand() {
    // Boundary is strictly `>`: a 32-byte ByteString is a valid GetBoolean operand
    // (all-zero -> false), so BOOLAND(true, false) HALTs with false (no fault).
    //   PUSH1                            (left = 1, truthy)
    //   PUSHDATA1 0x20 <32 zero bytes>   (right = 32-byte ByteString, GetBoolean=false)
    //   BOOLAND; RET
    let mut script = vec![OpCode::PUSH1.byte(), OpCode::PUSHDATA1.byte(), 0x20];
    script.extend_from_slice(&[0u8; 32]);
    script.extend_from_slice(&[OpCode::BOOLAND.byte(), OpCode::RET.byte()]);
    let result = interpret(&script).expect("BOOLAND with a 32-byte operand must not fault");
    assert_eq!(result.state, VmState::Halt);
    assert_eq!(result.stack, vec![StackValue::Boolean(false)]);
}

#[test]
fn buffer_as_integer_operand_faults() {
    // Canonical (int)GetInteger() throws (uncatchable) on a Buffer — Buffer is
    // NOT a PrimitiveType, so its GetInteger hits the throwing base. An opcode
    // count/index popped as a Buffer must FAULT, not decode the Buffer's bytes
    // as a little-endian integer (pre-fix HALTed). D-1.
    //   PUSH1; NEWBUFFER -> Buffer[0x00]; second NEWBUFFER pops it as size -> fault
    let result = interpret(&[
        OpCode::PUSH1.byte(),
        OpCode::NEWBUFFER.byte(),
        OpCode::NEWBUFFER.byte(),
        OpCode::RET.byte(),
    ]);
    match result {
        Err(_) => {}
        Ok(r) => assert_eq!(
            r.state,
            VmState::Fault,
            "NEWBUFFER with a Buffer size operand must fault (Buffer.GetInteger throws), got {r:?}"
        ),
    }
}

#[test]
fn convert_null_and_integer_to_pointer() {
    // Canonical: Null.ConvertTo(Pointer) returns Null (Pointer is a defined,
    // non-Any type); base StackItem.ConvertTo returns self when type==Type.
    // Pre-fix neo-vm-rs faulted ALL CONVERT->Pointer (0x10 missing from the
    // target whitelist, checked before the Null short-circuit). D-8.
    let null_to_ptr = interpret(&[
        OpCode::PUSHNULL.byte(),
        OpCode::CONVERT.byte(),
        0x10,
        OpCode::RET.byte(),
    ])
    .expect("CONVERT Null->Pointer must not fault");
    assert_eq!(null_to_ptr.state, VmState::Halt);
    assert_eq!(null_to_ptr.stack, vec![StackValue::Null]);
    // An Integer source -> Pointer is an invalid conversion (InvalidCastException).
    let int_to_ptr = interpret(&[
        OpCode::PUSH1.byte(),
        OpCode::CONVERT.byte(),
        0x10,
        OpCode::RET.byte(),
    ]);
    match int_to_ptr {
        Err(_) => {}
        Ok(r) => assert_eq!(
            r.state,
            VmState::Fault,
            "CONVERT Integer->Pointer must fault, got {r:?}"
        ),
    }
}

#[test]
fn pushdata4_over_max_item_size_faults() {
    // Canonical PushData4 asserts MaxItemSize (ushort::MAX*2 = 131070) on the
    // operand length; 131071 bytes faults. Pre-fix the only bound was a 1 MiB
    // cap, so 131071..=1048576 HALTed where canonical FAULTs. D-2.
    let mk = |len: u32| {
        let mut s = vec![OpCode::PUSHDATA4.byte()];
        s.extend_from_slice(&len.to_le_bytes());
        s.resize(s.len() + len as usize, 0u8);
        s.push(OpCode::RET.byte());
        s
    };
    match interpret(&mk(131071)) {
        Err(_) => {}
        Ok(r) => assert_eq!(
            r.state,
            VmState::Fault,
            "PUSHDATA4 of 131071 bytes must fault (> MaxItemSize), got {r:?}"
        ),
    }
    // Boundary: 131070 == MaxItemSize is allowed.
    let ok = interpret(&mk(131070)).expect("PUSHDATA4 of 131070 bytes must not fault");
    assert_eq!(ok.state, VmState::Halt);
}

#[test]
fn try_with_both_offsets_zero_faults() {
    // Canonical ExecuteTry faults (uncatchable) when catchOffset==0 AND
    // finallyOffset==0 (pre-fix neo-vm-rs pushed a degenerate frame and
    // continued). D-3.   TRY 00 00 ; RET
    let result = interpret(&[OpCode::TRY.byte(), 0x00, 0x00, OpCode::RET.byte()]);
    match result {
        Err(_) => {}
        Ok(r) => assert_eq!(
            r.state,
            VmState::Fault,
            "TRY with both offsets 0 must fault, got {r:?}"
        ),
    }
}

#[test]
fn jmp_to_script_end_faults() {
    // Canonical ExecuteJump faults when position >= Script.Length (strict >=),
    // unlike CALL which allows ==Length. A JMP landing exactly at the end of the
    // script must FAULT, not fall through and HALT (pre-fix). D-4.
    //   [JMP, 0x02] (len 2): target = 0 + 2 = 2 == Script.Length -> fault
    let result = interpret(&[OpCode::JMP.byte(), 0x02]);
    match result {
        Err(_) => {}
        Ok(r) => assert_eq!(
            r.state,
            VmState::Fault,
            "JMP to target == Script.Length must fault, got {r:?}"
        ),
    }
}

#[test]
fn endtry_in_finally_block_faults() {
    // Canonical ExecuteEndTry faults (uncatchable) when the frame is already in
    // its FINALLY block. A second ENDTRY reached inside the finally body must
    // FAULT (pre-fix neo-vm-rs popped + jumped). D-7.
    //   ip0 TRY catch=0 finally=5   (finally_ip = 5)
    //   ip3 ENDTRY +4               (first ENDTRY -> enter finally, jump to ip5)
    //   ip5 ENDTRY +2               (second ENDTRY, now in finally -> FAULT)
    //   ip7 RET
    let result = interpret(&[
        OpCode::TRY.byte(),
        0x00,
        0x05,
        OpCode::ENDTRY.byte(),
        0x04,
        OpCode::ENDTRY.byte(),
        0x02,
        OpCode::RET.byte(),
    ]);
    match result {
        Err(_) => {}
        Ok(r) => assert_eq!(
            r.state,
            VmState::Fault,
            "a second ENDTRY inside the FINALLY block must fault, got {r:?}"
        ),
    }
}

#[test]
fn endtry_out_of_bounds_target_faults() {
    // Canonical assigns the ENDTRY end target through the InstructionPointer
    // setter, which faults on value > Script.Length. An out-of-range end target
    // must FAULT, not fall through and HALT (pre-fix). D-9.
    //   ip0 TRY catch=3 finally=0
    //   ip3 ENDTRY +127  (target = 3 + 127 = 130 > len(5) -> fault)
    let result = interpret(&[
        OpCode::TRY.byte(),
        0x03,
        0x00,
        OpCode::ENDTRY.byte(),
        0x7F,
    ]);
    match result {
        Err(_) => {}
        Ok(r) => assert_eq!(
            r.state,
            VmState::Fault,
            "ENDTRY with an out-of-bounds end target must fault, got {r:?}"
        ),
    }
}

fn _assert_result_is_public(_: ExecutionResult) {}

/// A faulting script may surface either as `Ok` with `VmState::Fault` or as an
/// `Err` at the executor boundary; both are a FAULT for our purposes.
fn assert_faults(script: &[u8], context: &str) {
    if let Ok(result) = interpret(script) {
        assert_eq!(
            result.state,
            VmState::Fault,
            "{context}: expected FAULT, got {:?}",
            result.state
        );
    }
}

// Regression: Neo v3.10.0 consistency residuals (verified against C# Neo.VM).

#[test]
fn endtry_without_matching_try_faults() {
    // ENDTRY (0x3d) offset 0 with no preceding TRY. C# ExecuteEndTry throws
    // "The corresponding TRY block cannot be found." => FAULT (not HALT).
    assert_faults(&[0x3d, 0x00], "ENDTRY without TRY");
}

#[test]
fn endtry_l_without_matching_try_faults() {
    // ENDTRY_L (0x3e) offset 0 with no preceding TRY => FAULT.
    assert_faults(&[0x3e, 0x00, 0x00, 0x00, 0x00], "ENDTRY_L without TRY");
}

#[test]
fn newarray_t_invalid_type_operand_faults() {
    // PUSH1; NEWARRAY_T type=0x02. 0x02 is not a defined StackItemType, so C#
    // NewArray_T's `Enum.IsDefined` check throws => FAULT (not HALT).
    assert_faults(&[0x11, 0xc4, 0x02], "NEWARRAY_T invalid type");
}

#[test]
fn newarray_t_boolean_elements_default_to_false() {
    // PUSH2; NEWARRAY_T type=Boolean(0x20); DUP; PUSH0; PICKITEM.
    // C# fills Boolean arrays with StackItem.False, so the picked element is
    // Boolean(false), not Null.
    let result = interpret(&[0x12, 0xc4, 0x20, 0x4a, 0x10, 0xce])
        .expect("NEWARRAY_T Boolean script should execute");
    assert_eq!(result.state, VmState::Halt);
    assert_eq!(
        result.stack.last(),
        Some(&StackValue::Boolean(false)),
        "picked Boolean array element must default to false, got {:?}",
        result.stack.last()
    );
}
