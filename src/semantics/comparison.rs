//! Shared comparison and boolean semantics for ABI-level NeoVM runtimes.

use alloc::string::{String, ToString};

use num_bigint::BigInt;
use num_traits::Zero;

use crate::{StackValue, semantics::numeric};

/// Return NeoVM equality for public ABI values.
#[must_use]
pub fn equal_values(left: &StackValue, right: &StackValue) -> bool {
    left == right
}

/// Return NeoVM inequality for public ABI values.
#[must_use]
pub fn not_equal_values(left: &StackValue, right: &StackValue) -> bool {
    !equal_values(left, right)
}

/// Numeric equality across primitive numeric and byte-string values.
pub fn num_equal_values(left: &StackValue, right: &StackValue) -> Result<bool, String> {
    Ok(integer_value_for_num_equal(left)? == integer_value_for_num_equal(right)?)
}

/// Numeric inequality across primitive numeric and byte-string values.
pub fn num_not_equal_values(left: &StackValue, right: &StackValue) -> Result<bool, String> {
    Ok(!num_equal_values(left, right)?)
}

/// Numeric less-than with NeoVM's null-as-false comparison rule.
pub fn less_than_values(left: &StackValue, right: &StackValue) -> Result<bool, String> {
    compare_ordered_values(left, right, |left, right| left < right)
}

/// Numeric less-than-or-equal with NeoVM's null-as-false comparison rule.
pub fn less_or_equal_values(left: &StackValue, right: &StackValue) -> Result<bool, String> {
    compare_ordered_values(left, right, |left, right| left <= right)
}

/// Numeric greater-than with NeoVM's null-as-false comparison rule.
pub fn greater_than_values(left: &StackValue, right: &StackValue) -> Result<bool, String> {
    compare_ordered_values(left, right, |left, right| left > right)
}

/// Numeric greater-than-or-equal with NeoVM's null-as-false comparison rule.
pub fn greater_or_equal_values(left: &StackValue, right: &StackValue) -> Result<bool, String> {
    compare_ordered_values(left, right, |left, right| left >= right)
}

// Strict ordered comparisons for the JMP* family. Canonical NeoVM jump
// comparisons (JmpGt/JmpGe/JmpLt/JmpLe — and JmpEq/JmpNe) call `Pop().GetInteger()`
// on BOTH operands with NO null guard, so a null/non-integer operand FAULTS
// (uncatchable InvalidCastException). This differs from the numeric LT/LE/GT/GE
// opcodes above, which deliberately treat null as false.
fn compare_ordered_strict<F>(left: &StackValue, right: &StackValue, compare: F) -> Result<bool, String>
where
    F: Fn(&BigInt, &BigInt) -> bool,
{
    Ok(compare(
        &integer_value_for_ordering(left)?,
        &integer_value_for_ordering(right)?,
    ))
}

/// Strict numeric greater-than for JMPGT (faults on null/non-integer).
pub fn strict_greater_than_values(left: &StackValue, right: &StackValue) -> Result<bool, String> {
    compare_ordered_strict(left, right, |left, right| left > right)
}

/// Strict numeric greater-or-equal for JMPGE (faults on null/non-integer).
pub fn strict_greater_or_equal_values(left: &StackValue, right: &StackValue) -> Result<bool, String> {
    compare_ordered_strict(left, right, |left, right| left >= right)
}

/// Strict numeric less-than for JMPLT (faults on null/non-integer).
pub fn strict_less_than_values(left: &StackValue, right: &StackValue) -> Result<bool, String> {
    compare_ordered_strict(left, right, |left, right| left < right)
}

/// Strict numeric less-or-equal for JMPLE (faults on null/non-integer).
pub fn strict_less_or_equal_values(left: &StackValue, right: &StackValue) -> Result<bool, String> {
    compare_ordered_strict(left, right, |left, right| left <= right)
}

fn compare_ordered_values<F>(
    left: &StackValue,
    right: &StackValue,
    compare: F,
) -> Result<bool, String>
where
    F: Fn(&BigInt, &BigInt) -> bool,
{
    if matches!(left, StackValue::Null) || matches!(right, StackValue::Null) {
        return Ok(false);
    }
    Ok(compare(
        &integer_value_for_ordering(left)?,
        &integer_value_for_ordering(right)?,
    ))
}

fn integer_value_for_ordering(value: &StackValue) -> Result<BigInt, String> {
    integer_value(value, "expected integer on stack")
}

fn integer_value_for_num_equal(value: &StackValue) -> Result<BigInt, String> {
    integer_value(
        value,
        "NUMEQUAL expects primitive numeric or byte string values",
    )
}

/// Boolean AND.
#[must_use]
pub fn bool_and(left: bool, right: bool) -> bool {
    left && right
}

/// Boolean OR.
#[must_use]
pub fn bool_or(left: bool, right: bool) -> bool {
    left || right
}

/// NeoVM `NZ`, which is a numeric zero check rather than general truthiness.
pub fn nz_value(value: &StackValue) -> Result<bool, String> {
    Ok(!integer_value_for_nz(value)?.is_zero())
}

fn integer_value_for_nz(value: &StackValue) -> Result<BigInt, String> {
    integer_value(value, "expected integer-compatible value")
}

fn integer_value(value: &StackValue, type_error: &'static str) -> Result<BigInt, String> {
    match value {
        StackValue::Integer(value) => Ok(BigInt::from(*value)),
        StackValue::BigInteger(value) | StackValue::ByteString(value) => {
            numeric::decode_signed_le_bytes_bigint(value)
        }
        StackValue::Boolean(value) => Ok(BigInt::from(if *value { 1 } else { 0 })),
        StackValue::Null
        | StackValue::Buffer(_, _)
        | StackValue::Pointer(_)
        | StackValue::Array(_, _)
        | StackValue::Struct(_, _)
        | StackValue::Map(_, _)
        | StackValue::Interop(_)
        | StackValue::Iterator(_) => Err(type_error.to_string()),
    }
}

/// General NeoVM truthiness used by Boolean conversion and boolean opcodes.
#[must_use]
pub fn boolean_value(value: &StackValue) -> bool {
    value.to_bool()
}

/// NeoVM `GetBoolean()` — the strict boolean coercion shared by NOT, JMPIF/
/// JMPIFNOT/ASSERT (via `pop_boolean`) and BOOLAND/BOOLOR. Only a ByteString
/// (or the BigInteger integer representation) wider than `Integer.MaxSize`
/// (32 bytes) faults (uncatchable `InvalidCastException`); Buffer/Pointer/
/// Array/Struct/Map/InteropInterface are unconditionally truthy, matching
/// canonical (`CompoundType`'s sealed `GetBoolean` returns `true`).
pub fn strict_boolean_value(value: &StackValue) -> Result<bool, String> {
    match value {
        StackValue::Boolean(value) => Ok(*value),
        StackValue::Integer(value) => Ok(*value != 0),
        StackValue::BigInteger(bytes) | StackValue::ByteString(bytes) => {
            if bytes.len() > numeric::MAX_INTEGER_SIZE {
                return Err("integer size exceeds maximum".to_string());
            }
            Ok(bytes.iter().any(|byte| *byte != 0))
        }
        StackValue::Null => Ok(false),
        StackValue::Buffer(_, _)
        | StackValue::Pointer(_)
        | StackValue::Array(_, _)
        | StackValue::Struct(_, _)
        | StackValue::Map(_, _)
        | StackValue::Interop(_)
        | StackValue::Iterator(_) => Ok(true),
    }
}

/// Boolean NOT using NeoVM's strict integer path.
pub fn not_value(value: &StackValue) -> Result<bool, String> {
    Ok(!strict_boolean_value(value)?)
}

/// NeoVM `BOOLAND`: both operands coerced via the strict `GetBoolean` path.
///
/// Canonical NeoVM pops and `GetBoolean()`s BOTH operands eagerly (top/right
/// first, then left) BEFORE combining — it does not short-circuit. So a
/// ByteString wider than `Integer.MaxSize` (32 bytes) in EITHER position faults
/// (uncatchably) regardless of the other operand's truthiness. Evaluating both
/// before the `&&` reproduces that exactly (a lax `left? && right?` would skip
/// the right operand's fault when `left` is false).
pub fn bool_and_values(left: &StackValue, right: &StackValue) -> Result<bool, String> {
    let right = strict_boolean_value(right)?;
    let left = strict_boolean_value(left)?;
    Ok(left && right)
}

/// NeoVM `BOOLOR`: both operands coerced via the strict `GetBoolean` path,
/// evaluated eagerly with no short-circuit (see [`bool_and_values`]).
pub fn bool_or_values(left: &StackValue, right: &StackValue) -> Result<bool, String> {
    let right = strict_boolean_value(right)?;
    let left = strict_boolean_value(left)?;
    Ok(left || right)
}

/// Null predicate.
#[must_use]
pub fn is_null(value: &StackValue) -> bool {
    matches!(value, StackValue::Null)
}
