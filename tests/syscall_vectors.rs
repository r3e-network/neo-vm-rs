use neo_vm_rs::{interop_hash, syscall_arg_count};

#[test]
fn known_syscall_hashes_match_neo_sha256_little_endian_rule() {
    let vectors = [
        ("System.Contract.Call", 0x525b_7d62, 4),
        ("System.Contract.CallNative", 0x677b_f71a, usize::MAX),
        ("System.Contract.Create", 0x852c_35ce, 2),
        ("System.Contract.Update", 0x1d33_c631, 2),
        ("System.Contract.NativeOnPersist", 0x93bc_db2e, 0),
        ("System.Contract.NativePostPersist", 0x165d_a144, 0),
        ("System.Runtime.Platform", 0xf6fc_79b2, 0),
        ("System.Runtime.GetTrigger", 0xa038_7de9, 0),
        ("System.Runtime.CheckWitness", 0x8cec_27f8, 1),
        ("System.Runtime.Notify", 0x616f_0195, 2),
        ("System.Runtime.Log", 0x9647_e7cf, 1),
        ("System.Storage.GetContext", 0xce67_f69b, 0),
        ("System.Storage.Get", 0x31e8_5d92, 2),
        ("System.Storage.Put", 0x8418_3fe6, 3),
        ("System.Storage.Delete", 0xedc5_582f, 2),
        ("System.Crypto.CheckSig", 0x27b3_e756, 2),
        ("System.Crypto.CheckMultisig", 0x3adc_d09e, usize::MAX),
        ("System.Iterator.Next", 0x9ced_089c, 1),
        ("System.Iterator.Value", 0x1dbf_54f3, 1),
    ];

    for (name, hash, arg_count) in vectors {
        assert_eq!(interop_hash(name), hash, "{name} hash mismatch");
        assert_eq!(
            syscall_arg_count(hash),
            arg_count,
            "{name} arg count mismatch"
        );
    }
}

#[test]
fn unknown_syscall_hashes_request_full_stack_forwarding() {
    assert_eq!(
        syscall_arg_count(interop_hash("System.Test.Unknown")),
        usize::MAX
    );
}
