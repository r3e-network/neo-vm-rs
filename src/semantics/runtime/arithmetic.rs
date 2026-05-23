//! Runtime stack adapters for arithmetic and bitwise opcodes.

use super::{push_value_result, RuntimeStack};
use crate::semantics::arithmetic as rules;

pub fn add<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    push_value_result(runtime, rules::add_values(a, b));
}

pub fn sub<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    push_value_result(runtime, rules::sub_values(a, b));
}

pub fn mul<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    push_value_result(runtime, rules::mul_values(a, b));
}

pub fn div<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    push_value_result(runtime, rules::div_values(a, b));
}

pub fn modulo<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    push_value_result(runtime, rules::modulo_values(a, b));
}

pub fn negate<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let a = runtime.pop_value();
    push_value_result(runtime, rules::negate_value(a));
}

pub fn abs<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let a = runtime.pop_value();
    push_value_result(runtime, rules::abs_value(a));
}

pub fn sign<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let a = runtime.pop_value();
    push_value_result(runtime, rules::sign_value(a));
}

pub fn max<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    push_value_result(runtime, rules::max_values(a, b));
}

pub fn min<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    push_value_result(runtime, rules::min_values(a, b));
}

pub fn pow<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let exponent = runtime.pop_value();
    let base = runtime.pop_value();
    push_value_result(runtime, rules::pow_values(base, exponent));
}

pub fn sqrt<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let a = runtime.pop_value();
    push_value_result(runtime, rules::sqrt_value(a));
}

pub fn modmul<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let modulus = runtime.pop_value();
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    push_value_result(runtime, rules::modmul_values(a, b, modulus));
}

pub fn modpow<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let modulus = runtime.pop_value();
    let exponent = runtime.pop_value();
    let base = runtime.pop_value();
    push_value_result(runtime, rules::modpow_values(base, exponent, modulus));
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
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    push_value_result(runtime, rules::bitwise_and_values(a, b));
}

pub fn bitwise_or<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    push_value_result(runtime, rules::bitwise_or_values(a, b));
}

pub fn bitwise_xor<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_value();
    let a = runtime.pop_value();
    push_value_result(runtime, rules::bitwise_xor_values(a, b));
}

pub fn bitwise_not<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let a = runtime.pop_value();
    push_value_result(runtime, rules::invert_value(a));
}

pub fn inc<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let a = runtime.pop_value();
    push_value_result(runtime, rules::inc_value(a));
}

pub fn dec<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let a = runtime.pop_value();
    push_value_result(runtime, rules::dec_value(a));
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
