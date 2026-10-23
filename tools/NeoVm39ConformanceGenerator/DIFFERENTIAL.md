# Live C# / Rust randomized differential execution

This is a bounded, deterministic property test against the real NuGet
Neo.VM reference engine, not a second handwritten interpreter or a static
fixture. It tests the property that both engines return the same terminal
state and full result stack for each generated script. See the parent
repository's doc.md §8 for the execution-profile boundary.

## Reference version

The oracle targets the Neo N3 3.10 line:
`neo-node v3.10.1` → `Neo 3.10.1` → `Neo.VM 3.10.1` (NuGet, net10.0). This
replaces the earlier 3.9.0 pin. Versions were verified against NuGet
(`Neo.VM 3.10.1`, `Neo 3.10.1` both published) and the neo-node Git tag
`v3.10.1`. `Neo.VM 3.10.2` is not published on NuGet (404). The project is
targeted at `net10.0` because Neo.VM 3.10.1 only ships a net10.0 asset.

## Run from the neo-n4 repository root

Requires .NET 10 SDK, Rust, and restored dependencies:

```sh
dotnet build external/neo-vm-rs/tools/NeoVm39ConformanceGenerator -p:NuGetAudit=false
cargo test --manifest-path external/neo-vm-rs/Cargo.toml --locked --test random_add_differential -- --ignored
cargo test --manifest-path external/neo-vm-rs/Cargo.toml --locked --test random_ops_differential -- --ignored
```

Build immediately before testing: the test executes the Debug/net10.0 DLL
rather than rebuilding it. A missing DLL or unsuccessful oracle process fails
the test. Both tests are explicitly ignored by ordinary `cargo test` because
they require the external .NET toolchain; a green ordinary Cargo run does not
certify these gates. No parent CI wiring is added by this change.

## Generator modes and evidence boundaries

Both generators emit JSON to stdout and accept `--random-ops|--random-add
<nonzero uint seed> <count>`, with count bounded to 1..4096. Existing
no-argument fixed-vector output and positional-file export remain unchanged.
Randomness uses xorshift32 with a fixed seed, so scripts are reproducible.

- `--random-add`: straight-line sums of 2..32 signed-byte operands
  (`PUSHINT8 ADD RET`); max 96 bytes, operand-stack depth two; always Halt.
- `--random-ops`: 12 families across integer and byte-splice semantics:
  `negate`, `abs`, `sub`, `div`, `mod`, `cat`, `left`, `right`, `substr`, and
  three forced-FAULT paths (`fault_empty_add`, `fault_empty_abs`,
  `fault_underflow_sub`). `div`/`mod`/`left`/`right`/`substr` reach Halt or
  Fault depending on generated operands (e.g. divide-by-zero, out-of-range
  byte-slice span).

Each Rust test runs 1024 cases per fixed seed (1, 42, 20260918, 4294967295),
compares terminal state for every case and the full result stack for every
Halt case, and additionally checks corpus size, name/family structure,
opcode-grammar shape, and per-family coverage. On a Fault case the state is
compared; the fault-path result stack is not, matching the official Neo.VM
VMUT harness convention. No loops, syscalls, network, storage or credentials
are involved.

## Findings this harness surfaced

Running the oracle at 3.10.1 with `--random-ops` initially failed because the
`cat` family emitted a `PUSHDATA1` length prefix drawn from a different
xorshift32 draw than the data bytes, producing a malformed script (stray
`0x3b` TRY bytes) that the two engines terminated differently. This was a
generator bug, not a VM divergence; the fix makes each byte-string use one
draw so the length prefix and data always agree. The mismatch is evidence the
differential gate detects invalid/ambiguous scripts rather than silently
accepting them.

## Scope limits

This covers normal integer arithmetic (including negative/zero operands),
integer division/modulo semantics, byte-string splice (`cat`, `left`,
`right`, `substr`), and fault states (divide-by-zero, out-of-range slice,
empty/underflowed operand stack). It does not implement shrinking or
coverage-guided fuzzing and does not cover collection aliasing, control flow,
syscalls, gas, arbitrary-width integers, maps/structs, or protocol
completeness. Both executors run natively on the host: this is not RISC-V
target execution, SP1 proving, or a machine-checked formal equivalence proof.
