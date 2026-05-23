//! Runtime stack adapters for arithmetic and bitwise opcodes.

use super::value_stack;
use crate::runtime::{push_value_result, RuntimeStack};
use crate::semantics::arithmetic as rules;

pub fn add<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    value_stack::apply_or_fault(runtime, |stack| {
        value_stack::binary_value(stack, rules::add_values)
    });
}

pub fn sub<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    value_stack::apply_or_fault(runtime, |stack| {
        value_stack::binary_value(stack, rules::sub_values)
    });
}

pub fn mul<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    value_stack::apply_or_fault(runtime, |stack| {
        value_stack::binary_value(stack, rules::mul_values)
    });
}

pub fn div<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    value_stack::apply_or_fault(runtime, |stack| {
        value_stack::binary_value(stack, rules::div_values)
    });
}

pub fn modulo<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    value_stack::apply_or_fault(runtime, |stack| {
        value_stack::binary_value(stack, rules::modulo_values)
    });
}

pub fn negate<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    value_stack::apply_or_fault(runtime, |stack| {
        value_stack::unary_value(stack, rules::negate_value)
    });
}

pub fn abs<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    value_stack::apply_or_fault(runtime, |stack| {
        value_stack::unary_value(stack, rules::abs_value)
    });
}

pub fn sign<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    value_stack::apply_or_fault(runtime, |stack| {
        value_stack::unary_value(stack, rules::sign_value)
    });
}

pub fn max<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    value_stack::apply_or_fault(runtime, |stack| {
        value_stack::binary_value(stack, rules::max_values)
    });
}

pub fn min<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    value_stack::apply_or_fault(runtime, |stack| {
        value_stack::binary_value(stack, rules::min_values)
    });
}

pub fn pow<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    value_stack::apply_or_fault(runtime, |stack| {
        value_stack::binary_value(stack, rules::pow_values)
    });
}

pub fn sqrt<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    value_stack::apply_or_fault(runtime, |stack| {
        value_stack::unary_value(stack, rules::sqrt_value)
    });
}

pub fn modmul<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    value_stack::apply_or_fault(runtime, |stack| {
        value_stack::ternary_value(stack, rules::modmul_values)
    });
}

pub fn modpow<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    value_stack::apply_or_fault(runtime, |stack| {
        value_stack::ternary_value(stack, rules::modpow_values)
    });
}

pub fn shl<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let shift = runtime.pop_i64();
    let value = runtime.pop_value();
    push_value_result(runtime, rules::shl_value(value, shift));
}

pub fn shr<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let shift = runtime.pop_i64();
    let value = runtime.pop_value();
    push_value_result(runtime, rules::shr_value(value, shift));
}

pub fn bitwise_and<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    value_stack::apply_or_fault(runtime, |stack| {
        value_stack::binary_value(stack, rules::bitwise_and_values)
    });
}

pub fn bitwise_or<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    value_stack::apply_or_fault(runtime, |stack| {
        value_stack::binary_value(stack, rules::bitwise_or_values)
    });
}

pub fn bitwise_xor<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    value_stack::apply_or_fault(runtime, |stack| {
        value_stack::binary_value(stack, rules::bitwise_xor_values)
    });
}

pub fn bitwise_not<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    value_stack::apply_or_fault(runtime, |stack| {
        value_stack::unary_value(stack, rules::invert_value)
    });
}

pub fn inc<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    value_stack::apply_or_fault(runtime, |stack| {
        value_stack::unary_value(stack, rules::inc_value)
    });
}

pub fn dec<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    value_stack::apply_or_fault(runtime, |stack| {
        value_stack::unary_value(stack, rules::dec_value)
    });
}

pub fn within<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    let x = runtime.pop_value();
    match rules::within_values(x, a, b) {
        Ok(value) => runtime.push_bool(value),
        Err(message) => runtime.fault(&message),
    }
}
