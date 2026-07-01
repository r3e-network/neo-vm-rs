#![no_main]
//! Fuzz the interpreter with an arbitrary PRE-BUILT initial evaluation stack —
//! modelling the arguments a contract is invoked with. This reaches the
//! compound/aliasing/reference-counting/clone paths (SETITEM, APPEND, PICKITEM,
//! REMOVE, VALUES, EQUAL over nested Arrays/Maps/Structs/Buffers) that are hard
//! to construct from bytecode alone, where F1 (ReferenceCounter) and the D-5
//! aliasing rules live. Same oracle as `interpret`: never PANIC, and any
//! terminal result satisfies the consensus size invariants.

use libfuzzer_sys::fuzz_target;
use neo_vm_rs::{
    DEFAULT_MAX_STACK_DEPTH, ExecutionResult, MAX_ITEM_SIZE, MAX_SCRIPT_SIZE, StackValue,
    SyscallProvider, interpret_with_stack_and_syscalls, next_stack_item_id,
};
use std::collections::BTreeSet;

const STEP_LIMIT: u64 = 100_000;

struct FuzzSyscalls {
    steps: u64,
}

impl SyscallProvider for FuzzSyscalls {
    fn on_instruction(&mut self, _opcode: u8) -> Result<(), String> {
        self.steps += 1;
        if self.steps > STEP_LIMIT {
            return Err("fuzz step limit exceeded".into());
        }
        Ok(())
    }
    fn syscall(&mut self, _a: u32, _i: usize, _s: &mut Vec<StackValue>) -> Result<(), String> {
        Ok(())
    }
}

/// A cursor over the fuzz input used to synthesize an initial stack, then hand
/// the remainder to the interpreter as the script.
struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn byte(&mut self) -> u8 {
        let b = self.data.get(self.pos).copied().unwrap_or(0);
        self.pos = self.pos.saturating_add(1);
        b
    }
    fn take(&mut self, n: usize) -> Vec<u8> {
        let end = self.pos.saturating_add(n).min(self.data.len());
        let out = self.data[self.pos.min(self.data.len())..end].to_vec();
        self.pos = end;
        out
    }
}

/// Build one StackValue, bounded by `depth` (nesting) so the synthesized graph
/// stays small. Compounds get global-band ids (disjoint from the interpreter's
/// per-execution CompoundIds), matching how the host imports argument compounds.
fn build_value(c: &mut Cursor, depth: u8) -> StackValue {
    match c.byte() % 8 {
        0 => {
            let mut bytes = [0u8; 8];
            for slot in bytes.iter_mut() {
                *slot = c.byte();
            }
            StackValue::Integer(i64::from_le_bytes(bytes))
        }
        1 => {
            let len = (c.byte() % 64) as usize;
            StackValue::ByteString(c.take(len))
        }
        2 => StackValue::Boolean(c.byte() & 1 == 1),
        3 => StackValue::Null,
        4 => {
            let len = (c.byte() % 32) as usize;
            StackValue::BigInteger(c.take(len))
        }
        5 if depth > 0 => {
            let n = (c.byte() % 5) as usize;
            let mut items = Vec::with_capacity(n);
            for _ in 0..n {
                items.push(build_value(c, depth - 1));
            }
            // Array or Struct (both id + Vec).
            if c.byte() & 1 == 0 {
                StackValue::Array(next_stack_item_id(), items)
            } else {
                StackValue::Struct(next_stack_item_id(), items)
            }
        }
        6 if depth > 0 => {
            let n = (c.byte() % 4) as usize;
            let mut entries = Vec::with_capacity(n);
            for _ in 0..n {
                let key = build_primitive_key(c);
                let value = build_value(c, depth - 1);
                entries.push((key, value));
            }
            StackValue::Map(next_stack_item_id(), entries)
        }
        7 => {
            let len = (c.byte() % 64) as usize;
            StackValue::Buffer(next_stack_item_id(), c.take(len))
        }
        _ => StackValue::Null,
    }
}

/// Map keys must be PrimitiveType (Integer/Boolean/ByteString).
fn build_primitive_key(c: &mut Cursor) -> StackValue {
    match c.byte() % 3 {
        0 => StackValue::Integer(i64::from(c.byte())),
        1 => StackValue::Boolean(c.byte() & 1 == 1),
        _ => {
            let len = (c.byte() % 16) as usize;
            StackValue::ByteString(c.take(len))
        }
    }
}

fuzz_target!(|data: &[u8]| {
    let mut c = Cursor { data, pos: 0 };
    let n_items = (c.byte() % 5) as usize; // <= 4 argument values
    let initial: Vec<StackValue> = (0..n_items).map(|_| build_value(&mut c, 3)).collect();
    let script = &data[c.pos.min(data.len())..];
    if script.len() > MAX_SCRIPT_SIZE {
        return;
    }
    let mut host = FuzzSyscalls { steps: 0 };
    if let Ok(result) = interpret_with_stack_and_syscalls(script, initial, &mut host) {
        check_invariants(&result);
    }
});

fn check_invariants(result: &ExecutionResult) {
    assert!(
        result.stack.len() <= DEFAULT_MAX_STACK_DEPTH,
        "stack depth {} exceeds MaxStackSize {}",
        result.stack.len(),
        DEFAULT_MAX_STACK_DEPTH
    );
    let mut visited: BTreeSet<u64> = BTreeSet::new();
    let mut work: Vec<&StackValue> = result.stack.iter().collect();
    let mut budget: u32 = 5_000_000;
    while let Some(value) = work.pop() {
        budget -= 1;
        if budget == 0 {
            return;
        }
        match value {
            StackValue::BigInteger(bytes) => {
                assert!(bytes.len() <= 32, "BigInteger {} bytes > 32", bytes.len())
            }
            StackValue::ByteString(bytes) | StackValue::Buffer(_, bytes) => assert!(
                bytes.len() <= MAX_ITEM_SIZE,
                "byte item {} > MaxItemSize {}",
                bytes.len(),
                MAX_ITEM_SIZE
            ),
            StackValue::Array(id, items) | StackValue::Struct(id, items) => {
                if visited.insert(*id) {
                    work.extend(items.iter());
                }
            }
            StackValue::Map(id, entries) => {
                if visited.insert(*id) {
                    for (k, v) in entries {
                        work.push(k);
                        work.push(v);
                    }
                }
            }
            _ => {}
        }
    }
}
