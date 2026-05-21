# neo-vm-rs

`neo-vm-rs` is the shared NeoVM semantics crate used by the Neo N4 Rust execution
profile crates. It intentionally stays small, deterministic, and `no_std +
alloc` compatible so PolkaVM/RISC-V runtimes and zkVM/proving runtimes can reuse
the same VM-facing types without inheriting each other's host or prover stack.

## Scope

This crate owns the common semantics that must stay identical across N4 VM
consumers:

- canonical NeoVM opcode metadata and byte decoding
- shared execution result and VM state reporting types
- shared stack value representation used at ABI/proof boundaries
- shared Neo syscall hashing and fixed argument-count metadata
- shared execution limit constants

It does not contain a full interpreter, host runtime, storage engine, verifier,
or prover. Those remain in the consuming crates:

- `neo-riscv-vm`: canonical N4 Layer-2 NeoVM2/RISC-V execution profile
- `neo-zkvm`: proof-oriented zkVM integration and verifier tooling

## Validation

Run the same commands used by CI:

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --locked --all-targets
cargo check --locked --no-default-features --all-targets
cargo bench --locked --no-run
```

The test suite covers canonical opcode byte acceptance and gap rejection,
metadata round trips, syscall hash vectors, stack value conversion semantics,
and serde wire compatibility for execution results.

The benchmark target compiles Criterion benchmarks for opcode decode,
metadata lookup, syscall helpers, and stack value conversions. CI compiles the
benchmarks with `--no-run`; performance tracking jobs can run the same target on
dedicated hardware.
