use super::super::helpers::*;
use super::super::opcodes::*;
use super::super::runtime_types::StackValue;
use super::control::Dispatch;
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use num_bigint::BigInt;
use num_traits::{ToPrimitive, Zero};

#[inline]
pub(super) fn execute(opcode: u8, stack: &mut Vec<StackValue>) -> Result<Dispatch, String> {
    match opcode {
        // =============================================================================
        // BITWISE LOGIC OPERATIONS (0x90-0x98)
        // =============================================================================
        INVERT => {
            let value = pop_item(stack)?;
            match value {
                StackValue::Integer(v) => stack.push(StackValue::Integer(!v)),
                StackValue::BigInteger(v) | StackValue::ByteString(v) => {
                    let n = decode_signed_le_bytes_bigint(&v)?;
                    stack.push(numeric_result_bigint(!n, "integer overflow for INVERT")?);
                }
                StackValue::Boolean(v) => stack.push(StackValue::Integer(if v { -2 } else { -1 })),
                _ => return Err("INVERT expects an integer or boolean".to_string()),
            }
        }
        AND => {
            let right = pop_item(stack)?;
            let left = pop_item(stack)?;
            stack.push(bitwise_result(&left, &right, |l, r| l & r)?);
        }
        OR => {
            let right = pop_item(stack)?;
            let left = pop_item(stack)?;
            stack.push(bitwise_result(&left, &right, |l, r| l | r)?);
        }
        XOR => {
            let right = pop_item(stack)?;
            let left = pop_item(stack)?;
            stack.push(bitwise_result(&left, &right, |l, r| l ^ r)?);
        }
        NUMEQUAL => {
            let right = pop_item(stack)?;
            let left = pop_item(stack)?;
            stack.push(StackValue::Boolean(num_equal(&left, &right)?));
        }
        NUMNOTEQUAL => {
            let right = pop_item(stack)?;
            let left = pop_item(stack)?;
            stack.push(StackValue::Boolean(!num_equal(&left, &right)?));
        }
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
        LT => {
            let comparison = pop_bigint_pair_allowing_null_false(stack)?;
            stack.push(StackValue::Boolean(
                matches!(comparison, Some((left, right)) if left < right),
            ));
        }
        LE => {
            let comparison = pop_bigint_pair_allowing_null_false(stack)?;
            stack.push(StackValue::Boolean(
                matches!(comparison, Some((left, right)) if left <= right),
            ));
        }
        GT => {
            let comparison = pop_bigint_pair_allowing_null_false(stack)?;
            stack.push(StackValue::Boolean(
                matches!(comparison, Some((left, right)) if left > right),
            ));
        }
        GE => {
            let comparison = pop_bigint_pair_allowing_null_false(stack)?;
            stack.push(StackValue::Boolean(
                matches!(comparison, Some((left, right)) if left >= right),
            ));
        }
        // =============================================================================
        // ARITHMETIC OPERATIONS (0x99-0xbb)
        // =============================================================================
        SIGN => {
            let value = pop_numeric_bigint(stack)?;
            stack.push(StackValue::Integer(bigint_sign(&value)));
        }
        ABS => {
            let value = pop_numeric_bigint(stack)?;
            stack.push(numeric_result_bigint(
                bigint_abs(value),
                "integer overflow for ABS",
            )?);
        }
        NEGATE => {
            let value = pop_numeric_bigint(stack)?;
            stack.push(numeric_result_bigint(
                -value,
                "integer overflow for NEGATE",
            )?);
        }
        ADD => {
            let right = pop_numeric_bigint(stack)?;
            let left = pop_numeric_bigint(stack)?;
            stack.push(numeric_result_bigint(
                left + right,
                "integer overflow for ADD",
            )?);
        }
        INC => {
            let value = pop_numeric_bigint(stack)?;
            stack.push(numeric_result_bigint(
                value + BigInt::from(1),
                "integer overflow for INC",
            )?);
        }
        SUB => {
            let right = pop_numeric_bigint(stack)?;
            let left = pop_numeric_bigint(stack)?;
            stack.push(numeric_result_bigint(
                left - right,
                "integer overflow for SUB",
            )?);
        }
        POW => {
            let exponent = pop_numeric_bigint(stack)?;
            if exponent < BigInt::from(0) {
                return Err("negative exponent for POW".to_string());
            }
            let exponent = exponent
                .to_u32()
                .ok_or_else(|| "exponent too large for POW".to_string())?;
            let base = pop_numeric_bigint(stack)?;
            stack.push(numeric_result_bigint(
                base.pow(exponent),
                "integer overflow for POW",
            )?);
        }
        SQRT => {
            let value = pop_numeric_bigint(stack)?;
            if value < BigInt::from(0) {
                return Err("negative value for SQRT".to_string());
            }
            stack.push(numeric_result_bigint(
                value.sqrt(),
                "integer overflow for SQRT",
            )?);
        }
        MODMUL => {
            let modulus = pop_numeric_bigint(stack)?;
            if modulus.is_zero() {
                return Err("division by zero for MODMUL".to_string());
            }
            let right = pop_numeric_bigint(stack)?;
            let left = pop_numeric_bigint(stack)?;
            stack.push(numeric_result_bigint(
                (left * right) % modulus,
                "integer overflow for MODMUL",
            )?);
        }
        MODPOW => {
            let modulus = pop_numeric_bigint(stack)?;
            if modulus.is_zero() {
                return Err("division by zero for MODPOW".to_string());
            }
            let exponent = pop_numeric_bigint(stack)?;
            let base = pop_numeric_bigint(stack)?;
            stack.push(numeric_result_bigint(
                mod_pow_bigint(base, exponent, modulus)?,
                "integer overflow for MODPOW",
            )?);
        }
        SHL => {
            let shift = pop_shift_count(stack)?;
            let value = pop_item(stack)?;
            if !(0..=256).contains(&shift) {
                return Err("shift count out of range for SHL".to_string());
            }
            if shift == 0 {
                stack.push(value);
            } else {
                stack.push(shift_value_from_item(value)?.shift_left(shift as u32)?);
            }
        }
        SHR => {
            let shift = pop_shift_count(stack)?;
            let value = pop_item(stack)?;
            if !(0..=256).contains(&shift) {
                return Err("shift count out of range for SHR".to_string());
            }
            if shift == 0 {
                stack.push(value);
            } else {
                stack.push(shift_value_from_item(value)?.shift_right(shift as u32)?);
            }
        }
        NOT => {
            // NeoVM: NOT converts to boolean via integer path.
            // ByteString > 32 bytes cannot be converted to integer → FAULT.
            let item = pop_item(stack)?;
            let b = item_to_boolean_strict(&item)?;
            stack.push(StackValue::Boolean(!b));
        }
        MUL => {
            let right = pop_numeric_bigint(stack)?;
            let left = pop_numeric_bigint(stack)?;
            stack.push(numeric_result_bigint(
                left * right,
                "integer overflow for MUL",
            )?);
        }
        DIV => {
            let right = pop_numeric_bigint(stack)?;
            let left = pop_numeric_bigint(stack)?;
            if right == BigInt::from(0) {
                return Err("division by zero for DIV".to_string());
            }
            stack.push(numeric_result_bigint(
                left / right,
                "integer overflow for DIV",
            )?);
        }
        MOD => {
            let right = pop_numeric_bigint(stack)?;
            let left = pop_numeric_bigint(stack)?;
            if right == BigInt::from(0) {
                return Err("division by zero for MOD".to_string());
            }
            stack.push(numeric_result_bigint(
                left % right,
                "integer overflow for MOD",
            )?);
        }
        DEC => {
            let value = pop_numeric_bigint(stack)?;
            stack.push(numeric_result_bigint(
                value - BigInt::from(1),
                "integer overflow for DEC",
            )?);
        }
        BOOLAND => {
            let right = pop_boolean(stack)?;
            let left = pop_boolean(stack)?;
            stack.push(StackValue::Boolean(left && right));
        }
        BOOLOR => {
            let right = pop_boolean(stack)?;
            let left = pop_boolean(stack)?;
            stack.push(StackValue::Boolean(left || right));
        }
        NZ => {
            let value = pop_numeric_bigint(stack)?;
            stack.push(StackValue::Boolean(!value.is_zero()));
        }
        MIN => {
            let right = pop_numeric_bigint(stack)?;
            let left = pop_numeric_bigint(stack)?;
            stack.push(numeric_result_bigint(
                if left < right { left } else { right },
                "integer overflow for MIN",
            )?);
        }
        MAX => {
            let right = pop_numeric_bigint(stack)?;
            let left = pop_numeric_bigint(stack)?;
            stack.push(numeric_result_bigint(
                if left > right { left } else { right },
                "integer overflow for MAX",
            )?);
        }
        WITHIN => {
            let upper = pop_numeric_bigint(stack)?;
            let lower = pop_numeric_bigint(stack)?;
            let value = pop_numeric_bigint(stack)?;
            stack.push(StackValue::Boolean(value >= lower && value < upper));
        }
        _ => unreachable!("opcode routed to numeric_ops: 0x{opcode:02x}"),
    }
    Ok(Dispatch::Fallthrough)
}
