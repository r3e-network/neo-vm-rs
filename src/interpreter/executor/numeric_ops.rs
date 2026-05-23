use super::super::helpers::*;
use super::super::opcodes::*;
use super::super::runtime_types::StackValue;
use super::control::Dispatch;
use crate::{
    semantics::{arithmetic as arithmetic_rules, comparison as comparison_rules},
    StackValue as AbiStackValue,
};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

#[inline]
pub(super) fn execute(opcode: u8, stack: &mut Vec<StackValue>) -> Result<Dispatch, String> {
    match opcode {
        // =============================================================================
        // BITWISE LOGIC OPERATIONS (0x90-0x98)
        // =============================================================================
        INVERT => unary_value(stack, arithmetic_rules::invert_value)?,
        AND => binary_value(stack, arithmetic_rules::bitwise_and_values)?,
        OR => binary_value(stack, arithmetic_rules::bitwise_or_values)?,
        XOR => binary_value(stack, arithmetic_rules::bitwise_xor_values)?,
        NUMEQUAL => binary_bool(stack, comparison_rules::num_equal_values)?,
        NUMNOTEQUAL => binary_bool(stack, comparison_rules::num_not_equal_values)?,
        EQUAL => {
            let right = pop_item(stack)?;
            let left = pop_item(stack)?;
            stack.push(StackValue::Boolean(vm_equal(&left, &right)));
        }
        NOTEQUAL => {
            let right = pop_item(stack)?;
            let left = pop_item(stack)?;
            stack.push(StackValue::Boolean(!vm_equal(&left, &right)));
        }
        LT => binary_bool(stack, comparison_rules::less_than_values)?,
        LE => binary_bool(stack, comparison_rules::less_or_equal_values)?,
        GT => binary_bool(stack, comparison_rules::greater_than_values)?,
        GE => binary_bool(stack, comparison_rules::greater_or_equal_values)?,
        // =============================================================================
        // ARITHMETIC OPERATIONS (0x99-0xbb)
        // =============================================================================
        SIGN => unary_value(stack, arithmetic_rules::sign_value)?,
        ABS => unary_value(stack, arithmetic_rules::abs_value)?,
        NEGATE => unary_value(stack, arithmetic_rules::negate_value)?,
        ADD => binary_value(stack, arithmetic_rules::add_values)?,
        INC => unary_value(stack, arithmetic_rules::inc_value)?,
        SUB => binary_value(stack, arithmetic_rules::sub_values)?,
        POW => binary_value(stack, arithmetic_rules::pow_values)?,
        SQRT => unary_value(stack, arithmetic_rules::sqrt_value)?,
        MODMUL => ternary_value(stack, arithmetic_rules::modmul_values)?,
        MODPOW => ternary_value(stack, arithmetic_rules::modpow_values)?,
        SHL => shift_value(stack, arithmetic_rules::shl_value)?,
        SHR => shift_value(stack, arithmetic_rules::shr_value)?,
        NOT => {
            let value = pop_abi_value(stack)?;
            stack.push(StackValue::Boolean(comparison_rules::not_value(&value)?));
        }
        MUL => binary_value(stack, arithmetic_rules::mul_values)?,
        DIV => binary_value(stack, arithmetic_rules::div_values)?,
        MOD => binary_value(stack, arithmetic_rules::modulo_values)?,
        DEC => unary_value(stack, arithmetic_rules::dec_value)?,
        BOOLAND => bool_binary(stack, comparison_rules::bool_and)?,
        BOOLOR => bool_binary(stack, comparison_rules::bool_or)?,
        NZ => {
            let value = pop_abi_value(stack)?;
            stack.push(StackValue::Boolean(comparison_rules::nz_value(&value)?));
        }
        MIN => binary_value(stack, arithmetic_rules::min_values)?,
        MAX => binary_value(stack, arithmetic_rules::max_values)?,
        WITHIN => {
            let upper = pop_abi_value(stack)?;
            let lower = pop_abi_value(stack)?;
            let value = pop_abi_value(stack)?;
            stack.push(StackValue::Boolean(arithmetic_rules::within_values(
                value, lower, upper,
            )?));
        }
        _ => unreachable!("opcode routed to numeric_ops: 0x{opcode:02x}"),
    }
    Ok(Dispatch::Fallthrough)
}

fn unary_value(
    stack: &mut Vec<StackValue>,
    op: fn(AbiStackValue) -> Result<AbiStackValue, String>,
) -> Result<(), String> {
    let value = pop_abi_value(stack)?;
    push_abi_value(stack, op(value)?)
}

fn binary_value(
    stack: &mut Vec<StackValue>,
    op: fn(AbiStackValue, AbiStackValue) -> Result<AbiStackValue, String>,
) -> Result<(), String> {
    let right = pop_abi_value(stack)?;
    let left = pop_abi_value(stack)?;
    push_abi_value(stack, op(left, right)?)
}

fn ternary_value(
    stack: &mut Vec<StackValue>,
    op: fn(AbiStackValue, AbiStackValue, AbiStackValue) -> Result<AbiStackValue, String>,
) -> Result<(), String> {
    let third = pop_abi_value(stack)?;
    let second = pop_abi_value(stack)?;
    let first = pop_abi_value(stack)?;
    push_abi_value(stack, op(first, second, third)?)
}

fn binary_bool(
    stack: &mut Vec<StackValue>,
    op: fn(&AbiStackValue, &AbiStackValue) -> Result<bool, String>,
) -> Result<(), String> {
    let right = pop_abi_value(stack)?;
    let left = pop_abi_value(stack)?;
    stack.push(StackValue::Boolean(op(&left, &right)?));
    Ok(())
}

fn bool_binary(stack: &mut Vec<StackValue>, op: fn(bool, bool) -> bool) -> Result<(), String> {
    let right = pop_abi_value(stack)?;
    let left = pop_abi_value(stack)?;
    stack.push(StackValue::Boolean(op(
        comparison_rules::boolean_value(&left),
        comparison_rules::boolean_value(&right),
    )));
    Ok(())
}

fn shift_value(
    stack: &mut Vec<StackValue>,
    op: fn(AbiStackValue, i64) -> Result<AbiStackValue, String>,
) -> Result<(), String> {
    let shift = pop_shift_count(stack)?;
    let value = pop_item(stack)?;
    if shift == 0 {
        stack.push(value);
        return Ok(());
    }
    push_abi_value(stack, op(into_abi_value(value), shift)?)
}

fn pop_abi_value(stack: &mut Vec<StackValue>) -> Result<AbiStackValue, String> {
    stack
        .pop()
        .map(into_abi_value)
        .ok_or_else(|| "stack underflow".to_string())
}

fn push_abi_value(stack: &mut Vec<StackValue>, value: AbiStackValue) -> Result<(), String> {
    stack.push(match value {
        AbiStackValue::Integer(value) => StackValue::Integer(value),
        AbiStackValue::BigInteger(value) => StackValue::BigInteger(value),
        AbiStackValue::ByteString(value) => StackValue::ByteString(value),
        AbiStackValue::Boolean(value) => StackValue::Boolean(value),
        AbiStackValue::Null => StackValue::Null,
        AbiStackValue::Pointer(value) => {
            StackValue::Pointer(usize::try_from(value).map_err(|_| {
                "semantic opcode produced a pointer outside the interpreter range".to_string()
            })?)
        }
        AbiStackValue::Buffer(_)
        | AbiStackValue::Array(_)
        | AbiStackValue::Struct(_)
        | AbiStackValue::Map(_)
        | AbiStackValue::Interop(_)
        | AbiStackValue::Iterator(_) => {
            return Err("semantic numeric opcode produced a compound result".to_string())
        }
    });
    Ok(())
}

fn into_abi_value(value: StackValue) -> AbiStackValue {
    match value {
        StackValue::Integer(value) => AbiStackValue::Integer(value),
        StackValue::BigInteger(value) => AbiStackValue::BigInteger(value),
        StackValue::ByteString(value) => AbiStackValue::ByteString(value),
        StackValue::Boolean(value) => AbiStackValue::Boolean(value),
        StackValue::Pointer(value) => AbiStackValue::Pointer(value as i64),
        StackValue::Array(_, items) => {
            AbiStackValue::Array(items.into_iter().map(into_abi_value).collect())
        }
        StackValue::Struct(_, items) => {
            AbiStackValue::Struct(items.into_iter().map(into_abi_value).collect())
        }
        StackValue::Map(_, items) => AbiStackValue::Map(
            items
                .into_iter()
                .map(|(key, value)| (into_abi_value(key), into_abi_value(value)))
                .collect(),
        ),
        StackValue::Buffer(_, bytes) => AbiStackValue::Buffer(bytes),
        StackValue::Interop(value) => AbiStackValue::Interop(value),
        StackValue::Iterator(value) => AbiStackValue::Iterator(value),
        StackValue::Null => AbiStackValue::Null,
    }
}
