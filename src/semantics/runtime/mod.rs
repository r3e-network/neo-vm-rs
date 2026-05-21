//! Runtime-level opcode adapters shared by compiled VM runtimes.
//!
//! The pure modules in `semantics::{arithmetic, comparison, conversion,
//! collections}` own the VM rules. This module owns the common stack operation
//! shape around those rules: pop operands, call the shared rule, push results,
//! and report faults through a host runtime adapter.

use crate::StackValue;

pub mod arithmetic;
pub mod byte_ops;
pub mod collections;
pub mod comparison;
pub mod conversion;
pub mod stack;

/// Minimal stack/fault interface required by shared runtime opcode adapters.
pub trait RuntimeStack {
    /// Pop one value from the evaluation stack.
    fn pop_value(&mut self) -> StackValue;

    /// Push one value to the evaluation stack.
    fn push_value(&mut self, value: StackValue);

    /// Borrow the current top value mutably for in-place collection opcodes.
    fn top_value_mut(&mut self) -> Option<&mut StackValue>;

    /// Borrow the full evaluation stack.
    fn stack_values(&self) -> &[StackValue];

    /// Borrow the full evaluation stack mutably.
    fn stack_values_mut(&mut self) -> &mut alloc::vec::Vec<StackValue>;

    /// Put the runtime into a faulted state.
    fn fault(&mut self, message: &str);

    /// Pop an integer-compatible value that fits in `i64`.
    ///
    /// This intentionally preserves the existing compiled-runtime behavior:
    /// invalid generated stacks are programmer/runtime faults and panic in the
    /// same way the previous per-runtime wrappers did.
    fn pop_i64(&mut self) -> i64 {
        let value = self.pop_value();
        value
            .to_i128()
            .and_then(|integer| i64::try_from(integer).ok())
            .unwrap_or_else(|| {
                panic!(
                    "expected integer-compatible StackValue fitting i64, got {:?}",
                    value
                )
            })
    }

    /// Push a compact integer result.
    fn push_i64(&mut self, value: i64) {
        self.push_value(StackValue::Integer(value));
    }

    /// Push a boolean result.
    fn push_bool(&mut self, value: bool) {
        self.push_value(StackValue::Boolean(value));
    }

    /// Pop a value and coerce it through NeoVM truthiness rules.
    fn pop_bool_value(&mut self) -> bool {
        self.pop_value().to_bool()
    }
}

pub(crate) fn push_i64_result<R: RuntimeStack + ?Sized>(
    runtime: &mut R,
    result: Result<i64, &'static str>,
) {
    match result {
        Ok(value) => runtime.push_i64(value),
        Err(message) => runtime.fault(message),
    }
}

pub(crate) fn push_value_result<R: RuntimeStack + ?Sized>(
    runtime: &mut R,
    result: Result<StackValue, alloc::string::String>,
) {
    match result {
        Ok(value) => runtime.push_value(value),
        Err(message) => runtime.fault(&message),
    }
}
