//! Shared comparison and boolean semantics for ABI-level NeoVM runtimes.

use crate::StackValue;

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

/// Numeric less-than.
#[must_use]
pub fn less_than_i64(left: i64, right: i64) -> bool {
    left < right
}

/// Numeric less-than-or-equal.
#[must_use]
pub fn less_or_equal_i64(left: i64, right: i64) -> bool {
    left <= right
}

/// Numeric greater-than.
#[must_use]
pub fn greater_than_i64(left: i64, right: i64) -> bool {
    left > right
}

/// Numeric greater-than-or-equal.
#[must_use]
pub fn greater_or_equal_i64(left: i64, right: i64) -> bool {
    left >= right
}

/// Numeric equality.
#[must_use]
pub fn num_equal_i64(left: i64, right: i64) -> bool {
    left == right
}

/// Numeric inequality.
#[must_use]
pub fn num_not_equal_i64(left: i64, right: i64) -> bool {
    left != right
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

/// Boolean NOT.
#[must_use]
pub fn bool_not(value: bool) -> bool {
    !value
}

/// NeoVM truthiness.
#[must_use]
pub fn nz(value: &StackValue) -> bool {
    value.to_bool()
}

/// Null predicate.
#[must_use]
pub fn is_null(value: &StackValue) -> bool {
    matches!(value, StackValue::Null)
}
