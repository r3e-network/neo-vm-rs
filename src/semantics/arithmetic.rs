//! Shared integer arithmetic semantics for ABI-level NeoVM runtimes.

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use num_bigint::BigInt;
use num_traits::{ToPrimitive, Zero};

use crate::{encode_integer, semantics::numeric, StackValue};

/// Return the canonical NeoVM integer result representation.
pub(crate) fn numeric_stack_value(
    value: BigInt,
    overflow_message: &str,
) -> Result<StackValue, String> {
    match numeric::integer_result(value, overflow_message)? {
        numeric::IntegerResult::Small(value) => Ok(StackValue::Integer(value)),
        numeric::IntegerResult::Big(bytes) => Ok(StackValue::BigInteger(bytes)),
    }
}

/// Convert a primitive numeric-compatible value into the canonical BigInt form.
pub(crate) fn numeric_bigint(value: StackValue) -> Result<BigInt, String> {
    match value {
        StackValue::Integer(value) => Ok(BigInt::from(value)),
        StackValue::Boolean(value) => Ok(BigInt::from(if value { 1 } else { 0 })),
        StackValue::ByteString(value) | StackValue::BigInteger(value) => {
            numeric::decode_signed_le_bytes_bigint(&value)
        }
        StackValue::Null => Err("expected integer-compatible value".into()),
        StackValue::Buffer(_)
        | StackValue::Pointer(_)
        | StackValue::Array(_)
        | StackValue::Struct(_)
        | StackValue::Map(_)
        | StackValue::Interop(_)
        | StackValue::Iterator(_) => Err("expected integer-compatible value".into()),
    }
}

/// Bitwise inversion.
pub fn invert_value(value: StackValue) -> Result<StackValue, String> {
    let value = numeric_bigint(value).map_err(|error| {
        if error == "expected integer-compatible value" {
            "INVERT expects an integer or boolean".to_string()
        } else {
            error
        }
    })?;
    numeric_stack_value(!value, "integer overflow for INVERT")
}

/// Add two integer-compatible values.
pub fn add_values(left: StackValue, right: StackValue) -> Result<StackValue, String> {
    numeric_stack_value(
        numeric_bigint(left)? + numeric_bigint(right)?,
        "integer overflow for ADD",
    )
}

/// Subtract two integer-compatible values.
pub fn sub_values(left: StackValue, right: StackValue) -> Result<StackValue, String> {
    numeric_stack_value(
        numeric_bigint(left)? - numeric_bigint(right)?,
        "integer overflow for SUB",
    )
}

/// Multiply two integer-compatible values.
pub fn mul_values(left: StackValue, right: StackValue) -> Result<StackValue, String> {
    numeric_stack_value(
        numeric_bigint(left)? * numeric_bigint(right)?,
        "integer overflow for MUL",
    )
}

/// Divide two integer-compatible values.
pub fn div_values(left: StackValue, right: StackValue) -> Result<StackValue, String> {
    let right = numeric_bigint(right)?;
    if right.is_zero() {
        return Err("division by zero for DIV".into());
    }
    numeric_stack_value(numeric_bigint(left)? / right, "integer overflow for DIV")
}

/// Compute integer remainder for two integer-compatible values.
pub fn modulo_values(left: StackValue, right: StackValue) -> Result<StackValue, String> {
    let right = numeric_bigint(right)?;
    if right.is_zero() {
        return Err("division by zero for MOD".into());
    }
    numeric_stack_value(numeric_bigint(left)? % right, "integer overflow for MOD")
}

/// Negate an integer-compatible value.
pub fn negate_value(value: StackValue) -> Result<StackValue, String> {
    numeric_stack_value(-numeric_bigint(value)?, "integer overflow for NEGATE")
}

/// Return absolute value for an integer-compatible value.
pub fn abs_value(value: StackValue) -> Result<StackValue, String> {
    numeric_stack_value(
        numeric::bigint_abs(numeric_bigint(value)?),
        "integer overflow for ABS",
    )
}

/// Return sign as -1, 0, or 1.
pub fn sign_value(value: StackValue) -> Result<StackValue, String> {
    Ok(StackValue::Integer(numeric::bigint_sign(&numeric_bigint(
        value,
    )?)))
}

/// Increment an integer-compatible value.
pub fn inc_value(value: StackValue) -> Result<StackValue, String> {
    numeric_stack_value(
        numeric_bigint(value)? + BigInt::from(1),
        "integer overflow for INC",
    )
}

/// Decrement an integer-compatible value.
pub fn dec_value(value: StackValue) -> Result<StackValue, String> {
    numeric_stack_value(
        numeric_bigint(value)? - BigInt::from(1),
        "integer overflow for DEC",
    )
}

/// Raise `base` to `exponent`.
pub fn pow_values(base: StackValue, exponent: StackValue) -> Result<StackValue, String> {
    let exponent = numeric_bigint(exponent)?;
    if exponent < BigInt::from(0) {
        return Err("negative exponent for POW".into());
    }
    let exponent = exponent
        .to_u32()
        .ok_or_else(|| "exponent too large for POW".to_string())?;
    numeric_stack_value(
        numeric_bigint(base)?.pow(exponent),
        "integer overflow for POW",
    )
}

/// Integer square root.
pub fn sqrt_value(value: StackValue) -> Result<StackValue, String> {
    let value = numeric_bigint(value)?;
    if value < BigInt::from(0) {
        return Err("negative value for SQRT".into());
    }
    numeric_stack_value(value.sqrt(), "integer overflow for SQRT")
}

/// Compute `(left * right) % modulus`.
pub fn modmul_values(
    left: StackValue,
    right: StackValue,
    modulus: StackValue,
) -> Result<StackValue, String> {
    let modulus = numeric_bigint(modulus)?;
    if modulus.is_zero() {
        return Err("division by zero for MODMUL".into());
    }
    numeric_stack_value(
        (numeric_bigint(left)? * numeric_bigint(right)?) % modulus,
        "integer overflow for MODMUL",
    )
}

/// Compute `base^exponent % modulus`, including NeoVM modular inverse support.
pub fn modpow_values(
    base: StackValue,
    exponent: StackValue,
    modulus: StackValue,
) -> Result<StackValue, String> {
    let modulus = numeric_bigint(modulus)?;
    if modulus.is_zero() {
        return Err("division by zero for MODPOW".into());
    }
    numeric_stack_value(
        numeric::mod_pow_bigint(numeric_bigint(base)?, numeric_bigint(exponent)?, modulus)?,
        "integer overflow for MODPOW",
    )
}

/// Shift left. A zero shift preserves the original value.
pub fn shl_value(value: StackValue, shift: i64) -> Result<StackValue, String> {
    if !(0..=256).contains(&shift) {
        return Err("shift count out of range for SHL".into());
    }
    if shift == 0 {
        return Ok(value);
    }
    numeric_stack_value(
        numeric_bigint(value)? << (shift as usize),
        "integer overflow for SHL",
    )
}

/// Arithmetic shift right. A zero shift preserves the original value.
pub fn shr_value(value: StackValue, shift: i64) -> Result<StackValue, String> {
    if !(0..=256).contains(&shift) {
        return Err("shift count out of range for SHR".into());
    }
    if shift == 0 {
        return Ok(value);
    }
    numeric_stack_value(
        numeric_bigint(value)? >> (shift as usize),
        "integer overflow for SHR",
    )
}

/// Bitwise AND.
pub fn bitwise_and_values(left: StackValue, right: StackValue) -> Result<StackValue, String> {
    bitwise_values(&left, &right, |left, right| left & right)
}

/// Bitwise OR.
pub fn bitwise_or_values(left: StackValue, right: StackValue) -> Result<StackValue, String> {
    bitwise_values(&left, &right, |left, right| left | right)
}

/// Bitwise XOR.
pub fn bitwise_xor_values(left: StackValue, right: StackValue) -> Result<StackValue, String> {
    bitwise_values(&left, &right, |left, right| left ^ right)
}

fn bitwise_values<F>(left: &StackValue, right: &StackValue, op: F) -> Result<StackValue, String>
where
    F: Fn(u8, u8) -> u8,
{
    let left_bytes = bitwise_operand_bytes(left)?;
    let right_bytes = bitwise_operand_bytes(right)?;
    bigint_or_integer(numeric::bitwise_signed_bytes(
        &left_bytes,
        &right_bytes,
        op,
    )?)
}

fn bitwise_operand_bytes(value: &StackValue) -> Result<Vec<u8>, String> {
    match value {
        StackValue::Integer(value) => Ok(encode_integer(*value)),
        StackValue::BigInteger(value) | StackValue::ByteString(value) => Ok(value.clone()),
        StackValue::Boolean(value) => Ok(encode_integer(if *value { 1 } else { 0 })),
        StackValue::Null
        | StackValue::Buffer(_)
        | StackValue::Pointer(_)
        | StackValue::Array(_)
        | StackValue::Struct(_)
        | StackValue::Map(_)
        | StackValue::Interop(_)
        | StackValue::Iterator(_) => {
            Err("bitwise op expects primitive numeric or byte string operands".into())
        }
    }
}

fn bigint_or_integer(bytes: Vec<u8>) -> Result<StackValue, String> {
    let trimmed = numeric::trim_le_bytes(bytes);
    Ok(if trimmed.len() <= 8 {
        StackValue::Integer(numeric::bytes_to_integer(&trimmed))
    } else {
        StackValue::BigInteger(trimmed)
    })
}

/// Return the larger integer-compatible value.
pub fn max_values(left: StackValue, right: StackValue) -> Result<StackValue, String> {
    let left = numeric_bigint(left)?;
    let right = numeric_bigint(right)?;
    numeric_stack_value(
        if left > right { left } else { right },
        "integer overflow for MAX",
    )
}

/// Return the smaller integer-compatible value.
pub fn min_values(left: StackValue, right: StackValue) -> Result<StackValue, String> {
    let left = numeric_bigint(left)?;
    let right = numeric_bigint(right)?;
    numeric_stack_value(
        if left < right { left } else { right },
        "integer overflow for MIN",
    )
}

/// Return whether `lower <= value < upper`.
pub fn within_values(
    value: StackValue,
    lower: StackValue,
    upper: StackValue,
) -> Result<bool, String> {
    let value = numeric_bigint(value)?;
    let lower = numeric_bigint(lower)?;
    let upper = numeric_bigint(upper)?;
    Ok(value >= lower && value < upper)
}

/// Add two `i64` values using the RISC-V runtime fast-path wrapping rule.
#[must_use]
pub fn add_i64(left: i64, right: i64) -> i64 {
    left.wrapping_add(right)
}

/// Subtract two `i64` values using the RISC-V runtime fast-path wrapping rule.
#[must_use]
pub fn sub_i64(left: i64, right: i64) -> i64 {
    left.wrapping_sub(right)
}

/// Multiply two `i64` values using the RISC-V runtime fast-path wrapping rule.
#[must_use]
pub fn mul_i64(left: i64, right: i64) -> i64 {
    left.wrapping_mul(right)
}

/// Divide two integers, faulting on division by zero.
pub fn div_i64(left: i64, right: i64) -> Result<i64, &'static str> {
    if right == 0 {
        return Err("DIV: division by zero");
    }
    Ok(left.wrapping_div(right))
}

/// Compute integer remainder, faulting on division by zero.
pub fn modulo_i64(left: i64, right: i64) -> Result<i64, &'static str> {
    if right == 0 {
        return Err("MOD: division by zero");
    }
    Ok(left.wrapping_rem(right))
}

/// Negate an integer using wrapping arithmetic.
#[must_use]
pub fn negate_i64(value: i64) -> i64 {
    value.wrapping_neg()
}

/// Return absolute value using wrapping arithmetic.
#[must_use]
pub fn abs_i64(value: i64) -> i64 {
    value.wrapping_abs()
}

/// Return integer sign as -1, 0, or 1.
#[must_use]
pub fn sign_i64(value: i64) -> i64 {
    value.signum()
}

/// Return the larger value.
#[must_use]
pub fn max_i64(left: i64, right: i64) -> i64 {
    left.max(right)
}

/// Return the smaller value.
#[must_use]
pub fn min_i64(left: i64, right: i64) -> i64 {
    left.min(right)
}

/// Raise `base` to `exp`, using the compiled runtime's bounded fast path.
pub fn pow_i64(base: i64, exp: i64) -> Result<i64, &'static str> {
    if exp < 0 {
        return Err("POW: negative exponent");
    }
    if exp > 63 {
        return Err("POW: exponent too large for i64 fast path");
    }
    #[allow(clippy::cast_sign_loss)]
    Ok(base.wrapping_pow(exp as u32))
}

/// Integer square root.
pub fn sqrt_i64(value: i64) -> Result<i64, &'static str> {
    if value < 0 {
        return Err("SQRT: negative value");
    }
    Ok(isqrt(value as u64) as i64)
}

/// Compute `(left * right) % modulus`.
pub fn modmul_i64(left: i64, right: i64, modulus: i64) -> Result<i64, &'static str> {
    if modulus == 0 {
        return Err("MODMUL: division by zero");
    }
    let result = ((left as i128) * (right as i128)) % (modulus as i128);
    #[allow(clippy::cast_possible_truncation)]
    Ok(result as i64)
}

/// Compute `base^exp % modulus`.
pub fn modpow_i64(base: i64, exp: i64, modulus: i64) -> Result<i64, &'static str> {
    if modulus == 0 {
        return Err("MODPOW: division by zero");
    }
    if exp < 0 {
        return Err("MODPOW: negative exponent");
    }
    Ok(mod_pow_i64(base, exp, modulus))
}

/// Shift left by a bounded amount.
pub fn shl_i64(value: i64, shift: i64) -> Result<i64, &'static str> {
    if !(0..64).contains(&shift) {
        return Err("SHL: shift amount out of range");
    }
    #[allow(clippy::cast_sign_loss)]
    Ok(value.wrapping_shl(shift as u32))
}

/// Arithmetic shift right by a bounded amount.
pub fn shr_i64(value: i64, shift: i64) -> Result<i64, &'static str> {
    if !(0..64).contains(&shift) {
        return Err("SHR: shift amount out of range");
    }
    #[allow(clippy::cast_sign_loss)]
    Ok(value.wrapping_shr(shift as u32))
}

/// Bitwise AND.
#[must_use]
pub fn bitwise_and_i64(left: i64, right: i64) -> i64 {
    left & right
}

/// Bitwise OR.
#[must_use]
pub fn bitwise_or_i64(left: i64, right: i64) -> i64 {
    left | right
}

/// Bitwise XOR.
#[must_use]
pub fn bitwise_xor_i64(left: i64, right: i64) -> i64 {
    left ^ right
}

/// Bitwise NOT.
#[must_use]
pub fn bitwise_not_i64(value: i64) -> i64 {
    !value
}

/// Increment using wrapping arithmetic.
#[must_use]
pub fn inc_i64(value: i64) -> i64 {
    value.wrapping_add(1)
}

/// Decrement using wrapping arithmetic.
#[must_use]
pub fn dec_i64(value: i64) -> i64 {
    value.wrapping_sub(1)
}

/// Return whether `lower <= value < upper`.
#[must_use]
pub fn within_i64(value: i64, lower: i64, upper: i64) -> bool {
    value >= lower && value < upper
}

fn mod_pow_i64(mut base: i64, mut exp: i64, modulus: i64) -> i64 {
    if modulus == 1 || modulus == -1 {
        return 0;
    }
    let m = modulus as i128;
    let mut result: i128 = 1;
    base = ((base as i128) % m) as i64;
    let mut b = base as i128;
    while exp > 0 {
        if exp & 1 == 1 {
            result = (result * b) % m;
        }
        exp >>= 1;
        b = (b * b) % m;
    }
    result as i64
}

fn isqrt(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = x.div_ceil(2);
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}
