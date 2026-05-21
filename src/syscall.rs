//! Shared NeoVM syscall helpers.

use sha2::{Digest, Sha256};

/// Returns the first four bytes of SHA-256(name) as a little-endian syscall id.
#[must_use]
pub fn interop_hash(name: &str) -> u32 {
    let digest = Sha256::digest(name.as_bytes());
    u32::from_le_bytes([digest[0], digest[1], digest[2], digest[3]])
}

/// Returns the number of stack arguments consumed by a known NeoVM syscall.
///
/// Unknown or count-dependent syscalls return `usize::MAX`, which tells callers
/// to pass the full stack through the host boundary.
#[must_use]
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
    use super::{interop_hash, syscall_arg_count};

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
