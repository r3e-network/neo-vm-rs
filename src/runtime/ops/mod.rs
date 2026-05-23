//! Runtime-level opcode adapters shared by compiled VM runtimes.
//!
//! The pure modules in `semantics::{arithmetic, comparison, conversion,
//! collections, splice, stack, stack_shape}` own the VM rules and stack
//! shapes. This module only adapts those rules to a host runtime's stack/fault
//! interface.

use alloc::string::String;

use crate::{runtime::RuntimeStack, semantics::stack_shape::ValueStack, StackValue};

pub mod arithmetic;
pub mod bytes;
pub mod collections;
pub mod comparison;
pub mod conversion;
pub mod stack;

impl<R: RuntimeStack + ?Sized> ValueStack for R {
    fn pop_value(&mut self) -> Result<StackValue, String> {
        Ok(RuntimeStack::pop_value(self))
    }

    fn push_value(&mut self, value: StackValue) -> Result<(), String> {
        RuntimeStack::push_value(self, value);
        Ok(())
    }
}

pub(crate) fn apply_or_fault<R, F>(runtime: &mut R, apply: F)
where
    R: RuntimeStack + ?Sized,
    F: FnOnce(&mut R) -> Result<(), String>,
{
    if let Err(message) = apply(runtime) {
        runtime.fault(&message);
    }
}
