//! Runtime stack adapters for comparison and boolean opcodes.

use alloc::string::String;

use super::RuntimeStack;
use crate::semantics::comparison as rules;

pub fn equal<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    runtime.push_bool(rules::equal_values(&a, &b));
}

pub fn not_equal<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    runtime.push_bool(rules::not_equal_values(&a, &b));
}

pub fn less_than<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    push_bool_result(runtime, rules::less_than_values(&a, &b));
}

pub fn less_or_equal<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    push_bool_result(runtime, rules::less_or_equal_values(&a, &b));
}

pub fn greater_than<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    push_bool_result(runtime, rules::greater_than_values(&a, &b));
}

pub fn greater_or_equal<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    push_bool_result(runtime, rules::greater_or_equal_values(&a, &b));
}

pub fn num_equal<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    push_bool_result(runtime, rules::num_equal_values(&a, &b));
}

pub fn num_not_equal<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    push_bool_result(runtime, rules::num_not_equal_values(&a, &b));
}

pub fn bool_and<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    runtime.push_bool(rules::bool_and(
        rules::boolean_value(&a),
        rules::boolean_value(&b),
    ));
}

pub fn bool_or<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    runtime.push_bool(rules::bool_or(
        rules::boolean_value(&a),
        rules::boolean_value(&b),
    ));
}

pub fn not<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let value = runtime.pop_value();
    push_bool_result(runtime, rules::not_value(&value));
}

pub fn nz<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let value = runtime.pop_value();
    push_bool_result(runtime, rules::nz_value(&value));
}

pub fn is_null<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let value = runtime.pop_value();
    runtime.push_bool(rules::is_null(&value));
}

pub fn pop_bool<R: RuntimeStack + ?Sized>(runtime: &mut R) -> bool {
    runtime.pop_bool_value()
}

pub fn pop_cmp_eq<R: RuntimeStack + ?Sized>(runtime: &mut R) -> bool {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    rules::equal_values(&a, &b)
}

pub fn pop_cmp_ne<R: RuntimeStack + ?Sized>(runtime: &mut R) -> bool {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    rules::not_equal_values(&a, &b)
}

pub fn pop_cmp_gt<R: RuntimeStack + ?Sized>(runtime: &mut R) -> bool {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    bool_or_fault(runtime, rules::greater_than_values(&a, &b))
}

pub fn pop_cmp_ge<R: RuntimeStack + ?Sized>(runtime: &mut R) -> bool {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    bool_or_fault(runtime, rules::greater_or_equal_values(&a, &b))
}

pub fn pop_cmp_lt<R: RuntimeStack + ?Sized>(runtime: &mut R) -> bool {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    bool_or_fault(runtime, rules::less_than_values(&a, &b))
}

pub fn pop_cmp_le<R: RuntimeStack + ?Sized>(runtime: &mut R) -> bool {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    bool_or_fault(runtime, rules::less_or_equal_values(&a, &b))
}

fn push_bool_result<R: RuntimeStack + ?Sized>(runtime: &mut R, result: Result<bool, String>) {
    match result {
        Ok(value) => runtime.push_bool(value),
        Err(message) => runtime.fault(&message),
    }
}

fn bool_or_fault<R: RuntimeStack + ?Sized>(runtime: &mut R, result: Result<bool, String>) -> bool {
    match result {
        Ok(value) => value,
        Err(message) => {
            runtime.fault(&message);
            false
        }
    }
}
