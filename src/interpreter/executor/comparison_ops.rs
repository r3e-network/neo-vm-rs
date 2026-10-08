use super::super::helpers::{pop_item, vm_equal};
use super::super::runtime_types::{StackValue, to_abi_value};
use alloc::{string::String, vec::Vec};

pub(super) fn jump_equal(
    stack: &mut Vec<StackValue>,
    reference_mode: bool,
) -> Result<bool, String> {
    let right = pop_item(stack)?;
    let left = pop_item(stack)?;
    if reference_mode {
        neo_integer_equal(&left, &right)
    } else {
        Ok(vm_equal(&left, &right))
    }
}

fn neo_integer_equal(left: &StackValue, right: &StackValue) -> Result<bool, String> {
    crate::semantics::comparison::num_equal_values(&to_abi_value(left), &to_abi_value(right))
}
