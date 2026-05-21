//! Canonical NeoVM stack value types.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// Compact integer tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_INTEGER: u8 = 0;
/// Compact boolean tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_BOOLEAN: u8 = 1;
/// Compact byte string tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_BYTESTRING: u8 = 2;
/// Compact big integer tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_BIG_INTEGER: u8 = 3;
/// Compact array tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_ARRAY: u8 = 4;
/// Compact struct tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_STRUCT: u8 = 5;
/// Compact map tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_MAP: u8 = 6;
/// Compact null tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_NULL: u8 = 7;
/// Compact interop handle tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_INTEROP: u8 = 8;
/// Compact iterator handle tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_ITERATOR: u8 = 9;
/// Compact buffer tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_BUFFER: u8 = 10;
/// Compact pointer tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_POINTER: u8 = 11;

/// Encode an integer using NeoVM's minimal little-endian two's-complement form.
#[must_use]
pub fn encode_integer(value: i64) -> Vec<u8> {
    if value == 0 {
        return Vec::new();
    }

    let mut bytes = value.to_le_bytes().to_vec();
    if value > 0 {
        while bytes.len() > 1 && bytes.last() == Some(&0) {
            if bytes[bytes.len() - 2] & 0x80 != 0 {
                break;
            }
            bytes.pop();
        }
    } else {
        while bytes.len() > 1 && bytes.last() == Some(&0xff) {
            if bytes[bytes.len() - 2] & 0x80 == 0 {
                break;
            }
            bytes.pop();
        }
    }

    bytes
}

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
    /// Return the compact runtime tag for this stack value.
    #[must_use]
    pub fn compact_type_tag(&self) -> u8 {
        match self {
            Self::Integer(_) => COMPACT_TAG_INTEGER,
            Self::Boolean(_) => COMPACT_TAG_BOOLEAN,
            Self::ByteString(_) => COMPACT_TAG_BYTESTRING,
            Self::BigInteger(_) => COMPACT_TAG_BIG_INTEGER,
            Self::Array(_) => COMPACT_TAG_ARRAY,
            Self::Struct(_) => COMPACT_TAG_STRUCT,
            Self::Map(_) => COMPACT_TAG_MAP,
            Self::Null => COMPACT_TAG_NULL,
            Self::Interop(_) => COMPACT_TAG_INTEROP,
            Self::Iterator(_) => COMPACT_TAG_ITERATOR,
            Self::Buffer(_) => COMPACT_TAG_BUFFER,
            Self::Pointer(_) => COMPACT_TAG_POINTER,
        }
    }

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

    /// Convert this value into bytes for NeoVM ByteString-compatible paths.
    ///
    /// This is the shared rule used by runtimes that need byte content without
    /// owning a private StackValue conversion table.
    #[must_use]
    pub fn to_byte_string_bytes(&self) -> Option<Vec<u8>> {
        match self {
            Self::ByteString(bytes) | Self::Buffer(bytes) | Self::BigInteger(bytes) => {
                Some(bytes.clone())
            }
            Self::Integer(value) => Some(encode_integer(*value)),
            Self::Boolean(value) => Some(alloc::vec![u8::from(*value)]),
            Self::Null => Some(Vec::new()),
            Self::Array(_)
            | Self::Struct(_)
            | Self::Map(_)
            | Self::Interop(_)
            | Self::Iterator(_)
            | Self::Pointer(_) => None,
        }
    }

    /// Convert this value as NeoVM `CONVERT` would for a ByteString target.
    #[must_use]
    pub fn convert_to_byte_string_value(&self) -> Option<Self> {
        if matches!(self, Self::Null) {
            return Some(Self::Null);
        }
        self.to_byte_string_bytes().map(Self::ByteString)
    }

    /// Convert this value as NeoVM `CONVERT` would for a Buffer target.
    #[must_use]
    pub fn convert_to_buffer_value(&self) -> Option<Self> {
        if matches!(self, Self::Null) {
            return Some(Self::Null);
        }
        self.to_byte_string_bytes().map(Self::Buffer)
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

    #[test]
    fn integer_encoding_uses_minimal_little_endian_twos_complement() {
        assert_eq!(super::encode_integer(0), vec![]);
        assert_eq!(super::encode_integer(1), vec![0x01]);
        assert_eq!(super::encode_integer(127), vec![0x7f]);
        assert_eq!(super::encode_integer(128), vec![0x80, 0x00]);
        assert_eq!(super::encode_integer(-1), vec![0xff]);
        assert_eq!(super::encode_integer(-129), vec![0x7f, 0xff]);
    }

    #[test]
    fn stack_values_expose_stable_runtime_type_tags() {
        assert_eq!(
            StackValue::Integer(1).compact_type_tag(),
            super::COMPACT_TAG_INTEGER
        );
        assert_eq!(
            StackValue::Boolean(true).compact_type_tag(),
            super::COMPACT_TAG_BOOLEAN
        );
        assert_eq!(
            StackValue::ByteString(vec![]).compact_type_tag(),
            super::COMPACT_TAG_BYTESTRING
        );
        assert_eq!(
            StackValue::BigInteger(vec![]).compact_type_tag(),
            super::COMPACT_TAG_BIG_INTEGER
        );
        assert_eq!(
            StackValue::Array(vec![]).compact_type_tag(),
            super::COMPACT_TAG_ARRAY
        );
        assert_eq!(
            StackValue::Struct(vec![]).compact_type_tag(),
            super::COMPACT_TAG_STRUCT
        );
        assert_eq!(
            StackValue::Map(vec![]).compact_type_tag(),
            super::COMPACT_TAG_MAP
        );
        assert_eq!(StackValue::Null.compact_type_tag(), super::COMPACT_TAG_NULL);
        assert_eq!(
            StackValue::Interop(1).compact_type_tag(),
            super::COMPACT_TAG_INTEROP
        );
        assert_eq!(
            StackValue::Iterator(1).compact_type_tag(),
            super::COMPACT_TAG_ITERATOR
        );
        assert_eq!(
            StackValue::Buffer(vec![]).compact_type_tag(),
            super::COMPACT_TAG_BUFFER
        );
        assert_eq!(
            StackValue::Pointer(0).compact_type_tag(),
            super::COMPACT_TAG_POINTER
        );
    }

    #[test]
    fn byte_string_conversion_bytes_follow_shared_stack_value_rules() {
        assert_eq!(
            StackValue::ByteString(b"neo".to_vec()).to_byte_string_bytes(),
            Some(b"neo".to_vec())
        );
        assert_eq!(
            StackValue::Buffer(b"n4".to_vec()).to_byte_string_bytes(),
            Some(b"n4".to_vec())
        );
        assert_eq!(
            StackValue::Integer(128).to_byte_string_bytes(),
            Some(vec![0x80, 0x00])
        );
        assert_eq!(
            StackValue::Boolean(true).to_byte_string_bytes(),
            Some(vec![1])
        );
        assert_eq!(
            StackValue::BigInteger(vec![0xff, 0x00]).to_byte_string_bytes(),
            Some(vec![0xff, 0x00])
        );
        assert_eq!(StackValue::Null.to_byte_string_bytes(), Some(vec![]));
        assert_eq!(StackValue::Array(vec![]).to_byte_string_bytes(), None);
    }

    #[test]
    fn conversion_values_follow_neovm_primitive_rules() {
        assert_eq!(
            StackValue::Integer(128).convert_to_byte_string_value(),
            Some(StackValue::ByteString(vec![0x80, 0x00]))
        );
        assert_eq!(
            StackValue::Boolean(false).convert_to_buffer_value(),
            Some(StackValue::Buffer(vec![0]))
        );
        assert_eq!(
            StackValue::BigInteger(vec![0xff, 0x00]).convert_to_buffer_value(),
            Some(StackValue::Buffer(vec![0xff, 0x00]))
        );
        assert_eq!(
            StackValue::Null.convert_to_byte_string_value(),
            Some(StackValue::Null)
        );
        assert_eq!(
            StackValue::Null.convert_to_buffer_value(),
            Some(StackValue::Null)
        );
        assert_eq!(StackValue::Array(vec![]).convert_to_buffer_value(), None);
    }
}
