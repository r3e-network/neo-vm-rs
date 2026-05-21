# NeoVM 3.9 Conformance Fixtures

`neo_vm_3_9_conformance.json` stores pure VM execution vectors generated from
NuGet `Neo.VM` `3.9.0`.

The version chain is:

- `neo-node` `v3.9.2`
- `Neo` `3.9.1`
- `Neo.VM` `3.9.0`

Regenerate the fixture after changing the target NeoVM package version:

```powershell
dotnet run --project tools/NeoVm39ConformanceGenerator -- tests/fixtures/neo_vm_3_9_conformance.json
cargo test --test neo_vm_3_9_conformance --locked
```

The vectors intentionally avoid interop syscalls. They cover deterministic VM
semantics that can be executed directly by both `Neo.VM.ExecutionEngine` and
the Rust interpreter.
