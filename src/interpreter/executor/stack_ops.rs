use super::super::helpers::{peek_item, pop_integer, pop_item};
use super::super::opcodes::*;
use super::super::runtime_types::StackValue;
use super::control::Dispatch;
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

#[inline]
pub(super) fn execute(opcode: u8, stack: &mut Vec<StackValue>) -> Result<Dispatch, String> {
    match opcode {
        // =============================================================================
        // STACK OPERATIONS (0x43-0x54)
        // =============================================================================
        DEPTH => {
            stack.push(StackValue::Integer(stack.len() as i64));
        }
        DROP => {
            pop_item(stack)?;
        }
        DUP => {
            let value = peek_item(stack)?;
            stack.push(value);
        }
        SWAP => {
            if stack.len() < 2 {
                return Err("stack underflow for SWAP".to_string());
            }
            let last = stack.len() - 1;
            stack.swap(last, last - 1);
        }
        NIP => {
            if stack.len() < 2 {
                return Err("stack underflow for NIP".to_string());
            }
            let x1 = stack.pop().expect("guarded by length check");
            stack.pop();
            stack.push(x1);
        }
        OVER => {
            if stack.len() < 2 {
                return Err("stack underflow for OVER".to_string());
            }
            let x1 = stack[stack.len() - 2].clone();
            stack.push(x1);
        }
        PICK => {
            let n = pop_integer(stack)?;
            if n < 0 {
                return Err("negative index for PICK".to_string());
            }
            let n = n as usize;
            if n >= stack.len() {
                return Err("index out of range for PICK".to_string());
            }
            let item = stack[stack.len() - 1 - n].clone();
            stack.push(item);
        }
        ROT => {
            // Rotate top 3 items: bottom moves to top, top and second shift down
            if stack.len() < 3 {
                return Err("stack underflow for ROT".to_string());
            }
            let n = stack.len() - 1;
            stack.swap(n - 2, n - 1);
            stack.swap(n - 1, n);
        }
        ROLL => {
            let n = pop_integer(stack)?;
            if n < 0 {
                return Err("negative index for ROLL".to_string());
            }
            let n = n as usize;
            if n >= stack.len() {
                return Err("index out of range for ROLL".to_string());
            }
            let idx = stack.len() - 1 - n;
            let item = stack.remove(idx);
            stack.push(item);
        }
        REVERSE3 => {
            // Reverse top 3 items: [a, b, c] where c is top → [c, b, a] where a is top
            if stack.len() < 3 {
                return Err("stack underflow for REVERSE3".to_string());
            }
            let n = stack.len() - 1;
            stack.swap(n - 2, n);
        }
        REVERSE4 => {
            // Reverse top 4 items: [a, b, c, d] where d is top → [d, c, b, a] where a is top
            if stack.len() < 4 {
                return Err("stack underflow for REVERSE4".to_string());
            }
            let n = stack.len() - 1;
            stack.swap(n - 3, n);
            stack.swap(n - 2, n - 1);
        }
        REVERSEN => {
            let n = pop_integer(stack)?;
            if n < 0 {
                return Err("negative count for REVERSEN".to_string());
            }
            let n = n as usize;
            if n > stack.len() {
                return Err("REVERSEN count exceeds stack depth".to_string());
            }
            let start = stack.len() - n;
            stack[start..].reverse();
        }
        TUCK => {
            if stack.len() < 2 {
                return Err("stack underflow for TUCK".to_string());
            }
            let x = stack[stack.len() - 1].clone();
            stack.insert(stack.len() - 2, x);
        }
        XDROP => {
            let n = pop_integer(stack)?;
            if n < 0 {
                return Err("negative index for XDROP".to_string());
            }
            let n = n as usize;
            if n >= stack.len() {
                return Err("XDROP index out of range".to_string());
            }
            let idx = stack.len() - 1 - n;
            stack.remove(idx);
        }
        CLEAR => {
            stack.clear();
        }
        _ => unreachable!("opcode routed to stack_ops: 0x{opcode:02x}"),
    }
    Ok(Dispatch::Fallthrough)
}
