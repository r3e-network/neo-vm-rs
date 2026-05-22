# neo-vm-rs

`neo-vm-rs` is the shared NeoVM semantics and interpreter crate used by the Neo
N4 Rust execution profile crates. It stays deterministic and `no_std + alloc`
compatible so PolkaVM/RISC-V runtimes and zkVM/proving runtimes can reuse the
same VM-facing types and execution behavior without inheriting each other's host
or prover stack.

## Scope

This crate owns the common semantics that must stay identical across N4 VM
consumers:

- canonical NeoVM opcode metadata and byte decoding
- shared execution result and VM state reporting types
- shared stack value representation used at ABI/proof boundaries
- shared Neo syscall hashing and fixed argument-count metadata
- shared execution limit constants
- the canonical NeoVM2 interpreter entry points used by the RISC-V guest facade

It does not contain a host runtime, storage engine, verifier, or prover. Those
remain in the consuming crates:

- `neo-riscv-vm`: canonical N4 Layer-2 NeoVM2/RISC-V execution profile
- `neo-zkvm`: proof-oriented zkVM integration and verifier tooling

## Layout

- `src/vm`: opcode metadata and execution constants
- `src/abi`: stack values and execution result wire types
- `src/interpreter`: no-std NeoVM2 interpreter, retained-state helpers, and
  host syscall trait
- `src/host`: syscall hash and stack argument metadata

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
serde wire compatibility for execution results, interpreter smoke execution,
slot handling, host syscall delegation, and try/catch exception flow.

It also includes the official Neo.VM VMUT corpus copied from
`neo-project/neo-vm` tag `v3.9.0`, the VM package used by the observed Neo N3
`neo-node v3.9.2` release line. The conformance runner executes 161 upstream
JSON fixture files, validates 723 executable `HALT`/`FAULT` cases, and counts
16 debugger-only `BREAK` cases as intentionally skipped because this crate does
not expose the C# step-debugger API. The runner mirrors Neo.VM's upstream VMUT
assertion behavior: `HALT` cases compare result stacks, while `FAULT` cases
compare final state and do not treat JSON stack fields as asserted behavior.

The benchmark target compiles Criterion benchmarks for opcode decode,
metadata lookup, syscall helpers, and stack value conversions. CI compiles the
benchmarks with `--no-run`; performance tracking jobs can run the same target on
dedicated hardware.

<!-- N4-CRATE-VISUAL-GUIDE:START -->

## Crate Visual Learning Guide

These diagrams are local to this crate. They explain `neo-vm-rs` as an independent unit: where it sits in the Neo N4 stack, which boundary it owns, how its internal workflow runs, and how data moves through it.

| View | Diagram | Source |
| --- | --- | --- |
| Position in Neo N4 | ![Position](docs/figures/position.svg) | [Mermaid](docs/figures/position.mmd) |
| Technical principles | ![Principles](docs/figures/principles.svg) | [Mermaid](docs/figures/principles.mmd) |
| Architecture | ![Architecture](docs/figures/architecture.svg) | [Mermaid](docs/figures/architecture.mmd) |
| Workflow | ![Workflow](docs/figures/workflow.svg) | [Mermaid](docs/figures/workflow.mmd) |
| Dataflow | ![Dataflow](docs/figures/dataflow.svg) | [Mermaid](docs/figures/dataflow.mmd) |

### Role in Neo N4

- **Layer:** Shared VM core
- **Purpose:** Canonical Rust implementation of NeoVM 3.9.x semantics shared by RISC-V and zkVM paths.
- **Primary inputs:** NeoVM bytecode, initial stack, syscall host callbacks
- **Primary outputs:** halt/fault result, final stack, gas/accounting evidence
- **Downstream consumers:** neo-riscv-vm, neo-zkvm, Neo N4 execution core

### Boundary and Responsibilities

- **Owns:** Decode canonical opcodes, Execute stack and state semantics, Expose reusable runtime APIs
- **Consumes:** NeoVM bytecode, initial stack, syscall host callbacks
- **Produces:** halt/fault result, final stack, gas/accounting evidence
- **Used by:** neo-riscv-vm, neo-zkvm, Neo N4 execution core

### Learning Path

1. Start with the position diagram to understand why this crate exists and who calls it.
2. Read the technical principles diagram to identify the invariants and responsibility boundary.
3. Use the architecture diagram to connect public inputs, internal components, dependencies, and outputs.
4. Follow the workflow and dataflow diagrams before reading source files or tests.

<!-- N4-CRATE-VISUAL-GUIDE:END -->
