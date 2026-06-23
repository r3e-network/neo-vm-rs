//! Runtime stack adapters for comparison and boolean opcodes.

use crate::runtime::RuntimeStack;
use crate::semantics::comparison as rules;

runtime_binary_bool_value_op!(equal, rules::equal_values);

runtime_binary_bool_value_op!(not_equal, rules::not_equal_values);

runtime_binary_bool_op!(less_than, rules::less_than_values);

runtime_binary_bool_op!(less_or_equal, rules::less_or_equal_values);

runtime_binary_bool_op!(greater_than, rules::greater_than_values);

runtime_binary_bool_op!(greater_or_equal, rules::greater_or_equal_values);

runtime_binary_bool_op!(num_equal, rules::num_equal_values);

runtime_binary_bool_op!(num_not_equal, rules::num_not_equal_values);

// Strict GetBoolean coercion (faults on a >32-byte ByteString operand), matching
// canonical NeoVM and the interpreter path; the prior lax boolean_value diverged.
runtime_binary_bool_op!(bool_and, rules::bool_and_values);

runtime_binary_bool_op!(bool_or, rules::bool_or_values);

runtime_unary_bool_op!(not, rules::not_value);

runtime_unary_bool_op!(nz, rules::nz_value);

pub fn is_null<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let value = runtime.pop_value();
    runtime.push_bool(rules::is_null(&value));
}
