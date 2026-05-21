//! Shared integer arithmetic semantics for ABI-level NeoVM runtimes.

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
