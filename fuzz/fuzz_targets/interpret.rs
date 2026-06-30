#![no_main]
//! Fuzz the consensus interpreter on arbitrary attacker-controlled bytecode.
//!
//! Oracle: the VM must never PANIC (a returned `Err` / `Ok(Fault)` is fine, a
//! Rust panic/abort is a bug), and any terminal result must satisfy the
//! consensus invariants — eval stack within MaxStackSize, integers within
//! Integer.MaxSize, byte items within MaxItemSize.

use libfuzzer_sys::fuzz_target;
use neo_vm_rs::{
    DEFAULT_MAX_STACK_DEPTH, ExecutionResult, MAX_ITEM_SIZE, MAX_SCRIPT_SIZE, StackValue,
    SyscallProvider, interpret_with_stack_and_syscalls,
};
use std::collections::BTreeSet;

/// Step limit. The pure interpreter is intentionally GAS-LESS (gas is metered
/// externally by the PolkaVM host in production, which bounds runaway loops), so
/// a `JMP`-to-self would spin forever and a gas-less allocating loop would pile
/// up transient allocations (RSS grows with the step count even though the final
/// stack stays bounded by MaxStackSize). We bound execution via the
/// per-instruction hook so both become a clean `Err` fault instead of a fuzz
/// timeout/OOM — WITHOUT altering the consensus VM. 100k opcodes dwarfs any real
/// script (and any state worth reaching for crash-finding) while keeping peak
/// RSS and exec time per input comfortably bounded.
const STEP_LIMIT: u64 = 100_000;

/// Deterministic syscall provider: returns `Ok` from `syscall` without touching
/// the stack so SYSCALL/CALLT exercise the dispatch path with no external
/// effects (CALLT routes through the trait-default `callt` -> `syscall`), and
/// caps total executed instructions via `on_instruction`.
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

    fn syscall(
        &mut self,
        _api: u32,
        _ip: usize,
        _stack: &mut Vec<StackValue>,
    ) -> Result<(), String> {
        Ok(())
    }
}

fuzz_target!(|data: &[u8]| {
    // The VM rejects scripts above MAX_SCRIPT_SIZE in validation; skip oversized
    // inputs so the budget is spent on executable scripts.
    if data.len() > MAX_SCRIPT_SIZE {
        return;
    }
    let mut host = FuzzSyscalls { steps: 0 };
    if let Ok(result) = interpret_with_stack_and_syscalls(data, Vec::new(), &mut host) {
        check_invariants(&result);
    }
});

fn check_invariants(result: &ExecutionResult) {
    // The eval stack must stay within MaxStackSize; the VM faults uncatchably
    // past the limit, so a terminal result can never exceed it.
    assert!(
        result.stack.len() <= DEFAULT_MAX_STACK_DEPTH,
        "stack depth {} exceeds MaxStackSize {}",
        result.stack.len(),
        DEFAULT_MAX_STACK_DEPTH
    );

    // Walk every reachable value. Cycle-safe (main's compound StackValue can
    // self-reference via SETITEM) via a visited-id set, and budget-bounded so a
    // pathological-but-legal object graph can't make the CHECKER itself run away.
    let mut visited: BTreeSet<u64> = BTreeSet::new();
    let mut work: Vec<&StackValue> = result.stack.iter().collect();
    let mut budget: u32 = 5_000_000;
    while let Some(value) = work.pop() {
        budget -= 1;
        if budget == 0 {
            return;
        }
        match value {
            StackValue::BigInteger(bytes) => assert!(
                bytes.len() <= 32,
                "BigInteger {} bytes exceeds Integer.MaxSize 32",
                bytes.len()
            ),
            StackValue::ByteString(bytes) => assert!(
                bytes.len() <= MAX_ITEM_SIZE,
                "ByteString {} bytes exceeds MaxItemSize {}",
                bytes.len(),
                MAX_ITEM_SIZE
            ),
            StackValue::Buffer(_, bytes) => assert!(
                bytes.len() <= MAX_ITEM_SIZE,
                "Buffer {} bytes exceeds MaxItemSize {}",
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
                    for (key, val) in entries {
                        work.push(key);
                        work.push(val);
                    }
                }
            }
            _ => {}
        }
    }
}
