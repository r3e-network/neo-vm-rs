//! Runtime stack adapters for comparison and boolean opcodes.

use alloc::string::String;

use super::apply_or_fault;
use crate::runtime::RuntimeStack;
use crate::semantics::{comparison as rules, stack_shape};

pub fn equal<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    apply_or_fault(runtime, |stack| {
        stack_shape::binary_bool(stack, |left, right| Ok(rules::equal_values(left, right)))
    });
}

pub fn not_equal<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    apply_or_fault(runtime, |stack| {
        stack_shape::binary_bool(stack, |left, right| {
            Ok(rules::not_equal_values(left, right))
        })
    });
}

pub fn less_than<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    apply_or_fault(runtime, |stack| {
        stack_shape::binary_bool(stack, rules::less_than_values)
    });
}

pub fn less_or_equal<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    apply_or_fault(runtime, |stack| {
        stack_shape::binary_bool(stack, rules::less_or_equal_values)
    });
}

pub fn greater_than<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    apply_or_fault(runtime, |stack| {
        stack_shape::binary_bool(stack, rules::greater_than_values)
    });
}

pub fn greater_or_equal<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    apply_or_fault(runtime, |stack| {
        stack_shape::binary_bool(stack, rules::greater_or_equal_values)
    });
}

pub fn num_equal<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    apply_or_fault(runtime, |stack| {
        stack_shape::binary_bool(stack, rules::num_equal_values)
    });
}

pub fn num_not_equal<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    apply_or_fault(runtime, |stack| {
        stack_shape::binary_bool(stack, rules::num_not_equal_values)
    });
}

pub fn bool_and<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    apply_or_fault(runtime, |stack| {
        stack_shape::bool_binary(stack, rules::bool_and, rules::boolean_value)
    });
}

pub fn bool_or<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    apply_or_fault(runtime, |stack| {
        stack_shape::bool_binary(stack, rules::bool_or, rules::boolean_value)
    });
}

pub fn not<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    apply_or_fault(runtime, |stack| {
        stack_shape::unary_bool(stack, rules::not_value)
    });
}

pub fn nz<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    apply_or_fault(runtime, |stack| {
        stack_shape::unary_bool(stack, rules::nz_value)
    });
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

fn bool_or_fault<R: RuntimeStack + ?Sized>(runtime: &mut R, result: Result<bool, String>) -> bool {
    match result {
        Ok(value) => value,
        Err(message) => {
            runtime.fault(&message);
            false
        }
    }
}
