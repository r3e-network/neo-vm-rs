//! Shared type inspection and conversion semantics for ABI-level NeoVM runtimes.

use alloc::{format, string::String};

use crate::{
    normalize_stack_item_type_tag, StackValue, COMPACT_TAG_ARRAY, COMPACT_TAG_BOOLEAN,
    COMPACT_TAG_BUFFER, COMPACT_TAG_BYTESTRING, COMPACT_TAG_INTEGER, COMPACT_TAG_STRUCT,
};

/// Return whether a value has the requested compact or NeoVM stack item type.
#[must_use]
pub fn is_type(value: &StackValue, type_tag: u8) -> bool {
    value.compact_type_tag() == normalize_stack_item_type_tag(type_tag)
}

/// Convert a public ABI stack value to the requested compact or NeoVM type.
pub fn convert_value(value: StackValue, target_type: u8) -> Result<StackValue, String> {
    match normalize_stack_item_type_tag(target_type) {
        COMPACT_TAG_INTEGER => convert_to_integer(&value),
        COMPACT_TAG_BOOLEAN => Ok(StackValue::Boolean(value.to_bool())),
        COMPACT_TAG_BYTESTRING => value
            .convert_to_byte_string_value()
            .ok_or_else(|| "CONVERT: cannot convert to ByteString".into()),
        COMPACT_TAG_BUFFER => value
            .convert_to_buffer_value()
            .ok_or_else(|| "CONVERT: cannot convert to Buffer".into()),
        COMPACT_TAG_ARRAY => match value {
            StackValue::Array(_) => Ok(value),
            StackValue::Struct(items) => Ok(StackValue::Array(items)),
            _ => Err("CONVERT: cannot convert to Array".into()),
        },
        COMPACT_TAG_STRUCT => match value {
            StackValue::Struct(_) => Ok(value),
            StackValue::Array(items) => Ok(StackValue::Struct(items)),
            _ => Err("CONVERT: cannot convert to Struct".into()),
        },
        other => Err(format!("CONVERT: unsupported target type {other}")),
    }
}

fn convert_to_integer(value: &StackValue) -> Result<StackValue, String> {
    let Some(integer) = value.to_i128() else {
        return Err("CONVERT: cannot convert to Integer".into());
    };
    let Ok(integer) = i64::try_from(integer) else {
        return Err("CONVERT: integer too large for i64".into());
    };
    Ok(StackValue::Integer(integer))
}
