use neo_vm_rs::OpCode;

// Snapshot source:
// - neo-node v3.9.2 depends on Neo v3.9.1.
// - Neo v3.9.1 depends on NuGet package Neo.VM v3.9.0.
// - The canonical opcode table is neo-project/neo-vm v3.9.0
//   src/Neo.VM/OpCode.cs.
//
// Tuple fields: opcode byte, canonical name, operand_size(), operand_prefix().
const NEO_VM_3_9_OPCODE_SNAPSHOT: &[(u8, &str, usize, usize)] = &[
    (0x00, "PUSHINT8", 1, 0),
    (0x01, "PUSHINT16", 2, 0),
    (0x02, "PUSHINT32", 4, 0),
    (0x03, "PUSHINT64", 8, 0),
    (0x04, "PUSHINT128", 16, 0),
    (0x05, "PUSHINT256", 32, 0),
    (0x08, "PUSHT", 0, 0),
    (0x09, "PUSHF", 0, 0),
    (0x0A, "PUSHA", 4, 0),
    (0x0B, "PUSHNULL", 0, 0),
    (0x0C, "PUSHDATA1", 1, 1),
    (0x0D, "PUSHDATA2", 2, 2),
    (0x0E, "PUSHDATA4", 4, 4),
    (0x0F, "PUSHM1", 0, 0),
    (0x10, "PUSH0", 0, 0),
    (0x11, "PUSH1", 0, 0),
    (0x12, "PUSH2", 0, 0),
    (0x13, "PUSH3", 0, 0),
    (0x14, "PUSH4", 0, 0),
    (0x15, "PUSH5", 0, 0),
    (0x16, "PUSH6", 0, 0),
    (0x17, "PUSH7", 0, 0),
    (0x18, "PUSH8", 0, 0),
    (0x19, "PUSH9", 0, 0),
    (0x1A, "PUSH10", 0, 0),
    (0x1B, "PUSH11", 0, 0),
    (0x1C, "PUSH12", 0, 0),
    (0x1D, "PUSH13", 0, 0),
    (0x1E, "PUSH14", 0, 0),
    (0x1F, "PUSH15", 0, 0),
    (0x20, "PUSH16", 0, 0),
    (0x21, "NOP", 0, 0),
    (0x22, "JMP", 1, 0),
    (0x23, "JMP_L", 4, 0),
    (0x24, "JMPIF", 1, 0),
    (0x25, "JMPIF_L", 4, 0),
    (0x26, "JMPIFNOT", 1, 0),
    (0x27, "JMPIFNOT_L", 4, 0),
    (0x28, "JMPEQ", 1, 0),
    (0x29, "JMPEQ_L", 4, 0),
    (0x2A, "JMPNE", 1, 0),
    (0x2B, "JMPNE_L", 4, 0),
    (0x2C, "JMPGT", 1, 0),
    (0x2D, "JMPGT_L", 4, 0),
    (0x2E, "JMPGE", 1, 0),
    (0x2F, "JMPGE_L", 4, 0),
    (0x30, "JMPLT", 1, 0),
    (0x31, "JMPLT_L", 4, 0),
    (0x32, "JMPLE", 1, 0),
    (0x33, "JMPLE_L", 4, 0),
    (0x34, "CALL", 1, 0),
    (0x35, "CALL_L", 4, 0),
    (0x36, "CALLA", 0, 0),
    (0x37, "CALLT", 2, 0),
    (0x38, "ABORT", 0, 0),
    (0x39, "ASSERT", 0, 0),
    (0x3A, "THROW", 0, 0),
    (0x3B, "TRY", 2, 0),
    (0x3C, "TRY_L", 8, 0),
    (0x3D, "ENDTRY", 1, 0),
    (0x3E, "ENDTRY_L", 4, 0),
    (0x3F, "ENDFINALLY", 0, 0),
    (0x40, "RET", 0, 0),
    (0x41, "SYSCALL", 4, 0),
    (0x43, "DEPTH", 0, 0),
    (0x45, "DROP", 0, 0),
    (0x46, "NIP", 0, 0),
    (0x48, "XDROP", 0, 0),
    (0x49, "CLEAR", 0, 0),
    (0x4A, "DUP", 0, 0),
    (0x4B, "OVER", 0, 0),
    (0x4D, "PICK", 0, 0),
    (0x4E, "TUCK", 0, 0),
    (0x50, "SWAP", 0, 0),
    (0x51, "ROT", 0, 0),
    (0x52, "ROLL", 0, 0),
    (0x53, "REVERSE3", 0, 0),
    (0x54, "REVERSE4", 0, 0),
    (0x55, "REVERSEN", 0, 0),
    (0x56, "INITSSLOT", 1, 0),
    (0x57, "INITSLOT", 2, 0),
    (0x58, "LDSFLD0", 0, 0),
    (0x59, "LDSFLD1", 0, 0),
    (0x5A, "LDSFLD2", 0, 0),
    (0x5B, "LDSFLD3", 0, 0),
    (0x5C, "LDSFLD4", 0, 0),
    (0x5D, "LDSFLD5", 0, 0),
    (0x5E, "LDSFLD6", 0, 0),
    (0x5F, "LDSFLD", 1, 0),
    (0x60, "STSFLD0", 0, 0),
    (0x61, "STSFLD1", 0, 0),
    (0x62, "STSFLD2", 0, 0),
    (0x63, "STSFLD3", 0, 0),
    (0x64, "STSFLD4", 0, 0),
    (0x65, "STSFLD5", 0, 0),
    (0x66, "STSFLD6", 0, 0),
    (0x67, "STSFLD", 1, 0),
    (0x68, "LDLOC0", 0, 0),
    (0x69, "LDLOC1", 0, 0),
    (0x6A, "LDLOC2", 0, 0),
    (0x6B, "LDLOC3", 0, 0),
    (0x6C, "LDLOC4", 0, 0),
    (0x6D, "LDLOC5", 0, 0),
    (0x6E, "LDLOC6", 0, 0),
    (0x6F, "LDLOC", 1, 0),
    (0x70, "STLOC0", 0, 0),
    (0x71, "STLOC1", 0, 0),
    (0x72, "STLOC2", 0, 0),
    (0x73, "STLOC3", 0, 0),
    (0x74, "STLOC4", 0, 0),
    (0x75, "STLOC5", 0, 0),
    (0x76, "STLOC6", 0, 0),
    (0x77, "STLOC", 1, 0),
    (0x78, "LDARG0", 0, 0),
    (0x79, "LDARG1", 0, 0),
    (0x7A, "LDARG2", 0, 0),
    (0x7B, "LDARG3", 0, 0),
    (0x7C, "LDARG4", 0, 0),
    (0x7D, "LDARG5", 0, 0),
    (0x7E, "LDARG6", 0, 0),
    (0x7F, "LDARG", 1, 0),
    (0x80, "STARG0", 0, 0),
    (0x81, "STARG1", 0, 0),
    (0x82, "STARG2", 0, 0),
    (0x83, "STARG3", 0, 0),
    (0x84, "STARG4", 0, 0),
    (0x85, "STARG5", 0, 0),
    (0x86, "STARG6", 0, 0),
    (0x87, "STARG", 1, 0),
    (0x88, "NEWBUFFER", 0, 0),
    (0x89, "MEMCPY", 0, 0),
    (0x8B, "CAT", 0, 0),
    (0x8C, "SUBSTR", 0, 0),
    (0x8D, "LEFT", 0, 0),
    (0x8E, "RIGHT", 0, 0),
    (0x90, "INVERT", 0, 0),
    (0x91, "AND", 0, 0),
    (0x92, "OR", 0, 0),
    (0x93, "XOR", 0, 0),
    (0x97, "EQUAL", 0, 0),
    (0x98, "NOTEQUAL", 0, 0),
    (0x99, "SIGN", 0, 0),
    (0x9A, "ABS", 0, 0),
    (0x9B, "NEGATE", 0, 0),
    (0x9C, "INC", 0, 0),
    (0x9D, "DEC", 0, 0),
    (0x9E, "ADD", 0, 0),
    (0x9F, "SUB", 0, 0),
    (0xA0, "MUL", 0, 0),
    (0xA1, "DIV", 0, 0),
    (0xA2, "MOD", 0, 0),
    (0xA3, "POW", 0, 0),
    (0xA4, "SQRT", 0, 0),
    (0xA5, "MODMUL", 0, 0),
    (0xA6, "MODPOW", 0, 0),
    (0xA8, "SHL", 0, 0),
    (0xA9, "SHR", 0, 0),
    (0xAA, "NOT", 0, 0),
    (0xAB, "BOOLAND", 0, 0),
    (0xAC, "BOOLOR", 0, 0),
    (0xB1, "NZ", 0, 0),
    (0xB3, "NUMEQUAL", 0, 0),
    (0xB4, "NUMNOTEQUAL", 0, 0),
    (0xB5, "LT", 0, 0),
    (0xB6, "LE", 0, 0),
    (0xB7, "GT", 0, 0),
    (0xB8, "GE", 0, 0),
    (0xB9, "MIN", 0, 0),
    (0xBA, "MAX", 0, 0),
    (0xBB, "WITHIN", 0, 0),
    (0xBE, "PACKMAP", 0, 0),
    (0xBF, "PACKSTRUCT", 0, 0),
    (0xC0, "PACK", 0, 0),
    (0xC1, "UNPACK", 0, 0),
    (0xC2, "NEWARRAY0", 0, 0),
    (0xC3, "NEWARRAY", 0, 0),
    (0xC4, "NEWARRAY_T", 1, 0),
    (0xC5, "NEWSTRUCT0", 0, 0),
    (0xC6, "NEWSTRUCT", 0, 0),
    (0xC8, "NEWMAP", 0, 0),
    (0xCA, "SIZE", 0, 0),
    (0xCB, "HASKEY", 0, 0),
    (0xCC, "KEYS", 0, 0),
    (0xCD, "VALUES", 0, 0),
    (0xCE, "PICKITEM", 0, 0),
    (0xCF, "APPEND", 0, 0),
    (0xD0, "SETITEM", 0, 0),
    (0xD1, "REVERSEITEMS", 0, 0),
    (0xD2, "REMOVE", 0, 0),
    (0xD3, "CLEARITEMS", 0, 0),
    (0xD4, "POPITEM", 0, 0),
    (0xD8, "ISNULL", 0, 0),
    (0xD9, "ISTYPE", 1, 0),
    (0xDB, "CONVERT", 1, 0),
    (0xE0, "ABORTMSG", 0, 0),
    (0xE1, "ASSERTMSG", 0, 0),
];

#[test]
fn opcode_table_matches_neo_node_3_9_2_vm_package_snapshot() {
    assert_eq!(NEO_VM_3_9_OPCODE_SNAPSHOT.len(), 196);
    assert_eq!(NEO_VM_3_9_OPCODE_SNAPSHOT.len(), OpCode::ALL.len());

    let mut accepted = [false; 256];
    for (index, &(byte, name, operand_size, operand_prefix)) in
        NEO_VM_3_9_OPCODE_SNAPSHOT.iter().enumerate()
    {
        assert!(
            !accepted[byte as usize],
            "duplicate opcode byte {byte:#04x}"
        );
        accepted[byte as usize] = true;

        let opcode = OpCode::try_from(byte).expect("canonical opcode should decode");
        assert_eq!(opcode.byte(), byte, "{name} byte mismatch");
        assert_eq!(opcode.name(), name, "opcode name mismatch for {byte:#04x}");
        assert_eq!(
            opcode.operand_size(),
            operand_size,
            "{name} operand size mismatch"
        );
        assert_eq!(
            opcode.operand_prefix(),
            operand_prefix,
            "{name} operand prefix mismatch"
        );
        assert_eq!(opcode.to_string(), name);
        assert_eq!(OpCode::from_u8(byte), Some(opcode));
        assert_eq!(
            OpCode::ALL[index],
            opcode,
            "OpCode::ALL order mismatch at index {index}"
        );
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
