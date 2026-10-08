//! Reference-entry operand-domain checks before the shared legacy numeric algorithms run.
//! Neo.VM 3.9 ByteString.GetBoolean/GetInteger are bounded by Integer.MaxSize; implicit
//! Buffer.GetInteger is unsupported even though explicit Buffer.ConvertTo(Integer) is allowed.
use super::super::{opcodes::*, runtime_types::StackValue};
use alloc::string::{String, ToString};

fn argument(stack: &[StackValue], from_top: usize) -> Result<&StackValue, String> {
    stack
        .len()
        .checked_sub(from_top + 1)
        .and_then(|i| stack.get(i))
        .ok_or_else(|| "stack underflow".to_string())
}
fn boolean(value: &StackValue) -> Result<(), String> {
    if matches!(value,StackValue::ByteString(bytes)|StackValue::BigInteger(bytes) if bytes.len()>32)
    {
        return Err("ByteString Boolean conversion exceeds Integer.MaxSize".to_string());
    }
    Ok(())
}
fn integer(value: &StackValue) -> Result<(), String> {
    match value {
        StackValue::Integer(_) | StackValue::Boolean(_) => Ok(()),
        StackValue::BigInteger(bytes) | StackValue::ByteString(bytes) if bytes.len() <= 32 => {
            Ok(())
        }
        _ => Err("operand does not support bounded GetInteger".to_string()),
    }
}

/// Validate the precise operand domains without changing legacy entry points or opcode costs.
pub(super) fn validate(
    opcode: u8,
    script: &[u8],
    ip: usize,
    stack: &[StackValue],
) -> Result<(), String> {
    let integer_positions: &[usize] = match opcode {
        INVERT | SIGN | ABS | NEGATE | INC | DEC | SQRT | NZ | XDROP | PICK | ROLL | REVERSEN
        | PACK | PACKSTRUCT | PACKMAP | NEWARRAY | NEWARRAY_T | NEWSTRUCT | NEWBUFFER | LEFT
        | RIGHT => &[0],
        AND | OR | XOR | ADD | SUB | MUL | DIV | MOD | POW | NUMEQUAL | NUMNOTEQUAL | MIN | MAX
        | JMPEQ | JMPEQ_L | JMPNE | JMPNE_L | JMPGT | JMPGT_L | JMPGE | JMPGE_L | JMPLT
        | JMPLT_L | JMPLE | JMPLE_L | SUBSTR => &[0, 1],
        MODMUL | MODPOW | WITHIN => &[0, 1, 2],
        MEMCPY => &[0, 1, 3],
        _ => &[],
    };
    for index in integer_positions {
        integer(argument(stack, *index)?)?;
    }
    match opcode {
        LT | LE | GT | GE => {
            let right = argument(stack, 0)?;
            let left = argument(stack, 1)?;
            if !matches!(left, StackValue::Null) && !matches!(right, StackValue::Null) {
                integer(left)?;
                integer(right)?;
            }
        }
        SHL | SHR => {
            let shift = argument(stack, 0)?;
            integer(shift)?;
            let zero = match shift {
                StackValue::Integer(n) => *n == 0,
                StackValue::Boolean(n) => !*n,
                StackValue::ByteString(bytes) | StackValue::BigInteger(bytes) => {
                    bytes.iter().all(|b| *b == 0)
                }
                _ => unreachable!("integer domain checked"),
            };
            // Core returns immediately after popping a zero shift without coercing the value.
            if !zero {
                integer(argument(stack, 1)?)?;
            }
        }
        JMPIF | JMPIF_L | JMPIFNOT | JMPIFNOT_L | ASSERT | NOT => boolean(argument(stack, 0)?)?,
        ASSERTMSG => boolean(argument(stack, 1)?)?,
        BOOLAND | BOOLOR => {
            boolean(argument(stack, 0)?)?;
            boolean(argument(stack, 1)?)?;
        }
        CONVERT if script.get(ip + 1) == Some(&(crate::StackItemType::Boolean as u8)) => {
            boolean(argument(stack, 0)?)?
        }
        EQUAL | NOTEQUAL => {
            // Equals is dispatched on the left operand. A non-ByteString left operand does not
            // inspect an unrelated right ByteString's size (Struct applies its own shared budget).
            if let StackValue::ByteString(left) = argument(stack, 1)? {
                if left.len() > 65535 {
                    return Err("ByteString exceeds maximum comparable size".to_string());
                }
                if matches!(argument(stack,0)?,StackValue::ByteString(right) if right.len()>65535) {
                    return Err("ByteString exceeds maximum comparable size".to_string());
                }
            }
        }
        _ => {}
    }
    Ok(())
}
