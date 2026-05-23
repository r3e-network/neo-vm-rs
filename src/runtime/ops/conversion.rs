//! Runtime stack adapters for type conversion opcodes.

use crate::runtime::{push_value_result, RuntimeStack};
use crate::{default_value_for_type_tag, semantics::conversion as rules, StackValue};

pub fn is_type<R: RuntimeStack + ?Sized>(runtime: &mut R, type_byte: u8) {
    let value = runtime.pop_value();
    runtime.push_bool(rules::is_type(&value, type_byte));
}

pub fn convert_to<R: RuntimeStack + ?Sized>(runtime: &mut R, target_type: u8) {
    let value = runtime.pop_value();
    push_value_result(runtime, rules::convert_value(value, target_type));
}

pub fn push_bigint<R: RuntimeStack + ?Sized>(runtime: &mut R, bytes: &[u8]) {
    runtime.push_value(StackValue::BigInteger(bytes.to_vec()));
}

pub fn push_default<R: RuntimeStack + ?Sized>(runtime: &mut R, type_byte: u8) {
    runtime.push_value(default_value_for_type_tag(type_byte));
}
