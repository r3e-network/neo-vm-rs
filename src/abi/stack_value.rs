//! Canonical NeoVM stack value types.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// NeoVM stack value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StackValue {
    /// 64-bit signed integer for compact ABI paths.
    Integer(i64),
    /// Arbitrary-precision integer encoded as little-endian two's complement.
    BigInteger(Vec<u8>),
    /// Immutable byte string.
    ByteString(Vec<u8>),
    /// Mutable byte buffer.
    Buffer(Vec<u8>),
    /// Boolean value.
    Boolean(bool),
    /// Ordered array.
    Array(Vec<StackValue>),
    /// Ordered struct.
    Struct(Vec<StackValue>),
    /// Key-value map.
    Map(Vec<(StackValue, StackValue)>),
    /// Host interop handle.
    Interop(u64),
    /// Iterator handle.
    Iterator(u64),
    /// Null value.
    Null,
    /// VM pointer.
    Pointer(i64),
}

impl StackValue {
    /// Convert this value to a NeoVM boolean.
    #[must_use]
    pub fn to_bool(&self) -> bool {
        match self {
            Self::Null => false,
            Self::Boolean(value) => *value,
            Self::Integer(value) => *value != 0,
            Self::BigInteger(bytes) | Self::ByteString(bytes) | Self::Buffer(bytes) => {
                bytes.iter().any(|byte| *byte != 0)
            }
            Self::Array(items) | Self::Struct(items) => !items.is_empty(),
            Self::Map(items) => !items.is_empty(),
            Self::Interop(_) | Self::Iterator(_) | Self::Pointer(_) => true,
        }
    }

    /// Convert this value to a bounded integer when possible.
    ///
    /// Byte values follow NeoVM's little-endian two's complement convention.
    #[must_use]
    pub fn to_i128(&self) -> Option<i128> {
        match self {
            Self::Integer(value) => Some(i128::from(*value)),
            Self::Boolean(value) => Some(i128::from(*value as i8)),
            Self::BigInteger(bytes) | Self::ByteString(bytes) | Self::Buffer(bytes) => {
                little_endian_twos_complement_i128(bytes)
            }
            _ => None,
        }
    }

    /// Borrow byte content from byte-like values.
    #[must_use]
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::BigInteger(bytes) | Self::ByteString(bytes) | Self::Buffer(bytes) => Some(bytes),
            _ => None,
        }
    }
}

fn little_endian_twos_complement_i128(bytes: &[u8]) -> Option<i128> {
    if bytes.is_empty() {
        return Some(0);
    }
    if bytes.len() > 16 {
        return None;
    }

    let mut raw = 0u128;
    for (index, byte) in bytes.iter().enumerate() {
        raw |= u128::from(*byte) << (index * 8);
    }

    if bytes.len() < 16 && bytes[bytes.len() - 1] & 0x80 != 0 {
        raw |= u128::MAX << (bytes.len() * 8);
    }

    Some(i128::from_le_bytes(raw.to_le_bytes()))
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::StackValue;

    #[test]
    fn bytes_use_little_endian_twos_complement() {
        assert_eq!(StackValue::ByteString(vec![]).to_i128(), Some(0));
        assert_eq!(StackValue::ByteString(vec![0x7f]).to_i128(), Some(127));
        assert_eq!(StackValue::ByteString(vec![0xff]).to_i128(), Some(-1));
        assert_eq!(
            StackValue::ByteString(vec![0x00, 0x80]).to_i128(),
            Some(-32768)
        );
        assert_eq!(StackValue::ByteString(vec![0xff; 16]).to_i128(), Some(-1));
        assert_eq!(StackValue::ByteString(vec![0x00; 17]).to_i128(), None);
    }
}
