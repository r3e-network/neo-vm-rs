//! Runtime stack adapters for arithmetic and bitwise opcodes.

use super::{push_i64_result, RuntimeStack};
use crate::semantics::arithmetic as rules;

pub fn add<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_i64();
    let a = runtime.pop_i64();
    runtime.push_i64(rules::add_i64(a, b));
}

pub fn sub<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_i64();
    let a = runtime.pop_i64();
    runtime.push_i64(rules::sub_i64(a, b));
}

pub fn mul<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_i64();
    let a = runtime.pop_i64();
    runtime.push_i64(rules::mul_i64(a, b));
}

pub fn div<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_i64();
    let a = runtime.pop_i64();
    push_i64_result(runtime, rules::div_i64(a, b));
}

pub fn modulo<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_i64();
    let a = runtime.pop_i64();
    push_i64_result(runtime, rules::modulo_i64(a, b));
}

pub fn negate<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let a = runtime.pop_i64();
    runtime.push_i64(rules::negate_i64(a));
}

pub fn abs<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let a = runtime.pop_i64();
    runtime.push_i64(rules::abs_i64(a));
}

pub fn sign<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let a = runtime.pop_i64();
    runtime.push_i64(rules::sign_i64(a));
}

pub fn max<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_i64();
    let a = runtime.pop_i64();
    runtime.push_i64(rules::max_i64(a, b));
}

pub fn min<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_i64();
    let a = runtime.pop_i64();
    runtime.push_i64(rules::min_i64(a, b));
}

pub fn pow<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let exponent = runtime.pop_i64();
    let base = runtime.pop_i64();
    push_i64_result(runtime, rules::pow_i64(base, exponent));
}

pub fn sqrt<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let a = runtime.pop_i64();
    push_i64_result(runtime, rules::sqrt_i64(a));
}

pub fn modmul<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let modulus = runtime.pop_i64();
    let b = runtime.pop_i64();
    let a = runtime.pop_i64();
    push_i64_result(runtime, rules::modmul_i64(a, b, modulus));
}

pub fn modpow<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let modulus = runtime.pop_i64();
    let exponent = runtime.pop_i64();
    let base = runtime.pop_i64();
    push_i64_result(runtime, rules::modpow_i64(base, exponent, modulus));
}

pub fn shl<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let shift = runtime.pop_i64();
    let value = runtime.pop_i64();
    push_i64_result(runtime, rules::shl_i64(value, shift));
}

pub fn shr<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let shift = runtime.pop_i64();
    let value = runtime.pop_i64();
    push_i64_result(runtime, rules::shr_i64(value, shift));
}

pub fn bitwise_and<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_i64();
    let a = runtime.pop_i64();
    runtime.push_i64(rules::bitwise_and_i64(a, b));
}

pub fn bitwise_or<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_i64();
    let a = runtime.pop_i64();
    runtime.push_i64(rules::bitwise_or_i64(a, b));
}

pub fn bitwise_xor<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_i64();
    let a = runtime.pop_i64();
    runtime.push_i64(rules::bitwise_xor_i64(a, b));
}

pub fn bitwise_not<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let a = runtime.pop_i64();
    runtime.push_i64(rules::bitwise_not_i64(a));
}

pub fn inc<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let a = runtime.pop_i64();
    runtime.push_i64(rules::inc_i64(a));
}

pub fn dec<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let a = runtime.pop_i64();
    runtime.push_i64(rules::dec_i64(a));
}

pub fn within<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let b = runtime.pop_i64();
    let a = runtime.pop_i64();
    let x = runtime.pop_i64();
    runtime.push_bool(rules::within_i64(x, a, b));
}
