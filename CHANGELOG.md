# Changelog

All notable changes to neo-vm-rs are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-07-02

First tagged release of neo-vm-rs — a single, canonical Rust reimplementation of
Neo N3's NeoVM, shared across the Neo N4 RISC-V stack so that opcode logic is never
re-implemented per project. It is `no_std + alloc` compatible, built on Rust 2024
edition, and organized as one shared `semantics` truth layer feeding two independent
execution engines (`interpreter` and `runtime`).

### Added

- **Multi-consumer cargo-feature split.** `default = ["std", "interpreter", "runtime"]`,
  with `interpreter` and `runtime` as independent, both-default-on features that each
  gate one engine and its re-exports. Consumers can now depend on only their slice —
  neo-vm on `interpreter`, neo-zkvm on `runtime`, riscvm on both — while sharing one
  VM. Backward-compatible: the default still pulls the full VM (`99a5ffa`, `c7e9035`).
- **Engine #1 — `interpreter`:** the consensus `match opcode` dispatch loop reached via
  `interpret()` and its `interpret_with_stack_and_syscalls[...]` variants; the path
  compiled into riscvm's frozen PolkaVM guest blob and used by the standalone neo-vm.
  Host functions plug in via the `SyscallProvider` trait (`f61f764`, `99a5ffa`).
- **Engine #2 — `runtime`:** the `VmContext` / `RuntimeStack` ABI plus an `ops`
  submodule, targeted by the devpack's NeoVM-to-Rust-to-RISC-V translator and by
  neo-zkvm's proving/trace backend (`c22a1a6`, `d41c383`).
- **Shared `semantics` layer** (`arithmetic`, `collections`, `comparison`, `conversion`,
  `splice`, plus internal numeric/stack helpers) — the always-compiled, pure opcode-rule
  layer that both engines route through, so interpreting and compiling a script can never
  diverge (`05fa120`, `e2fb5f7`).
- **Crate-root `PendingException` carrier** (`src/pending_exception.rs`), an always-on
  module that removes the sole coupling between the two engines so the features can be
  selected independently (`d202518`, `99a5ffa`).
- **`no_std + alloc`, Rust 2024 edition**, with lean, default-features-off dependencies
  (num-bigint, num-traits, serde, sha2) for the RISC-V guest environment (`05fa120`,
  `1770d15`).
- **Conformance harness:** official neo-project/neo-vm v3.9.0 VMUT fixtures (161 files /
  723 cases) replayed through the shared interpreter (`acbf528`); 20 pure-VM 3.9
  execution vectors with embedded version-chain provenance (`3c94beb`); a byte-for-byte
  opcode snapshot guard against canonical `OpCode.cs` (196 opcodes) (`3317b9a`,
  `b56db3e`); and 36 `source_layout` structural tests enforcing reviewability and the
  semantics/runtime module boundary (`2559b67`, `99a5ffa`).
- **cargo-fuzz targets** `interpret` and `interpret_with_stack`, sharing a
  consensus-invariant oracle and a `STEP_LIMIT`-bounded step hook for the gas-less
  interpreter (`6d5dcdf`, `5345326`, `c412440`), plus a Criterion `semantics` benchmark
  suite (`3ae72d3`).

### Fixed

Closed every known behavioral divergence from the canonical C# NeoVM (verified against
neo-project/neo-vm v3.9.0, the active pre-Gorgon mainnet behavior). Each item below was a
potential consensus-fork vector — a script that HALTs on one implementation and FAULTs on
the other.

- **Execution limits aligned to C# `ExecutionEngineLimits`:** `MaxItemSize` = 131070,
  `MaxComparableSize` = 65536 (enforced in EQUAL/NOTEQUAL/JMPEQ/JMPNE), native call depth
  raised to `MaxInvocationStackSize` = 1024 (64-frame bound kept only on the riscv32/
  PolkaVM heap-less profile), and the `MaxStackSize` (2048) check rewritten to mirror C#
  `ReferenceCounter.Count` (`b0913eb`, `ae819f4`, `9ae4416`, `edf4f4c`).
- **Strict operand-type and numeric coercion:** strict `GetBoolean`/`GetInteger` with
  uncatchable faults on >32-byte operands for BOOLAND/BOOLOR, JMPIF*/ASSERT, JMPEQ/JMPNE
  and CONVERT(Boolean); Buffer no longer accepted as a little-endian integer (D-1);
  PUSHDATA4 bounded to `MaxItemSize` (D-2); CLEARITEMS/POPITEM/POW/NEWARRAY_T operand
  checks (`ad928c8`, `4ef255e`, `02da7ff`, `10b4c54`, `26c3738`, `0fa5fdb`, `190522c`).
- **TRY/CATCH/FINALLY exception model:** faults on ENDTRY without an active try frame
  (D-3), JMP-family target `== Script.Length` (D-4), a second ENDTRY inside FINALLY
  (D-7), lazy end-target bounds check at IP assignment (D-9), per-context
  `MaxTryNestingDepth` = 16 (D-6), catch/finally bounds deferred to the IP setter (#2),
  and honoring a handler block whose target is IP 0 (`7c84e18`, `fcebc0f`, `c1f269d`,
  `7beb925`, `2ce4eaa`, `f1ac9e0`).
- **F5 — run finally bodies during exception unwinding:** an executor-local single-cell
  `unwind_exception` carries the exception while the finally runs (`pending_error`
  cleared) and re-propagates at ENDFINALLY, matching canonical `ExecuteThrow`; a new
  throw inside the finally supersedes it (`a4261bf`).
- **Collection index and store rules:** unified 32-byte strict index decoding across
  PICKITEM/SETITEM/REMOVE/HASKEY, strict index coercion (D8/D9/D10), Buffer SETITEM
  `[-128,255]` range check, ROLL `n==0` no-op, catchable out-of-range SETITEM, REMOVE
  absent-key no-op (D18), and BigInteger map keys matched numerically (D15) (`73f4d3c`,
  `0ee1335`, `0ecde99`, `d167c8e`, `96c3e35`, `abb43c0`, `14adeab`).
- **Compound-id disjointness keystone:** the global-atomic id band is now tagged with the
  high bit (`1<<63`) so it is provably disjoint from per-execution ids, eliminating a
  type-confusion/under-count collision in alias tracking and reference counting. This
  unblocked canonical `struct_clone` (D-5) and PACK/PACKSTRUCT store-by-reference
  (`e0c43e8`, `f3d2f40`, `242b688`).
- **Map-key validation and CONVERT rules:** Null map keys rejected across
  HASKEY/PICKITEM/SETITEM/REMOVE and PACKMAP; CONVERT(Iterator)→InteropInterface, CONVERT
  to Pointer rejected, ISTYPE(InteropInterface) true for iterators (D13) (`edfa83f`,
  `11fd816`).
- **VmContext (compiler runtime) exception parity:** the same single-cell
  `unwind_exception` design and corrected catchability (THROW catchable and routed to its
  own finally; ABORT/ABORTMSG/ASSERT/ASSERTMSG uncatchable) applied to the compiled path
  (`c22a1a6`, `d41c383`).

### Changed

- **Deduplicated opcode logic** by hoisting all behavior into the shared `src/semantics/`
  layer, removing hundreds of lines of divergent parallel copies while keeping observable
  semantics identical (`e2fb5f7`, `6098f43`, `0212b1c`, `7e47299`, `981899b`, `4107e95`).
- **Deduplicated codecs, ABI cursor, and host-retention logic** into shared helpers and a
  single `abi/cursor.rs`, preserving wire-format tag layout (`6db9b05`, `a1e2ef7`,
  `7a74856`, `da18d60`, `dcf1eee`).
- **Split oversized files** into focused directory modules with no behavior change: the
  interpreter state (call_frame/call_stack/try_frame/try_stack), the executor
  (`executor/mod.rs`), and the ~1279-line `abi/stack_value.rs` (tracked as a pure rename,
  all names re-exported) (`58fb4ff`, `2559b67`, `e7d82bf`).
- **Migrated to Rust 2024 edition** with edition-conforming import ordering (`377b408`,
  `1770d15`), and hardened `no_std` imports to pull from core/alloc (`d75694a`).
- **Documented SAFETY invariants** on the riscv32 unsafe paths; converted six
  `unreachable!()` dispatch arms to `Err()` returns; gated the per-execution
  `mem::forget` of result vectors to riscv32 only so host builds drop normally
  (`74ba7b5`, `837a8e9`).
- **Reached zero clippy warnings across every feature slice** and removed dead/duplicate
  helpers and compatibility aliases (`25570ca`, `c7e9035`, `1829a55`, `c203892`).

[0.2.0]: https://github.com/r3e-network/neo-vm-rs/releases/tag/v0.2.0
