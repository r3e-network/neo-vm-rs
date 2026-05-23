//! Runtime stack adapters for comparison and boolean opcodes.

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
