//! Shared NeoVM syscall helpers.

use sha2::{Digest, Sha256};

/// Returns the first four bytes of SHA-256(name) as a little-endian syscall id.
#[must_use]
#[inline]
pub fn interop_hash(name: &str) -> u32 {
    if let Some(hash) = known_interop_hash(name) {
        return hash;
    }

    let digest = Sha256::digest(name.as_bytes());
    u32::from_le_bytes([digest[0], digest[1], digest[2], digest[3]])
}

#[inline]
fn known_interop_hash(name: &str) -> Option<u32> {
    match name {
        // System.Contract
        "System.Contract.Call" => Some(0x525b_7d62),
        "System.Contract.CallNative" => Some(0x677b_f71a),
        "System.Contract.Create" => Some(0x852c_35ce),
        "System.Contract.Update" => Some(0x1d33_c631),
        "System.Contract.NativeOnPersist" => Some(0x93bc_db2e),
        "System.Contract.NativePostPersist" => Some(0x165d_a144),
        "System.Contract.GetCallFlags" => Some(0x813a_da95),
        "System.Contract.CreateStandardAccount" => Some(0x0287_99cf),
        "System.Contract.CreateMultisigAccount" => Some(0x09e9_336a),
        // System.Runtime
        "System.Runtime.CheckWitness" => Some(0x8cec_27f8),
        "System.Runtime.Notify" => Some(0x616f_0195),
        "System.Runtime.Log" => Some(0x9647_e7cf),
        "System.Runtime.GetNotifications" => Some(0xf135_4327),
        "System.Runtime.BurnGas" => Some(0xbc8c_5ac3),
        "System.Runtime.LoadScript" => Some(0x8f80_0cb3),
        "System.Runtime.Platform" => Some(0xf6fc_79b2),
        "System.Runtime.GetTrigger" => Some(0xa038_7de9),
        "System.Runtime.GetTime" => Some(0x0388_c3b7),
        "System.Runtime.GetScriptContainer" => Some(0x3008_512d),
        "System.Runtime.GetExecutingScriptHash" => Some(0x74a8_fedb),
        "System.Runtime.GetCallingScriptHash" => Some(0x3c6e_5339),
        "System.Runtime.GetEntryScriptHash" => Some(0x38e2_b4f9),
        "System.Runtime.GetInvocationCounter" => Some(0x4311_2784),
        "System.Runtime.GasLeft" => Some(0xced8_8814),
        "System.Runtime.GetAddressVersion" => Some(0xdc92_494c),
        "System.Runtime.CurrentSigners" => Some(0x8b18_f1ac),
        "System.Runtime.GetNetwork" => Some(0xe0a0_fbc5),
        "System.Runtime.GetRandom" => Some(0x28a9_de6b),
        // System.Storage
        "System.Storage.GetContext" => Some(0xce67_f69b),
        "System.Storage.GetReadOnlyContext" => Some(0xe26b_b4f6),
        "System.Storage.AsReadOnly" => Some(0xe9bf_4c76),
        "System.Storage.Local.Get" => Some(0xe85e_8dd5),
        "System.Storage.Local.Put" => Some(0x0ae3_0c39),
        "System.Storage.Local.Delete" => Some(0x94f5_5475),
        "System.Storage.Local.Find" => Some(0xf352_7607),
        "System.Storage.Get" => Some(0x31e8_5d92),
        "System.Storage.Find" => Some(0x9ab8_30df),
        "System.Storage.Put" => Some(0x8418_3fe6),
        "System.Storage.Delete" => Some(0xedc5_582f),
        // System.Crypto
        "System.Crypto.CheckSig" => Some(0x27b3_e756),
        "System.Crypto.CheckMultisig" => Some(0x3adc_d09e),
        // System.Iterator
        "System.Iterator.Next" => Some(0x9ced_089c),
        "System.Iterator.Value" => Some(0x1dbf_54f3),
        _ => None,
    }
}

/// Returns the number of stack arguments consumed by a known NeoVM syscall.
///
/// Unknown or count-dependent syscalls return `usize::MAX`, which tells callers
/// to pass the full stack through the host boundary.
#[must_use]
#[inline]
pub fn syscall_arg_count(api: u32) -> usize {
    match api {
        // System.Contract
        0x525b_7d62 => 4,          // System.Contract.Call
        0x677b_f71a => usize::MAX, // System.Contract.CallNative
        0x852c_35ce => 2,          // System.Contract.Create
        0x1d33_c631 => 2,          // System.Contract.Update
        0x93bc_db2e => 0,          // System.Contract.NativeOnPersist
        0x165d_a144 => 0,          // System.Contract.NativePostPersist
        0x813a_da95 => 0,          // System.Contract.GetCallFlags
        0x0287_99cf => 1,          // System.Contract.CreateStandardAccount
        0x09e9_336a => 2,          // System.Contract.CreateMultisigAccount
        // System.Runtime
        0x8cec_27f8 => 1, // System.Runtime.CheckWitness
        0x616f_0195 => 2, // System.Runtime.Notify
        0x9647_e7cf => 1, // System.Runtime.Log
        0xf135_4327 => 1, // System.Runtime.GetNotifications
        0xbc8c_5ac3 => 1, // System.Runtime.BurnGas
        0x8f80_0cb3 => 3, // System.Runtime.LoadScript
        0x0388_c3b7 | 0xf6fc_79b2 | 0xa038_7de9 | 0xe0a0_fbc5 | 0xdc92_494c | 0x3008_512d
        | 0x74a8_fedb | 0x3c6e_5339 | 0x38e2_b4f9 | 0x28a9_de6b | 0xced8_8814 | 0x4311_2784
        | 0x8b18_f1ac => 0,
        // System.Storage
        0xce67_f69b => 0, // System.Storage.GetContext
        0xe26b_b4f6 => 0, // System.Storage.GetReadOnlyContext
        0xe9bf_4c76 => 1, // System.Storage.AsReadOnly
        0xe85e_8dd5 => 1, // System.Storage.Local.Get
        0x0ae3_0c39 => 2, // System.Storage.Local.Put
        0x94f5_5475 => 1, // System.Storage.Local.Delete
        0xf352_7607 => 2, // System.Storage.Local.Find
        0x31e8_5d92 => 2, // System.Storage.Get
        0x9ab8_30df => 3, // System.Storage.Find
        0x8418_3fe6 => 3, // System.Storage.Put
        0xedc5_582f => 2, // System.Storage.Delete
        // System.Crypto
        0x27b3_e756 => 2,          // System.Crypto.CheckSig
        0x3adc_d09e => usize::MAX, // System.Crypto.CheckMultisig
        // System.Iterator
        0x9ced_089c => 1, // System.Iterator.Next
        0x1dbf_54f3 => 1, // System.Iterator.Value
        _ => usize::MAX,
    }
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use super::{interop_hash, syscall_arg_count};

    const KNOWN_SYSCALLS: &[&str] = &[
        "System.Contract.Call",
        "System.Contract.CallNative",
        "System.Contract.Create",
        "System.Contract.Update",
        "System.Contract.NativeOnPersist",
        "System.Contract.NativePostPersist",
        "System.Contract.GetCallFlags",
        "System.Contract.CreateStandardAccount",
        "System.Contract.CreateMultisigAccount",
        "System.Runtime.CheckWitness",
        "System.Runtime.Notify",
        "System.Runtime.Log",
        "System.Runtime.GetNotifications",
        "System.Runtime.BurnGas",
        "System.Runtime.LoadScript",
        "System.Runtime.Platform",
        "System.Runtime.GetTrigger",
        "System.Runtime.GetTime",
        "System.Runtime.GetScriptContainer",
        "System.Runtime.GetExecutingScriptHash",
        "System.Runtime.GetCallingScriptHash",
        "System.Runtime.GetEntryScriptHash",
        "System.Runtime.GetInvocationCounter",
        "System.Runtime.GasLeft",
        "System.Runtime.GetAddressVersion",
        "System.Runtime.CurrentSigners",
        "System.Runtime.GetNetwork",
        "System.Runtime.GetRandom",
        "System.Storage.GetContext",
        "System.Storage.GetReadOnlyContext",
        "System.Storage.AsReadOnly",
        "System.Storage.Local.Get",
        "System.Storage.Local.Put",
        "System.Storage.Local.Delete",
        "System.Storage.Local.Find",
        "System.Storage.Get",
        "System.Storage.Find",
        "System.Storage.Put",
        "System.Storage.Delete",
        "System.Crypto.CheckSig",
        "System.Crypto.CheckMultisig",
        "System.Iterator.Next",
        "System.Iterator.Value",
    ];

    fn canonical_hash(name: &str) -> u32 {
        let digest = Sha256::digest(name.as_bytes());
        u32::from_le_bytes([digest[0], digest[1], digest[2], digest[3]])
    }

    #[test]
    fn known_syscall_hashes_match_sha256_prefix() {
        for name in KNOWN_SYSCALLS {
            assert_eq!(
                interop_hash(name),
                canonical_hash(name),
                "hash mismatch for {name}"
            );
        }
    }

    #[test]
    fn unknown_syscall_hashes_still_use_sha256_prefix() {
        let name = "System.Test.Unknown";
        assert_eq!(interop_hash(name), canonical_hash(name));
    }

    #[test]
    fn known_syscall_argument_counts_match_hashes() {
        assert_eq!(syscall_arg_count(interop_hash("System.Contract.Call")), 4);
        assert_eq!(
            syscall_arg_count(interop_hash("System.Contract.CallNative")),
            usize::MAX
        );
        assert_eq!(
            syscall_arg_count(interop_hash("System.Runtime.Platform")),
            0
        );
    }
}
