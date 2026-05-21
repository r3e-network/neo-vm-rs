//! Canonical NeoVM stack value types.

use alloc::{format, string::String, vec::Vec};
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

/// NeoVM `StackItemType.Boolean`.
pub const NEOVM_STACK_ITEM_TYPE_BOOLEAN: u8 = 0x20;
/// NeoVM `StackItemType.Integer`.
pub const NEOVM_STACK_ITEM_TYPE_INTEGER: u8 = 0x21;
/// NeoVM `StackItemType.ByteString`.
pub const NEOVM_STACK_ITEM_TYPE_BYTESTRING: u8 = 0x28;
/// NeoVM `StackItemType.Buffer`.
pub const NEOVM_STACK_ITEM_TYPE_BUFFER: u8 = 0x30;
/// NeoVM `StackItemType.Array`.
pub const NEOVM_STACK_ITEM_TYPE_ARRAY: u8 = 0x40;
/// NeoVM `StackItemType.Struct`.
pub const NEOVM_STACK_ITEM_TYPE_STRUCT: u8 = 0x41;
/// NeoVM `StackItemType.Map`.
pub const NEOVM_STACK_ITEM_TYPE_MAP: u8 = 0x48;

/// Normalize NeoVM StackItemType tags and compact runtime tags into one compact tag space.
#[must_use]
pub fn normalize_stack_item_type_tag(type_tag: u8) -> u8 {
    match type_tag {
        NEOVM_STACK_ITEM_TYPE_BOOLEAN => COMPACT_TAG_BOOLEAN,
        NEOVM_STACK_ITEM_TYPE_INTEGER => COMPACT_TAG_INTEGER,
        NEOVM_STACK_ITEM_TYPE_BYTESTRING => COMPACT_TAG_BYTESTRING,
        NEOVM_STACK_ITEM_TYPE_BUFFER => COMPACT_TAG_BUFFER,
        NEOVM_STACK_ITEM_TYPE_ARRAY => COMPACT_TAG_ARRAY,
        NEOVM_STACK_ITEM_TYPE_STRUCT => COMPACT_TAG_STRUCT,
        NEOVM_STACK_ITEM_TYPE_MAP => COMPACT_TAG_MAP,
        other => other,
    }
}

/// Return the shared default value for a compact runtime or NeoVM StackItemType tag.
#[must_use]
pub fn default_value_for_type_tag(type_tag: u8) -> StackValue {
    match normalize_stack_item_type_tag(type_tag) {
        COMPACT_TAG_BOOLEAN => StackValue::Boolean(false),
        COMPACT_TAG_INTEGER | COMPACT_TAG_BIG_INTEGER => StackValue::Integer(0),
        COMPACT_TAG_BYTESTRING => StackValue::ByteString(Vec::new()),
        COMPACT_TAG_BUFFER => StackValue::Buffer(Vec::new()),
        COMPACT_TAG_ARRAY => StackValue::Array(Vec::new()),
        COMPACT_TAG_STRUCT => StackValue::Struct(Vec::new()),
        COMPACT_TAG_MAP => StackValue::Map(Vec::new()),
        COMPACT_TAG_NULL => StackValue::Null,
        _ => StackValue::Null,
    }
}

/// Return the NeoVM `NEWARRAY_T` item default for a compact or NeoVM type tag.
#[must_use]
pub fn new_array_default_value_for_type_tag(type_tag: u8) -> StackValue {
    match normalize_stack_item_type_tag(type_tag) {
        COMPACT_TAG_INTEGER => StackValue::Integer(0),
        COMPACT_TAG_BYTESTRING => StackValue::ByteString(Vec::new()),
        _ => StackValue::Null,
    }
}

/// Pop a syscall-style byte argument from the top of a stack.
///
/// NeoVM byte syscalls accept ByteString and Buffer values as raw bytes. This
/// deliberately does not accept BigInteger, even though it has a byte
/// representation, because syscall argument validation is type based.
pub fn pop_byte_arg(stack: &mut Vec<StackValue>, context: &str) -> Result<Vec<u8>, String> {
    match stack.pop() {
        Some(StackValue::ByteString(bytes)) | Some(StackValue::Buffer(bytes)) => Ok(bytes),
        Some(other) => Err(format!(
            "{context} expects ByteString or Buffer, got {other:?}"
        )),
        None => Err(format!("{context} expects one stack argument")),
    }
}

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

    #[test]
    fn neovm_type_tags_normalize_to_compact_runtime_tags() {
        assert_eq!(
            super::normalize_stack_item_type_tag(0x20),
            super::COMPACT_TAG_BOOLEAN
        );
        assert_eq!(
            super::normalize_stack_item_type_tag(0x21),
            super::COMPACT_TAG_INTEGER
        );
        assert_eq!(
            super::normalize_stack_item_type_tag(0x28),
            super::COMPACT_TAG_BYTESTRING
        );
        assert_eq!(
            super::normalize_stack_item_type_tag(0x30),
            super::COMPACT_TAG_BUFFER
        );
        assert_eq!(
            super::normalize_stack_item_type_tag(0x40),
            super::COMPACT_TAG_ARRAY
        );
        assert_eq!(
            super::normalize_stack_item_type_tag(0x41),
            super::COMPACT_TAG_STRUCT
        );
        assert_eq!(
            super::normalize_stack_item_type_tag(0x48),
            super::COMPACT_TAG_MAP
        );
        assert_eq!(
            super::normalize_stack_item_type_tag(super::COMPACT_TAG_POINTER),
            super::COMPACT_TAG_POINTER
        );
    }

    #[test]
    fn default_values_follow_shared_type_tag_rules() {
        assert_eq!(
            super::default_value_for_type_tag(0x20),
            StackValue::Boolean(false)
        );
        assert_eq!(
            super::default_value_for_type_tag(0x21),
            StackValue::Integer(0)
        );
        assert_eq!(
            super::default_value_for_type_tag(super::COMPACT_TAG_BIG_INTEGER),
            StackValue::Integer(0)
        );
        assert_eq!(
            super::default_value_for_type_tag(0x28),
            StackValue::ByteString(Vec::new())
        );
        assert_eq!(
            super::default_value_for_type_tag(0x30),
            StackValue::Buffer(Vec::new())
        );
        assert_eq!(
            super::default_value_for_type_tag(0x40),
            StackValue::Array(Vec::new())
        );
        assert_eq!(
            super::default_value_for_type_tag(0x41),
            StackValue::Struct(Vec::new())
        );
        assert_eq!(
            super::default_value_for_type_tag(0x48),
            StackValue::Map(Vec::new())
        );
        assert_eq!(
            super::default_value_for_type_tag(super::COMPACT_TAG_NULL),
            StackValue::Null
        );
        assert_eq!(super::default_value_for_type_tag(0xff), StackValue::Null);
    }

    #[test]
    fn new_array_default_values_follow_neovm_rules() {
        assert_eq!(
            super::new_array_default_value_for_type_tag(0x21),
            StackValue::Integer(0)
        );
        assert_eq!(
            super::new_array_default_value_for_type_tag(super::COMPACT_TAG_INTEGER),
            StackValue::Integer(0)
        );
        assert_eq!(
            super::new_array_default_value_for_type_tag(0x28),
            StackValue::ByteString(Vec::new())
        );
        assert_eq!(
            super::new_array_default_value_for_type_tag(super::COMPACT_TAG_BYTESTRING),
            StackValue::ByteString(Vec::new())
        );
        assert_eq!(
            super::new_array_default_value_for_type_tag(0x20),
            StackValue::Null
        );
        assert_eq!(
            super::new_array_default_value_for_type_tag(0xff),
            StackValue::Null
        );
    }

    #[test]
    fn byte_arg_pop_accepts_only_bytestring_and_buffer() {
        let mut byte_string_stack = vec![
            StackValue::Integer(7),
            StackValue::ByteString(b"neo".to_vec()),
        ];
        assert_eq!(
            super::pop_byte_arg(&mut byte_string_stack, "System.Crypto.SHA256"),
            Ok(b"neo".to_vec())
        );
        assert_eq!(byte_string_stack, vec![StackValue::Integer(7)]);

        let mut buffer_stack = vec![StackValue::Buffer(b"n4".to_vec())];
        assert_eq!(
            super::pop_byte_arg(&mut buffer_stack, "System.Crypto.SHA256"),
            Ok(b"n4".to_vec())
        );

        let mut integer_stack = vec![StackValue::Integer(1)];
        let error = super::pop_byte_arg(&mut integer_stack, "System.Crypto.SHA256")
            .expect_err("integer is not a byte syscall argument");
        assert!(error.contains("System.Crypto.SHA256 expects ByteString or Buffer"));

        let mut empty_stack = Vec::new();
        let error = super::pop_byte_arg(&mut empty_stack, "System.Crypto.SHA256")
            .expect_err("empty stack should report missing argument");
        assert_eq!(error, "System.Crypto.SHA256 expects one stack argument");
    }
}
