//! Runtime-level stack opcode adapters.

use crate::{runtime::RuntimeStack, StackValue};

pub fn drop_top<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let _ = runtime.pop_value();
}

pub fn dup<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let top = runtime
        .stack_values()
        .last()
        .expect("stack underflow: dup on empty stack")
        .clone();
    runtime.push_value(top);
}

pub fn swap<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let stack = runtime.stack_values_mut();
    let len = stack.len();
    assert!(len >= 2, "stack underflow: swap requires at least 2 items");
    stack.swap(len - 1, len - 2);
}

pub fn nip<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let stack = runtime.stack_values_mut();
    let len = stack.len();
    assert!(len >= 2, "stack underflow: nip requires at least 2 items");
    stack.remove(len - 2);
}

pub fn xdrop<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let index = runtime.pop_i64();
    if index < 0 {
        runtime.fault("XDROP: negative index");
        return;
    }
    #[allow(clippy::cast_sign_loss)]
    let index = index as usize;
    let len = runtime.stack_values().len();
    if index >= len {
        runtime.fault("XDROP: index out of range");
        return;
    }
    runtime.stack_values_mut().remove(len - 1 - index);
}

pub fn over<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let stack = runtime.stack_values();
    let len = stack.len();
    assert!(len >= 2, "stack underflow: over requires at least 2 items");
    runtime.push_value(stack[len - 2].clone());
}

pub fn pick<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let index = runtime.pop_i64();
    if index < 0 {
        runtime.fault("pick: negative index");
        return;
    }
    #[allow(clippy::cast_sign_loss)]
    pick_n(runtime, index as usize);
}

pub fn pick_n<R: RuntimeStack + ?Sized>(runtime: &mut R, index: usize) {
    let stack = runtime.stack_values();
    let len = stack.len();
    if index >= len {
        runtime.fault(&alloc::format!("pick({index}): stack underflow"));
        return;
    }
    runtime.push_value(stack[len - 1 - index].clone());
}

pub fn tuck<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let stack = runtime.stack_values_mut();
    let len = stack.len();
    assert!(len >= 2, "stack underflow: tuck requires at least 2 items");
    let top = stack[len - 1].clone();
    stack.insert(len - 2, top);
}

pub fn rot<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let stack = runtime.stack_values_mut();
    let len = stack.len();
    assert!(len >= 3, "stack underflow: rot requires at least 3 items");
    let value = stack.remove(len - 3);
    stack.push(value);
}

pub fn roll<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let index = runtime.pop_i64();
    if index < 0 {
        runtime.fault("roll: negative index");
        return;
    }
    #[allow(clippy::cast_sign_loss)]
    let index = index as usize;
    let len = runtime.stack_values().len();
    if index >= len {
        runtime.fault(&alloc::format!("roll({index}): stack underflow"));
        return;
    }
    let value = runtime.stack_values_mut().remove(len - 1 - index);
    runtime.push_value(value);
}

pub fn reverse3<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let stack = runtime.stack_values_mut();
    let len = stack.len();
    assert!(
        len >= 3,
        "stack underflow: reverse3 requires at least 3 items"
    );
    stack[len - 3..].reverse();
}

pub fn reverse4<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let stack = runtime.stack_values_mut();
    let len = stack.len();
    assert!(
        len >= 4,
        "stack underflow: reverse4 requires at least 4 items"
    );
    stack[len - 4..].reverse();
}

pub fn reverse_n<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let count = runtime.pop_i64();
    if count < 0 {
        runtime.fault("reverse_n: negative count");
        return;
    }
    #[allow(clippy::cast_sign_loss)]
    let count = count as usize;
    let len = runtime.stack_values().len();
    if count > len {
        runtime.fault(&alloc::format!("reverse_n({count}): stack underflow"));
        return;
    }
    if count > 1 {
        runtime.stack_values_mut()[len - count..].reverse();
    }
}

pub fn depth<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    let depth = runtime.stack_values().len() as i64;
    runtime.push_value(StackValue::Integer(depth));
}

pub fn clear<R: RuntimeStack + ?Sized>(runtime: &mut R) {
    runtime.stack_values_mut().clear();
}
