//! Canonical NeoVM stack value types.

use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};

use crate::semantics::numeric;

mod ops;
mod tags;

pub use ops::*;
pub use tags::*;

/// C# Neo.VM stack item type tags.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StackItemType {
    /// Represents any type.
    Any = NEOVM_STACK_ITEM_TYPE_ANY,
    /// Represents a code pointer.
    Pointer = NEOVM_STACK_ITEM_TYPE_POINTER,
    /// Represents a boolean value.
    Boolean = NEOVM_STACK_ITEM_TYPE_BOOLEAN,
    /// Represents an integer value.
    Integer = NEOVM_STACK_ITEM_TYPE_INTEGER,
    /// Represents an immutable byte sequence.
    ByteString = NEOVM_STACK_ITEM_TYPE_BYTESTRING,
    /// Represents a mutable byte sequence.
    Buffer = NEOVM_STACK_ITEM_TYPE_BUFFER,
    /// Represents an array.
    Array = NEOVM_STACK_ITEM_TYPE_ARRAY,
    /// Represents a structure.
    Struct = NEOVM_STACK_ITEM_TYPE_STRUCT,
    /// Represents an ordered key-value map.
    Map = NEOVM_STACK_ITEM_TYPE_MAP,
    /// Represents a host interop interface.
    InteropInterface = NEOVM_STACK_ITEM_TYPE_INTEROP_INTERFACE,
}

impl StackItemType {
    /// Converts a C# Neo.VM stack item type byte into a typed tag.
    #[inline]
    #[must_use]
    pub const fn from_byte(value: u8) -> Option<Self> {
        match value {
            NEOVM_STACK_ITEM_TYPE_ANY => Some(Self::Any),
            NEOVM_STACK_ITEM_TYPE_POINTER => Some(Self::Pointer),
            NEOVM_STACK_ITEM_TYPE_BOOLEAN => Some(Self::Boolean),
            NEOVM_STACK_ITEM_TYPE_INTEGER => Some(Self::Integer),
            NEOVM_STACK_ITEM_TYPE_BYTESTRING => Some(Self::ByteString),
            NEOVM_STACK_ITEM_TYPE_BUFFER => Some(Self::Buffer),
            NEOVM_STACK_ITEM_TYPE_ARRAY => Some(Self::Array),
            NEOVM_STACK_ITEM_TYPE_STRUCT => Some(Self::Struct),
            NEOVM_STACK_ITEM_TYPE_MAP => Some(Self::Map),
            NEOVM_STACK_ITEM_TYPE_INTEROP_INTERFACE => Some(Self::InteropInterface),
            _ => None,
        }
    }

    /// Returns the C# Neo.VM stack item type byte.
    #[inline]
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::Any => NEOVM_STACK_ITEM_TYPE_ANY,
            Self::Pointer => NEOVM_STACK_ITEM_TYPE_POINTER,
            Self::Boolean => NEOVM_STACK_ITEM_TYPE_BOOLEAN,
            Self::Integer => NEOVM_STACK_ITEM_TYPE_INTEGER,
            Self::ByteString => NEOVM_STACK_ITEM_TYPE_BYTESTRING,
            Self::Buffer => NEOVM_STACK_ITEM_TYPE_BUFFER,
            Self::Array => NEOVM_STACK_ITEM_TYPE_ARRAY,
            Self::Struct => NEOVM_STACK_ITEM_TYPE_STRUCT,
            Self::Map => NEOVM_STACK_ITEM_TYPE_MAP,
            Self::InteropInterface => NEOVM_STACK_ITEM_TYPE_INTEROP_INTERFACE,
        }
    }

    /// Returns the canonical NeoVM stack item type name.
    #[inline]
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Any => "Any",
            Self::Pointer => "Pointer",
            Self::Boolean => "Boolean",
            Self::Integer => "Integer",
            Self::ByteString => "ByteString",
            Self::Buffer => "Buffer",
            Self::Array => "Array",
            Self::Struct => "Struct",
            Self::Map => "Map",
            Self::InteropInterface => "InteropInterface",
        }
    }
}

/// NeoVM stack value.
///
/// Compound variants (Array, Struct, Map, Buffer) carry a unique `u64` ID
/// that enables reference-equality semantics matching C# `StackItem`.
/// Use [`crate::next_stack_item_id`] to generate fresh IDs.
///
/// Serialization skips the runtime IDs for wire-format compatibility
/// with the original ID-less layout.
///
/// `PartialEq` compares compound types by ID (reference equality),
/// matching C# `ReferenceEquals` semantics for Array/Struct/Map/Buffer.
#[derive(Debug, Clone, Eq)]
pub enum StackValue {
    /// 64-bit signed integer for compact ABI paths.
    Integer(i64),
    /// Arbitrary-precision integer encoded as little-endian two's complement.
    BigInteger(Vec<u8>),
    /// Immutable byte string.
    ByteString(Vec<u8>),
    /// Mutable byte buffer.
    Buffer(u64, Vec<u8>),
    /// Boolean value.
    Boolean(bool),
    /// Ordered array.
    Array(u64, Vec<StackValue>),
    /// Ordered struct.
    Struct(u64, Vec<StackValue>),
    /// Key-value map.
    Map(u64, Vec<(StackValue, StackValue)>),
    /// Host interop handle.
    Interop(u64),
    /// Iterator handle.
    Iterator(u64),
    /// Null value.
    Null,
    /// VM pointer.
    Pointer(i64),
}

// ── Custom serialization: skip runtime IDs for wire-format compatibility ──

impl Serialize for StackValue {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Integer(v) => s.serialize_newtype_variant("StackValue", 0, "Integer", v),
            Self::BigInteger(v) => s.serialize_newtype_variant("StackValue", 1, "BigInteger", v),
            Self::ByteString(v) => s.serialize_newtype_variant("StackValue", 2, "ByteString", v),
            Self::Buffer(_, v) => s.serialize_newtype_variant("StackValue", 3, "Buffer", v),
            Self::Boolean(v) => s.serialize_newtype_variant("StackValue", 4, "Boolean", v),
            Self::Array(_, items) => s.serialize_newtype_variant("StackValue", 5, "Array", items),
            Self::Struct(_, items) => s.serialize_newtype_variant("StackValue", 6, "Struct", items),
            Self::Map(_, entries) => s.serialize_newtype_variant("StackValue", 7, "Map", entries),
            Self::Interop(v) => s.serialize_newtype_variant("StackValue", 8, "Interop", v),
            Self::Iterator(v) => s.serialize_newtype_variant("StackValue", 9, "Iterator", v),
            Self::Null => s.serialize_unit_variant("StackValue", 10, "Null"),
            Self::Pointer(v) => s.serialize_newtype_variant("StackValue", 11, "Pointer", v),
        }
    }
}

impl<'de> Deserialize<'de> for StackValue {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use core::fmt;
        use serde::de::{self, MapAccess, SeqAccess, Visitor};

        struct SvVisitor;

        impl<'de> Visitor<'de> for SvVisitor {
            type Value = StackValue;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a NeoVM StackValue")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<StackValue, E> {
                match v {
                    "Null" => Ok(StackValue::Null),
                    _ => Err(de::Error::unknown_variant(v, &["Null"])),
                }
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<StackValue, M::Error> {
                let key: Option<String> = map.next_key()?;
                let key = match key {
                    Some(k) => k,
                    None => return Err(de::Error::custom("expected StackValue variant key")),
                };
                match key.as_str() {
                    "Integer" => Ok(StackValue::Integer(map.next_value()?)),
                    "BigInteger" => Ok(StackValue::BigInteger(map.next_value()?)),
                    "ByteString" => Ok(StackValue::ByteString(map.next_value()?)),
                    "Buffer" => Ok(StackValue::Buffer(
                        crate::next_stack_item_id(),
                        map.next_value()?,
                    )),
                    "Boolean" => Ok(StackValue::Boolean(map.next_value()?)),
                    "Array" => Ok(StackValue::Array(
                        crate::next_stack_item_id(),
                        map.next_value()?,
                    )),
                    "Struct" => Ok(StackValue::Struct(
                        crate::next_stack_item_id(),
                        map.next_value()?,
                    )),
                    "Map" => Ok(StackValue::Map(
                        crate::next_stack_item_id(),
                        map.next_value()?,
                    )),
                    "Interop" => Ok(StackValue::Interop(map.next_value()?)),
                    "Iterator" => Ok(StackValue::Iterator(map.next_value()?)),
                    "Null" => Ok(StackValue::Null),
                    "Pointer" => Ok(StackValue::Pointer(map.next_value()?)),
                    _ => Err(de::Error::unknown_variant(
                        &key,
                        &[
                            "Integer",
                            "BigInteger",
                            "ByteString",
                            "Buffer",
                            "Boolean",
                            "Array",
                            "Struct",
                            "Map",
                            "Interop",
                            "Iterator",
                            "Null",
                            "Pointer",
                        ],
                    )),
                }
            }

            fn visit_seq<A: SeqAccess<'de>>(self, _seq: A) -> Result<StackValue, A::Error> {
                Err(de::Error::custom(
                    "StackValue is not a JSON array; use {\"Variant\": value} format",
                ))
            }
        }

        d.deserialize_any(SvVisitor)
    }
}

/// Reference equality for compound types (Array, Struct, Map, Buffer):
/// compounds compare by ID only (matching C# `ReferenceEquals`).
/// Primitives compare by value.
impl PartialEq for StackValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Integer(a), Self::Integer(b)) => a == b,
            (Self::BigInteger(a), Self::BigInteger(b)) => a == b,
            (Self::ByteString(a), Self::ByteString(b)) => a == b,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Null, Self::Null) => true,
            (Self::Pointer(a), Self::Pointer(b)) => a == b,
            (Self::Interop(a), Self::Interop(b)) => a == b,
            (Self::Iterator(a), Self::Iterator(b)) => a == b,
            // Compound types: reference equality by ID
            (Self::Buffer(id_a, _), Self::Buffer(id_b, _)) => id_a == id_b,
            (Self::Array(id_a, _), Self::Array(id_b, _)) => id_a == id_b,
            (Self::Struct(id_a, _), Self::Struct(id_b, _)) => id_a == id_b,
            (Self::Map(id_a, _), Self::Map(id_b, _)) => id_a == id_b,
            _ => false,
        }
    }
}

impl StackValue {
    /// Deep structural equality that compares by value, ignoring compound IDs.
    ///
    /// This is used for test assertions and conformance checking where
    /// two independently constructed values should be considered equal
    /// if they have the same structure and data.
    #[must_use]
    pub fn structural_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Integer(a), Self::Integer(b)) => a == b,
            (Self::BigInteger(a), Self::BigInteger(b)) => a == b,
            (Self::ByteString(a), Self::ByteString(b)) => a == b,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Null, Self::Null) => true,
            (Self::Pointer(a), Self::Pointer(b)) => a == b,
            (Self::Interop(a), Self::Interop(b)) => a == b,
            (Self::Iterator(a), Self::Iterator(b)) => a == b,
            (Self::Buffer(_, a), Self::Buffer(_, b)) => a == b,
            (Self::Array(_, a), Self::Array(_, b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.structural_eq(y))
            }
            (Self::Struct(_, a), Self::Struct(_, b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.structural_eq(y))
            }
            (Self::Map(_, a), Self::Map(_, b)) => {
                a.len() == b.len()
                    && a.iter()
                        .zip(b)
                        .all(|(x, y)| x.0.structural_eq(&y.0) && x.1.structural_eq(&y.1))
            }
            _ => false,
        }
    }

    /// Return the compact runtime tag for this stack value.
    #[must_use]
    pub fn compact_type_tag(&self) -> u8 {
        match self {
            Self::Integer(_) => COMPACT_TAG_INTEGER,
            Self::Boolean(_) => COMPACT_TAG_BOOLEAN,
            Self::ByteString(_) => COMPACT_TAG_BYTESTRING,
            Self::BigInteger(_) => COMPACT_TAG_BIG_INTEGER,
            Self::Array(_, _) => COMPACT_TAG_ARRAY,
            Self::Struct(_, _) => COMPACT_TAG_STRUCT,
            Self::Map(_, _) => COMPACT_TAG_MAP,
            Self::Null => COMPACT_TAG_NULL,
            Self::Interop(_) => COMPACT_TAG_INTEROP,
            Self::Iterator(_) => COMPACT_TAG_ITERATOR,
            Self::Buffer(_, _) => COMPACT_TAG_BUFFER,
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
            Self::BigInteger(bytes) | Self::ByteString(bytes) => {
                bytes.iter().any(|byte| *byte != 0)
            }
            Self::Buffer(_, _)
            | Self::Array(_, _)
            | Self::Struct(_, _)
            | Self::Map(_, _)
            | Self::Interop(_)
            | Self::Iterator(_)
            | Self::Pointer(_) => true,
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
            Self::BigInteger(bytes) | Self::ByteString(bytes) | Self::Buffer(_, bytes) => {
                numeric::decode_signed_le_bytes_i128(bytes).ok()
            }
            _ => None,
        }
    }

    /// Borrow byte content from byte-like values.
    #[must_use]
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::BigInteger(bytes) | Self::ByteString(bytes) | Self::Buffer(_, bytes) => {
                Some(bytes)
            }
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
            Self::ByteString(bytes) | Self::Buffer(_, bytes) | Self::BigInteger(bytes) => {
                Some(bytes.clone())
            }
            Self::Integer(value) => Some(encode_integer(*value)),
            Self::Boolean(value) => Some(alloc::vec![u8::from(*value)]),
            Self::Null => Some(Vec::new()),
            Self::Array(_, _)
            | Self::Struct(_, _)
            | Self::Map(_, _)
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
        self.to_byte_string_bytes()
            .map(|b| Self::Buffer(crate::next_stack_item_id(), b))
    }
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};

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
        assert_eq!(super::encode_integer(0), Vec::<u8>::new());
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
            StackValue::Array(0, vec![]).compact_type_tag(),
            super::COMPACT_TAG_ARRAY
        );
        assert_eq!(
            StackValue::Struct(0, vec![]).compact_type_tag(),
            super::COMPACT_TAG_STRUCT
        );
        assert_eq!(
            StackValue::Map(0, vec![]).compact_type_tag(),
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
            StackValue::Buffer(0, vec![]).compact_type_tag(),
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
            StackValue::Buffer(0, b"n4".to_vec()).to_byte_string_bytes(),
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
        assert_eq!(StackValue::Array(0, vec![]).to_byte_string_bytes(), None);
    }

    #[test]
    fn stack_value_as_bigint_follows_shared_integer_rules() {
        use num_bigint::BigInt;

        assert_eq!(
            super::stack_value_as_bigint(&StackValue::Integer(i64::MIN)).unwrap(),
            BigInt::from(i64::MIN)
        );
        assert_eq!(
            super::stack_value_as_bigint(&StackValue::Boolean(true)).unwrap(),
            BigInt::from(1)
        );
        assert_eq!(
            super::stack_value_as_bigint(&StackValue::ByteString(vec![0xff])).unwrap(),
            BigInt::from(-1)
        );
        assert_eq!(
            super::stack_value_as_bigint(&StackValue::Buffer(0, vec![0x00, 0x80])).unwrap(),
            BigInt::from(-32768)
        );
        assert_eq!(
            super::decode_integer_bytes(&[0xff, 0xff]).unwrap(),
            BigInt::from(-1)
        );
        assert!(super::stack_value_as_bigint(&StackValue::ByteString(vec![0; 33])).is_err());
        assert!(super::stack_value_as_bigint(&StackValue::Null).is_err());
    }

    #[test]
    fn conversion_values_follow_neovm_primitive_rules() {
        assert_eq!(
            StackValue::Integer(128).convert_to_byte_string_value(),
            Some(StackValue::ByteString(vec![0x80, 0x00]))
        );
        {
            let result = StackValue::Boolean(false)
                .convert_to_buffer_value()
                .unwrap();
            let expected = StackValue::Buffer(0, vec![0]);
            assert!(
                result.structural_eq(&expected),
                "expected {:?} but got {:?}",
                expected,
                result
            );
        }
        {
            let result = StackValue::BigInteger(vec![0xff, 0x00])
                .convert_to_buffer_value()
                .unwrap();
            let expected = StackValue::Buffer(0, vec![0xff, 0x00]);
            assert!(
                result.structural_eq(&expected),
                "expected {:?} but got {:?}",
                expected,
                result
            );
        }
        assert_eq!(
            StackValue::Null.convert_to_byte_string_value(),
            Some(StackValue::Null)
        );
        assert_eq!(
            StackValue::Null.convert_to_buffer_value(),
            Some(StackValue::Null)
        );
        assert_eq!(StackValue::Array(0, vec![]).convert_to_buffer_value(), None);
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
    fn stack_item_type_names_match_neo_vm() {
        assert_eq!(super::StackItemType::Any.name(), "Any");
        assert_eq!(super::StackItemType::Pointer.name(), "Pointer");
        assert_eq!(super::StackItemType::Boolean.name(), "Boolean");
        assert_eq!(super::StackItemType::Integer.name(), "Integer");
        assert_eq!(super::StackItemType::ByteString.name(), "ByteString");
        assert_eq!(super::StackItemType::Buffer.name(), "Buffer");
        assert_eq!(super::StackItemType::Array.name(), "Array");
        assert_eq!(super::StackItemType::Struct.name(), "Struct");
        assert_eq!(super::StackItemType::Map.name(), "Map");
        assert_eq!(
            super::StackItemType::InteropInterface.name(),
            "InteropInterface"
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
        {
            let result = super::default_value_for_type_tag(0x30);
            let expected = StackValue::Buffer(0, Vec::new());
            assert!(
                result.structural_eq(&expected),
                "expected {:?} but got {:?}",
                expected,
                result
            );
        }
        {
            let result = super::default_value_for_type_tag(0x40);
            let expected = StackValue::Array(0, Vec::new());
            assert!(
                result.structural_eq(&expected),
                "expected {:?} but got {:?}",
                expected,
                result
            );
        }
        {
            let result = super::default_value_for_type_tag(0x41);
            let expected = StackValue::Struct(0, Vec::new());
            assert!(
                result.structural_eq(&expected),
                "expected {:?} but got {:?}",
                expected,
                result
            );
        }
        {
            let result = super::default_value_for_type_tag(0x48);
            let expected = StackValue::Map(0, Vec::new());
            assert!(
                result.structural_eq(&expected),
                "expected {:?} but got {:?}",
                expected,
                result
            );
        }
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
        assert_eq!(
            super::new_array_default_value_for_neovm_type_tag(super::NEOVM_STACK_ITEM_TYPE_ANY),
            StackValue::Null
        );
        assert_eq!(
            super::new_array_default_value_for_neovm_type_tag(super::NEOVM_STACK_ITEM_TYPE_INTEGER),
            StackValue::Integer(0)
        );
        assert_eq!(
            super::new_array_default_value_for_neovm_type_tag(
                super::NEOVM_STACK_ITEM_TYPE_BYTESTRING
            ),
            StackValue::ByteString(Vec::new())
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

        let mut buffer_stack = vec![StackValue::Buffer(0, b"n4".to_vec())];
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

    #[test]
    fn byte_sequence_helpers_accept_only_bytestring_and_buffer() {
        let byte_string = StackValue::ByteString(b"neo".to_vec());
        let buffer = StackValue::Buffer(0, b"n4".to_vec());

        assert_eq!(super::byte_sequence_bytes(&byte_string), Some(&b"neo"[..]));
        assert_eq!(super::byte_sequence_bytes(&buffer), Some(&b"n4"[..]));
        assert_eq!(super::byte_sequence_len(&byte_string), Some(3));
        assert_eq!(super::byte_sequence_len(&buffer), Some(2));
        assert_eq!(
            super::byte_sequence_bytes(&StackValue::BigInteger(vec![1])),
            None
        );
        assert_eq!(super::byte_sequence_len(&StackValue::Integer(1)), None);
    }

    #[test]
    fn stack_value_extractors_cover_native_contract_result_shapes() {
        assert_eq!(
            super::stack_value_as_i64(&StackValue::Integer(42)),
            Some(42)
        );
        assert_eq!(
            super::stack_value_as_i64(&StackValue::BigInteger(vec![0xff, 0x00])),
            Some(255)
        );
        assert_eq!(
            super::stack_value_as_i64(&StackValue::BigInteger(vec![0xff; 16])),
            Some(-1)
        );
        assert_eq!(
            super::stack_value_as_i64(&StackValue::BigInteger(vec![
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80,
            ])),
            Some(i64::MIN)
        );
        assert_eq!(
            super::stack_value_as_i64(&StackValue::BigInteger(vec![
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00,
            ])),
            None
        );
        assert_eq!(super::stack_value_as_i64(&StackValue::Null), None);

        assert_eq!(
            super::stack_value_as_bool(&StackValue::Boolean(true)),
            Some(true)
        );
        assert_eq!(
            super::stack_value_as_bool(&StackValue::Integer(0)),
            Some(false)
        );
        assert_eq!(super::stack_value_as_bool(&StackValue::Null), None);

        assert_eq!(
            super::stack_value_as_u32(&StackValue::Integer(4_294_967_295)),
            Some(u32::MAX)
        );
        assert_eq!(super::stack_value_as_u32(&StackValue::Integer(-1)), None);
        assert_eq!(
            super::stack_value_as_u8(&StackValue::Integer(255)),
            Some(255)
        );
        assert_eq!(super::stack_value_as_u8(&StackValue::Integer(256)), None);

        assert_eq!(
            super::stack_value_as_bytes(&StackValue::Buffer(0, vec![1, 2, 3])),
            Some(vec![1, 2, 3])
        );
        assert_eq!(
            super::stack_value_as_fixed_bytes::<4>(&StackValue::ByteString(vec![1, 2, 3, 4])),
            Some([1, 2, 3, 4])
        );
        assert_eq!(
            super::stack_value_as_fixed_bytes::<4>(&StackValue::ByteString(vec![1, 2, 3])),
            None
        );
        assert_eq!(
            super::stack_value_as_string(&StackValue::ByteString(b"neo".to_vec())).as_deref(),
            Some("neo")
        );
        assert_eq!(
            super::stack_value_as_string(&StackValue::ByteString(vec![0xff])),
            None
        );

        assert_eq!(
            super::stack_value_into_items(StackValue::Array(0, vec![StackValue::Integer(1)])),
            Some(vec![StackValue::Integer(1)])
        );
        assert_eq!(super::stack_value_into_items(StackValue::Integer(1)), None);
    }

    #[test]
    fn concat_splice_values_uses_neovm_span_semantics() {
        {
            let result = super::concat_splice_values(
                &StackValue::ByteString(b"neo".to_vec()),
                &StackValue::Buffer(0, b"n4".to_vec()),
            )
            .unwrap();
            let expected = StackValue::Buffer(0, b"neon4".to_vec());
            assert!(
                result.structural_eq(&expected),
                "expected {:?} but got {:?}",
                expected,
                result
            );
        }
        {
            let result =
                super::concat_splice_values(&StackValue::Integer(128), &StackValue::Boolean(true))
                    .unwrap();
            let expected = StackValue::Buffer(0, vec![0x80, 0x00, 0x01]);
            assert!(
                result.structural_eq(&expected),
                "expected {:?} but got {:?}",
                expected,
                result
            );
        }
        assert_eq!(
            super::concat_splice_values(&StackValue::Null, &StackValue::Buffer(0, vec![2])),
            None
        );
    }

    #[test]
    fn slice_splice_value_uses_neovm_span_semantics() {
        {
            let result =
                super::slice_splice_value(&StackValue::ByteString(b"hello".to_vec()), 1, 3)
                    .unwrap();
            let expected = StackValue::Buffer(0, b"ell".to_vec());
            assert!(
                result.structural_eq(&expected),
                "expected {:?} but got {:?}",
                expected,
                result
            );
        }
        {
            let result = super::slice_splice_value(&StackValue::Integer(128), 0, 2).unwrap();
            let expected = StackValue::Buffer(0, vec![0x80, 0x00]);
            assert!(
                result.structural_eq(&expected),
                "expected {:?} but got {:?}",
                expected,
                result
            );
        }
        {
            let result =
                super::slice_splice_value(&StackValue::Buffer(0, b"hello".to_vec()), 5, 0).unwrap();
            let expected = StackValue::Buffer(0, Vec::new());
            assert!(
                result.structural_eq(&expected),
                "expected {:?} but got {:?}",
                expected,
                result
            );
        }
        assert_eq!(
            super::slice_splice_value(&StackValue::Buffer(0, b"hello".to_vec()), 4, 2),
            None
        );
        assert_eq!(super::slice_splice_value(&StackValue::Null, 0, 1), None);
    }
}
