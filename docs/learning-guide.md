# neo-vm-rs Source-Level Learning Guide

This guide is generated from the crate's actual `Cargo.toml`, Rust source files, public symbols, and test functions. It is meant to help a reader understand what this crate owns before reading implementation details.

## What This Crate Is

| Topic | Detail |
| --- | --- |
| Layer | Shared VM core |
| Purpose | Canonical Rust implementation of NeoVM 3.9.x semantics shared by RISC-V and zkVM paths. |
| Inputs | NeoVM bytecode, initial stack, syscall host callbacks |
| Responsibilities | Decode canonical opcodes, Execute stack and state semantics, Expose reusable runtime APIs |
| Outputs | halt/fault result, final stack, gas/accounting evidence |
| Consumers | neo-riscv-vm, neo-zkvm, Neo N4 execution core |

## Visual Reading Order

| Step | Diagram | Use it to learn |
| ---: | --- | --- |
| 1 | [Position](figures/position.svg) | Why this crate exists and where it sits in Neo N4. |
| 2 | [Principles](figures/principles.svg) | The invariants and boundaries this crate must protect. |
| 3 | [Module map](figures/module-map.svg) | Which files are the best entry points. |
| 4 | [Public API surface](figures/api-surface.svg) | Which exported symbols form the crate contract. |
| 5 | [Architecture](figures/architecture.svg) | How inputs, internal components, dependencies, and outputs connect. |
| 6 | [Workflow](figures/workflow.svg) | The normal execution path. |
| 7 | [Dataflow](figures/dataflow.svg) | How data is transformed across the crate boundary. |
| 8 | [Test evidence](figures/test-map.svg) | Which tests protect the behavior. |
| 9 | [Dependency map](figures/dependency-map.svg) | Which dependencies are runtime, test, or build-only. |

## Source File Map

| File | Role | Public symbols | Tests |
| --- | --- | ---: | ---: |
| `src/lib.rs` | crate root, public exports, and top-level documentation | 0 | 0 |
| `src/abi/stack_value.rs` | wire format, stack value, or host/guest boundary type | 59 | 13 |
| `src/interpreter/state.rs` | VM interpreter and opcode semantics | 37 | 0 |
| `src/interpreter/helpers/values.rs` | VM interpreter and opcode semantics | 36 | 0 |
| `src/runtime/mod.rs` | execution runtime, state transition, or gas behavior | 33 | 2 |
| `src/semantics/arithmetic.rs` | implementation detail or helper module | 23 | 0 |
| `src/semantics/runtime/arithmetic.rs` | execution runtime, state transition, or gas behavior | 23 | 0 |
| `src/semantics/runtime/collections.rs` | execution runtime, state transition, or gas behavior | 22 | 0 |
| `src/semantics/runtime/comparison.rs` | execution runtime, state transition, or gas behavior | 20 | 0 |
| `src/semantics/collections.rs` | implementation detail or helper module | 19 | 0 |
| `src/interpreter/runtime_types.rs` | execution runtime, state transition, or gas behavior | 16 | 0 |
| `src/semantics/runtime/stack.rs` | execution runtime, state transition, or gas behavior | 16 | 0 |
| `src/semantics/comparison.rs` | implementation detail or helper module | 13 | 0 |
| `src/interpreter/api.rs` | VM interpreter and opcode semantics | 11 | 1 |
| `src/vm/opcode.rs` | opcode metadata, pricing, or canonical decode rules | 7 | 0 |
| `src/abi/fast_codec.rs` | wire format, stack value, or host/guest boundary type | 3 | 8 |
| `src/interpreter/helpers/retained.rs` | VM interpreter and opcode semantics | 4 | 4 |
| `src/interpreter/helpers/bridge.rs` | VM interpreter and opcode semantics | 5 | 0 |
| `src/interpreter/helpers/mod.rs` | VM interpreter and opcode semantics | 5 | 0 |
| `src/semantics/runtime/byte_ops.rs` | execution runtime, state transition, or gas behavior | 5 | 0 |
| `src/semantics/runtime/conversion.rs` | execution runtime, state transition, or gas behavior | 4 | 0 |
| `src/vm/limits.rs` | implementation detail or helper module | 4 | 0 |
| `src/abi/callback_codec.rs` | wire format, stack value, or host/guest boundary type | 3 | 0 |
| `src/abi/execution.rs` | wire format, stack value, or host/guest boundary type | 3 | 0 |
| `src/host/syscall.rs` | host syscall contract and dispatch boundary | 2 | 3 |
| `src/semantics/runtime/mod.rs` | execution runtime, state transition, or gas behavior | 3 | 0 |
| `src/abi/result_codec.rs` | wire format, stack value, or host/guest boundary type | 2 | 0 |
| `src/semantics/conversion.rs` | implementation detail or helper module | 2 | 0 |
| `tests/runtime_opcode_ops.rs` | external behavior or integration test | 0 | 6 |
| `tests/abi_opcode_semantics.rs` | external behavior or integration test | 0 | 5 |
| `tests/shared_semantics.rs` | external behavior or integration test | 0 | 5 |
| `tests/source_layout.rs` | external behavior or integration test | 0 | 5 |
| `tests/interpreter_smoke.rs` | external behavior or integration test | 0 | 4 |
| `src/interpreter/executor.rs` | VM interpreter and opcode semantics | 1 | 0 |
| `src/interpreter/executor/byte_ops.rs` | VM interpreter and opcode semantics | 1 | 0 |
| `src/interpreter/executor/compound_ops.rs` | VM interpreter and opcode semantics | 1 | 0 |
| `src/interpreter/executor/control.rs` | VM interpreter and opcode semantics | 1 | 0 |
| `src/interpreter/executor/numeric_ops.rs` | VM interpreter and opcode semantics | 1 | 0 |
| `src/interpreter/executor/push_ops.rs` | VM interpreter and opcode semantics | 1 | 0 |
| `src/interpreter/executor/result_ops.rs` | VM interpreter and opcode semantics | 1 | 0 |
| `src/interpreter/executor/slot_ops.rs` | VM interpreter and opcode semantics | 1 | 0 |
| `src/interpreter/executor/stack_ops.rs` | VM interpreter and opcode semantics | 1 | 0 |
| `tests/boundary_codecs.rs` | external behavior or integration test | 0 | 2 |
| `tests/canonical_opcode_vectors.rs` | external behavior or integration test | 0 | 2 |
| `tests/syscall_vectors.rs` | external behavior or integration test | 0 | 2 |
| `tests/wire_semantics.rs` | external behavior or integration test | 0 | 2 |
| `tests/neo_vm_3_9_conformance.rs` | external behavior or integration test | 0 | 1 |
| `tests/official_neo_vm_3_9_vmut.rs` | external behavior or integration test | 0 | 1 |
| `benches/semantics.rs` | implementation detail or helper module | 0 | 0 |
| `src/abi/mod.rs` | wire format, stack value, or host/guest boundary type | 0 | 0 |
| `src/host/mod.rs` | host-side orchestration and native integration | 0 | 0 |
| `src/interpreter/mod.rs` | VM interpreter and opcode semantics | 0 | 0 |
| `src/interpreter/opcodes.rs` | VM interpreter and opcode semantics | 0 | 0 |
| `src/semantics/mod.rs` | implementation detail or helper module | 0 | 0 |
| `src/vm/mod.rs` | implementation detail or helper module | 0 | 0 |

## Public API Surface

| Symbol | File |
| --- | --- |
| `fn encode_stack_result` | `src/abi/callback_codec.rs` |
| `fn decode_stack_result_into` | `src/abi/callback_codec.rs` |
| `fn decode_stack_result` | `src/abi/callback_codec.rs` |
| `enum VmState` | `src/abi/execution.rs` |
| `enum BackendKind` | `src/abi/execution.rs` |
| `struct ExecutionResult` | `src/abi/execution.rs` |
| `fn encode_stack` | `src/abi/fast_codec.rs` |
| `fn encode_stack_to_slice` | `src/abi/fast_codec.rs` |
| `fn decode_stack` | `src/abi/fast_codec.rs` |
| `fn encode_execution_result` | `src/abi/result_codec.rs` |
| `fn decode_execution_result` | `src/abi/result_codec.rs` |
| `const COMPACT_TAG_INTEGER` | `src/abi/stack_value.rs` |
| `const COMPACT_TAG_BOOLEAN` | `src/abi/stack_value.rs` |
| `const COMPACT_TAG_BYTESTRING` | `src/abi/stack_value.rs` |
| `const COMPACT_TAG_BIG_INTEGER` | `src/abi/stack_value.rs` |
| `const COMPACT_TAG_ARRAY` | `src/abi/stack_value.rs` |
| `const COMPACT_TAG_STRUCT` | `src/abi/stack_value.rs` |
| `const COMPACT_TAG_MAP` | `src/abi/stack_value.rs` |
| `const COMPACT_TAG_NULL` | `src/abi/stack_value.rs` |
| `const COMPACT_TAG_INTEROP` | `src/abi/stack_value.rs` |
| `const COMPACT_TAG_ITERATOR` | `src/abi/stack_value.rs` |
| `const COMPACT_TAG_BUFFER` | `src/abi/stack_value.rs` |
| `const COMPACT_TAG_POINTER` | `src/abi/stack_value.rs` |
| `const STACK_VALUE_CODEC_TAG_INTEGER` | `src/abi/stack_value.rs` |
| `const STACK_VALUE_CODEC_TAG_BIG_INTEGER` | `src/abi/stack_value.rs` |
| `const STACK_VALUE_CODEC_TAG_BYTESTRING` | `src/abi/stack_value.rs` |
| `const STACK_VALUE_CODEC_TAG_BOOLEAN` | `src/abi/stack_value.rs` |
| `const STACK_VALUE_CODEC_TAG_ARRAY` | `src/abi/stack_value.rs` |
| `const STACK_VALUE_CODEC_TAG_STRUCT` | `src/abi/stack_value.rs` |
| `const STACK_VALUE_CODEC_TAG_MAP` | `src/abi/stack_value.rs` |
| `const STACK_VALUE_CODEC_TAG_INTEROP` | `src/abi/stack_value.rs` |
| `const STACK_VALUE_CODEC_TAG_ITERATOR` | `src/abi/stack_value.rs` |
| `const STACK_VALUE_CODEC_TAG_NULL` | `src/abi/stack_value.rs` |
| `const STACK_VALUE_CODEC_TAG_POINTER` | `src/abi/stack_value.rs` |
| `const STACK_VALUE_CODEC_TAG_BUFFER` | `src/abi/stack_value.rs` |
| `const NEOVM_STACK_ITEM_TYPE_ANY` | `src/abi/stack_value.rs` |
| `const NEOVM_STACK_ITEM_TYPE_POINTER` | `src/abi/stack_value.rs` |
| `const NEOVM_STACK_ITEM_TYPE_BOOLEAN` | `src/abi/stack_value.rs` |
| `const NEOVM_STACK_ITEM_TYPE_INTEGER` | `src/abi/stack_value.rs` |
| `const NEOVM_STACK_ITEM_TYPE_BYTESTRING` | `src/abi/stack_value.rs` |
| `const NEOVM_STACK_ITEM_TYPE_BUFFER` | `src/abi/stack_value.rs` |
| `const NEOVM_STACK_ITEM_TYPE_ARRAY` | `src/abi/stack_value.rs` |
| `const NEOVM_STACK_ITEM_TYPE_STRUCT` | `src/abi/stack_value.rs` |
| `const NEOVM_STACK_ITEM_TYPE_MAP` | `src/abi/stack_value.rs` |
| `const NEOVM_STACK_ITEM_TYPE_INTEROP_INTERFACE` | `src/abi/stack_value.rs` |
| `fn normalize_stack_item_type_tag` | `src/abi/stack_value.rs` |
| `fn default_value_for_type_tag` | `src/abi/stack_value.rs` |
| `fn new_array_default_value_for_type_tag` | `src/abi/stack_value.rs` |
| `fn pop_byte_arg` | `src/abi/stack_value.rs` |
| `fn byte_sequence_bytes` | `src/abi/stack_value.rs` |
| `fn byte_sequence_len` | `src/abi/stack_value.rs` |
| `fn stack_value_as_bool` | `src/abi/stack_value.rs` |
| `fn stack_value_as_i64` | `src/abi/stack_value.rs` |
| `fn stack_value_as_u32` | `src/abi/stack_value.rs` |
| `fn stack_value_as_u8` | `src/abi/stack_value.rs` |
| `fn stack_value_as_bytes` | `src/abi/stack_value.rs` |
| `fn stack_value_as_fixed_bytes` | `src/abi/stack_value.rs` |
| `fn stack_value_as_string` | `src/abi/stack_value.rs` |
| `fn stack_value_into_items` | `src/abi/stack_value.rs` |
| `fn concat_byte_sequences` | `src/abi/stack_value.rs` |
| `fn slice_byte_sequence` | `src/abi/stack_value.rs` |
| `fn encode_integer` | `src/abi/stack_value.rs` |
| `enum StackValue` | `src/abi/stack_value.rs` |
| `fn compact_type_tag` | `src/abi/stack_value.rs` |
| `fn to_bool` | `src/abi/stack_value.rs` |
| `fn to_i128` | `src/abi/stack_value.rs` |
| `fn as_bytes` | `src/abi/stack_value.rs` |
| `fn to_byte_string_bytes` | `src/abi/stack_value.rs` |
| `fn convert_to_byte_string_value` | `src/abi/stack_value.rs` |
| `fn convert_to_buffer_value` | `src/abi/stack_value.rs` |
| `fn interop_hash` | `src/host/syscall.rs` |
| `fn syscall_arg_count` | `src/host/syscall.rs` |
| `fn interpret` | `src/interpreter/api.rs` |
| `trait SyscallProvider` | `src/interpreter/api.rs` |
| `const CALLT_MARKER` | `src/interpreter/api.rs` |
| `const CALLT_MARKER_HI` | `src/interpreter/api.rs` |
| `const INITIALIZER_COMPLETE_MARKER` | `src/interpreter/api.rs` |
| `fn interpret_with_syscalls` | `src/interpreter/api.rs` |
| `fn interpret_with_stack_and_syscalls` | `src/interpreter/api.rs` |
| `fn interpret_with_stack_and_syscalls_at` | `src/interpreter/api.rs` |
| `fn interpret_with_stack_and_syscalls_at_with_result_limit` | `src/interpreter/api.rs` |
| `fn interpret_with_stack_and_syscalls_at_with_initializer` | `src/interpreter/api.rs` |
| `fn interpret_with_stack_and_syscalls_at_with_initializer_and_result_limit` | `src/interpreter/api.rs` |
| `fn execute` | `src/interpreter/executor/byte_ops.rs` |
| `fn execute` | `src/interpreter/executor/compound_ops.rs` |
| `enum Dispatch` | `src/interpreter/executor/control.rs` |
| `fn execute` | `src/interpreter/executor/numeric_ops.rs` |
| `fn execute` | `src/interpreter/executor/push_ops.rs` |
| `fn trim_halt_stack_for_result_limit` | `src/interpreter/executor/result_ops.rs` |
| `fn execute` | `src/interpreter/executor/slot_ops.rs` |
| `fn execute` | `src/interpreter/executor/stack_ops.rs` |
| `fn interpret_with_stack_and_syscalls_at_internal` | `src/interpreter/executor.rs` |
| `fn invoke_syscall` | `src/interpreter/helpers/bridge.rs` |
| `fn complete_initializer_retaining_state` | `src/interpreter/helpers/bridge.rs` |
| `fn retain_initializer_method_stack` | `src/interpreter/helpers/bridge.rs` |
| `fn restore_initializer_method_stack` | `src/interpreter/helpers/bridge.rs` |
| `fn invoke_callt` | `src/interpreter/helpers/bridge.rs` |
| `struct RetainedPrefixBuffer` | `src/interpreter/helpers/mod.rs` |
| `fn as_mut_slice` | `src/interpreter/helpers/mod.rs` |
| `fn as_slice` | `src/interpreter/helpers/mod.rs` |
| `static RETAINED_ARGS_BUF` | `src/interpreter/helpers/mod.rs` |
| `static RETAINED_CALL_STACK_BUF` | `src/interpreter/helpers/mod.rs` |
| `fn encode_retained_prefix_to_slice` | `src/interpreter/helpers/retained.rs` |
| `fn encode_retained_value_to_slice` | `src/interpreter/helpers/retained.rs` |
| `fn ensure_retained_capacity` | `src/interpreter/helpers/retained.rs` |
| `fn decode_retained_prefix_into` | `src/interpreter/helpers/retained.rs` |
| `fn peek_item` | `src/interpreter/helpers/values.rs` |
| `fn pop_item` | `src/interpreter/helpers/values.rs` |
| `fn pop_integer` | `src/interpreter/helpers/values.rs` |
| `fn pop_bigint_pair_allowing_null_false` | `src/interpreter/helpers/values.rs` |
| `fn pop_shift_count` | `src/interpreter/helpers/values.rs` |
| `fn pop_numeric_bigint` | `src/interpreter/helpers/values.rs` |
| `fn shift_value_from_item` | `src/interpreter/helpers/values.rs` |
| `fn num_equal` | `src/interpreter/helpers/values.rs` |
| `fn integer_value_for_collection_index` | `src/interpreter/helpers/values.rs` |
| `fn validate_map_key` | `src/interpreter/helpers/values.rs` |
| `fn primitive_key_equals` | `src/interpreter/helpers/values.rs` |
| `fn vm_equal` | `src/interpreter/helpers/values.rs` |
| `fn convert_value` | `src/interpreter/helpers/values.rs` |
| `fn boolean_value` | `src/interpreter/helpers/values.rs` |
| `fn decode_signed_le_bytes` | `src/interpreter/helpers/values.rs` |
| `fn decode_signed_le_bytes_bigint` | `src/interpreter/helpers/values.rs` |
| `struct ShiftValue` | `src/interpreter/helpers/values.rs` |
| `fn shift_left` | `src/interpreter/helpers/values.rs` |
| `fn shift_right` | `src/interpreter/helpers/values.rs` |
| `fn pop_boolean` | `src/interpreter/helpers/values.rs` |
| `fn item_to_boolean_strict` | `src/interpreter/helpers/values.rs` |
| `fn mod_pow_bigint` | `src/interpreter/helpers/values.rs` |
| `fn pop_bytes` | `src/interpreter/helpers/values.rs` |
| `fn stack_item_to_bytes` | `src/interpreter/helpers/values.rs` |
| `fn encode_integer` | `src/interpreter/helpers/values.rs` |
| `fn numeric_result_bigint` | `src/interpreter/helpers/values.rs` |
| `fn bigint_sign` | `src/interpreter/helpers/values.rs` |
| `fn bigint_abs` | `src/interpreter/helpers/values.rs` |
| `enum Offset` | `src/interpreter/helpers/values.rs` |
| `fn read_offset` | `src/interpreter/helpers/values.rs` |
| `fn compute_jump_target_offset` | `src/interpreter/helpers/values.rs` |
| `fn trim_le_bytes` | `src/interpreter/helpers/values.rs` |
| `fn bytes_to_integer` | `src/interpreter/helpers/values.rs` |
| `fn bitwise_result` | `src/interpreter/helpers/values.rs` |
| `fn bigint_or_integer` | `src/interpreter/helpers/values.rs` |
| `fn trim_le_bytes_slice` | `src/interpreter/helpers/values.rs` |
| `enum StackValue` | `src/interpreter/runtime_types.rs` |
| `struct CompoundIds` | `src/interpreter/runtime_types.rs` |
| `fn array` | `src/interpreter/runtime_types.rs` |
| `fn r` | `src/interpreter/runtime_types.rs` |
| `fn map` | `src/interpreter/runtime_types.rs` |
| `fn buffer` | `src/interpreter/runtime_types.rs` |
| `fn clone_struct_for_storage` | `src/interpreter/runtime_types.rs` |
| `fn deep_clone` | `src/interpreter/runtime_types.rs` |
| `fn import_abi` | `src/interpreter/runtime_types.rs` |
| `fn to_abi_stack` | `src/interpreter/runtime_types.rs` |
| `fn to_abi_value` | `src/interpreter/runtime_types.rs` |
| `fn structurally_equal` | `src/interpreter/runtime_types.rs` |
| `fn compound_id` | `src/interpreter/runtime_types.rs` |
| `fn find_affected_indices` | `src/interpreter/runtime_types.rs` |
| `fn propagate_update` | `src/interpreter/runtime_types.rs` |
| `fn propagate_aliases_from_sources` | `src/interpreter/runtime_types.rs` |
| `static LAST_INTERPRETER_IP` | `src/interpreter/state.rs` |
| `static LAST_RESULT_STAGE` | `src/interpreter/state.rs` |
| `static LAST_RESULT_STACK_LEN` | `src/interpreter/state.rs` |
| `static LAST_RESULT_LIMIT` | `src/interpreter/state.rs` |
| `fn record_interpreter_ip` | `src/interpreter/state.rs` |
| `fn last_interpreter_ip` | `src/interpreter/state.rs` |
| `fn last_result_stage` | `src/interpreter/state.rs` |
| `fn last_result_stack_len` | `src/interpreter/state.rs` |
| `fn last_result_limit` | `src/interpreter/state.rs` |
| `fn propagate_active_aliases_into_saved_frame` | `src/interpreter/state.rs` |
| `fn remember_consumed_mutation` | `src/interpreter/state.rs` |
| `fn reset_consumed_mutations` | `src/interpreter/state.rs` |
| `struct TryFrame` | `src/interpreter/state.rs` |
| `enum PendingException` | `src/interpreter/state.rs` |
| `fn message` | `src/interpreter/state.rs` |
| `fn thrown_value` | `src/interpreter/state.rs` |
| `fn into_catch_item` | `src/interpreter/state.rs` |
| `fn into_fault_message` | `src/interpreter/state.rs` |
| `const MAX_STACK_SIZE` | `src/interpreter/state.rs` |
| `const MAX_TRY_NESTING` | `src/interpreter/state.rs` |
| `const MAX_CALL_DEPTH` | `src/interpreter/state.rs` |
| `struct TryStack` | `src/interpreter/state.rs` |
| `struct CallFrame` | `src/interpreter/state.rs` |
| `type RestoredCallFrame` | `src/interpreter/state.rs` |
| `struct CallStack` | `src/interpreter/state.rs` |
| `fn new` | `src/interpreter/state.rs` |
| `fn len` | `src/interpreter/state.rs` |
| `fn push_frame_refs` | `src/interpreter/state.rs` |
| `fn push_frame` | `src/interpreter/state.rs` |
| `fn pop_and_restore` | `src/interpreter/state.rs` |
| `fn new` | `src/interpreter/state.rs` |
| `fn is_empty` | `src/interpreter/state.rs` |
| `fn push` | `src/interpreter/state.rs` |
| `fn pop` | `src/interpreter/state.rs` |
| `fn last_mut` | `src/interpreter/state.rs` |
| `fn find_uncaught_index` | `src/interpreter/state.rs` |
| `fn get_mut` | `src/interpreter/state.rs` |
| `struct VmContext` | `src/runtime/mod.rs` |
| `fn from_stack` | `src/runtime/mod.rs` |
| `fn from_abi_stack` | `src/runtime/mod.rs` |
| `fn init_slot` | `src/runtime/mod.rs` |
| `fn init_sslot` | `src/runtime/mod.rs` |
| `fn fault` | `src/runtime/mod.rs` |
| `fn is_faulted` | `src/runtime/mod.rs` |
| `fn into_execution_result` | `src/runtime/mod.rs` |
| `fn to_execution_result` | `src/runtime/mod.rs` |
| `fn push` | `src/runtime/mod.rs` |
| `fn push_int` | `src/runtime/mod.rs` |
| `fn push_bool` | `src/runtime/mod.rs` |
| `fn push_bytes` | `src/runtime/mod.rs` |
| `fn push_null` | `src/runtime/mod.rs` |
| `fn pop` | `src/runtime/mod.rs` |
| `fn load_arg` | `src/runtime/mod.rs` |
| `fn store_arg` | `src/runtime/mod.rs` |
| `fn load_local` | `src/runtime/mod.rs` |
| `fn store_local` | `src/runtime/mod.rs` |
| `fn load_static` | `src/runtime/mod.rs` |
| `fn store_static` | `src/runtime/mod.rs` |
| `fn throw_ex` | `src/runtime/mod.rs` |
| `fn abort` | `src/runtime/mod.rs` |
| `fn abort_msg` | `src/runtime/mod.rs` |
| `fn assert_top` | `src/runtime/mod.rs` |
| `fn assert_msg` | `src/runtime/mod.rs` |
| `fn try_enter` | `src/runtime/mod.rs` |
| `fn end_try` | `src/runtime/mod.rs` |
| `fn end_finally` | `src/runtime/mod.rs` |
| `fn check_exception` | `src/runtime/mod.rs` |
| `fn call_push` | `src/runtime/mod.rs` |
| `fn call_pop` | `src/runtime/mod.rs` |
| `fn ret` | `src/runtime/mod.rs` |
| `fn add_i64` | `src/semantics/arithmetic.rs` |
| `fn sub_i64` | `src/semantics/arithmetic.rs` |
| `fn mul_i64` | `src/semantics/arithmetic.rs` |
| `fn div_i64` | `src/semantics/arithmetic.rs` |
| `fn modulo_i64` | `src/semantics/arithmetic.rs` |
| `fn negate_i64` | `src/semantics/arithmetic.rs` |
| `fn abs_i64` | `src/semantics/arithmetic.rs` |
| `fn sign_i64` | `src/semantics/arithmetic.rs` |
| `fn max_i64` | `src/semantics/arithmetic.rs` |
| `fn min_i64` | `src/semantics/arithmetic.rs` |
| `fn pow_i64` | `src/semantics/arithmetic.rs` |
| `fn sqrt_i64` | `src/semantics/arithmetic.rs` |
| `fn modmul_i64` | `src/semantics/arithmetic.rs` |
| `fn modpow_i64` | `src/semantics/arithmetic.rs` |
| `fn shl_i64` | `src/semantics/arithmetic.rs` |
| `fn shr_i64` | `src/semantics/arithmetic.rs` |
| `fn bitwise_and_i64` | `src/semantics/arithmetic.rs` |
| `fn bitwise_or_i64` | `src/semantics/arithmetic.rs` |
| `fn bitwise_xor_i64` | `src/semantics/arithmetic.rs` |
| `fn bitwise_not_i64` | `src/semantics/arithmetic.rs` |
| `fn inc_i64` | `src/semantics/arithmetic.rs` |
| `fn dec_i64` | `src/semantics/arithmetic.rs` |
| `fn within_i64` | `src/semantics/arithmetic.rs` |
| `fn new_array` | `src/semantics/collections.rs` |
| `fn new_array_t` | `src/semantics/collections.rs` |
| `fn new_struct` | `src/semantics/collections.rs` |
| `fn new_buffer` | `src/semantics/collections.rs` |
| `fn append` | `src/semantics/collections.rs` |
| `fn set_item` | `src/semantics/collections.rs` |
| `fn pick_item` | `src/semantics/collections.rs` |
| `fn remove` | `src/semantics/collections.rs` |
| `fn size` | `src/semantics/collections.rs` |
| `fn has_key` | `src/semantics/collections.rs` |
| `fn keys` | `src/semantics/collections.rs` |
| `fn values` | `src/semantics/collections.rs` |
| `fn pack` | `src/semantics/collections.rs` |
| `fn unpack` | `src/semantics/collections.rs` |
| `fn reverse_items` | `src/semantics/collections.rs` |
| `fn clear_items` | `src/semantics/collections.rs` |
| `fn pop_item` | `src/semantics/collections.rs` |
| `fn pack_struct` | `src/semantics/collections.rs` |
| `fn pack_map` | `src/semantics/collections.rs` |
| `fn equal_values` | `src/semantics/comparison.rs` |
| `fn not_equal_values` | `src/semantics/comparison.rs` |
| `fn less_than_i64` | `src/semantics/comparison.rs` |
| `fn less_or_equal_i64` | `src/semantics/comparison.rs` |
| `fn greater_than_i64` | `src/semantics/comparison.rs` |
| `fn greater_or_equal_i64` | `src/semantics/comparison.rs` |
| `fn num_equal_i64` | `src/semantics/comparison.rs` |
| `fn num_not_equal_i64` | `src/semantics/comparison.rs` |
| `fn bool_and` | `src/semantics/comparison.rs` |
| `fn bool_or` | `src/semantics/comparison.rs` |
| `fn bool_not` | `src/semantics/comparison.rs` |
| `fn nz` | `src/semantics/comparison.rs` |
| `fn is_null` | `src/semantics/comparison.rs` |
| `fn is_type` | `src/semantics/conversion.rs` |
| `fn convert_value` | `src/semantics/conversion.rs` |
| `fn add` | `src/semantics/runtime/arithmetic.rs` |
| `fn sub` | `src/semantics/runtime/arithmetic.rs` |
| `fn mul` | `src/semantics/runtime/arithmetic.rs` |
| `fn div` | `src/semantics/runtime/arithmetic.rs` |
| `fn modulo` | `src/semantics/runtime/arithmetic.rs` |
| `fn negate` | `src/semantics/runtime/arithmetic.rs` |
| `fn abs` | `src/semantics/runtime/arithmetic.rs` |
| `fn sign` | `src/semantics/runtime/arithmetic.rs` |
| `fn max` | `src/semantics/runtime/arithmetic.rs` |
| `fn min` | `src/semantics/runtime/arithmetic.rs` |
| `fn pow` | `src/semantics/runtime/arithmetic.rs` |
| `fn sqrt` | `src/semantics/runtime/arithmetic.rs` |
| `fn modmul` | `src/semantics/runtime/arithmetic.rs` |
| `fn modpow` | `src/semantics/runtime/arithmetic.rs` |
| `fn shl` | `src/semantics/runtime/arithmetic.rs` |
| `fn shr` | `src/semantics/runtime/arithmetic.rs` |
| `fn bitwise_and` | `src/semantics/runtime/arithmetic.rs` |
| `fn bitwise_or` | `src/semantics/runtime/arithmetic.rs` |
| `fn bitwise_xor` | `src/semantics/runtime/arithmetic.rs` |
| `fn bitwise_not` | `src/semantics/runtime/arithmetic.rs` |
| `fn inc` | `src/semantics/runtime/arithmetic.rs` |
| `fn dec` | `src/semantics/runtime/arithmetic.rs` |
| `fn within` | `src/semantics/runtime/arithmetic.rs` |
| `fn cat` | `src/semantics/runtime/byte_ops.rs` |
| `fn substr` | `src/semantics/runtime/byte_ops.rs` |
| `fn left` | `src/semantics/runtime/byte_ops.rs` |
| `fn right` | `src/semantics/runtime/byte_ops.rs` |
| `fn memcpy` | `src/semantics/runtime/byte_ops.rs` |
| `fn new_array_0` | `src/semantics/runtime/collections.rs` |
| `fn new_array` | `src/semantics/runtime/collections.rs` |
| `fn new_array_t` | `src/semantics/runtime/collections.rs` |
| `fn new_struct_0` | `src/semantics/runtime/collections.rs` |
| `fn new_struct` | `src/semantics/runtime/collections.rs` |
| `fn new_map` | `src/semantics/runtime/collections.rs` |
| `fn new_buffer` | `src/semantics/runtime/collections.rs` |
| `fn append` | `src/semantics/runtime/collections.rs` |
| `fn set_item` | `src/semantics/runtime/collections.rs` |
| `fn pick_item` | `src/semantics/runtime/collections.rs` |
| `fn remove` | `src/semantics/runtime/collections.rs` |
| `fn size` | `src/semantics/runtime/collections.rs` |
| `fn has_key` | `src/semantics/runtime/collections.rs` |
| `fn keys` | `src/semantics/runtime/collections.rs` |
| `fn values` | `src/semantics/runtime/collections.rs` |
| `fn pack` | `src/semantics/runtime/collections.rs` |
| `fn unpack` | `src/semantics/runtime/collections.rs` |
| `fn reverse_items` | `src/semantics/runtime/collections.rs` |
| `fn clear_items` | `src/semantics/runtime/collections.rs` |
| `fn pop_item` | `src/semantics/runtime/collections.rs` |
| `fn pack_struct` | `src/semantics/runtime/collections.rs` |
| `fn pack_map` | `src/semantics/runtime/collections.rs` |
| `fn equal` | `src/semantics/runtime/comparison.rs` |
| `fn not_equal` | `src/semantics/runtime/comparison.rs` |
| `fn less_than` | `src/semantics/runtime/comparison.rs` |
| `fn less_or_equal` | `src/semantics/runtime/comparison.rs` |
| `fn greater_than` | `src/semantics/runtime/comparison.rs` |
| `fn greater_or_equal` | `src/semantics/runtime/comparison.rs` |
| `fn num_equal` | `src/semantics/runtime/comparison.rs` |
| `fn num_not_equal` | `src/semantics/runtime/comparison.rs` |
| `fn bool_and` | `src/semantics/runtime/comparison.rs` |
| `fn bool_or` | `src/semantics/runtime/comparison.rs` |
| `fn not` | `src/semantics/runtime/comparison.rs` |
| `fn nz` | `src/semantics/runtime/comparison.rs` |
| `fn is_null` | `src/semantics/runtime/comparison.rs` |
| `fn pop_bool` | `src/semantics/runtime/comparison.rs` |
| `fn pop_cmp_eq` | `src/semantics/runtime/comparison.rs` |
| `fn pop_cmp_ne` | `src/semantics/runtime/comparison.rs` |
| `fn pop_cmp_gt` | `src/semantics/runtime/comparison.rs` |
| `fn pop_cmp_ge` | `src/semantics/runtime/comparison.rs` |
| `fn pop_cmp_lt` | `src/semantics/runtime/comparison.rs` |
| `fn pop_cmp_le` | `src/semantics/runtime/comparison.rs` |
| `fn is_type` | `src/semantics/runtime/conversion.rs` |
| `fn convert_to` | `src/semantics/runtime/conversion.rs` |
| `fn push_bigint` | `src/semantics/runtime/conversion.rs` |
| `fn push_default` | `src/semantics/runtime/conversion.rs` |
| `trait RuntimeStack` | `src/semantics/runtime/mod.rs` |
| `fn push_i64_result` | `src/semantics/runtime/mod.rs` |
| `fn push_value_result` | `src/semantics/runtime/mod.rs` |
| `fn drop_top` | `src/semantics/runtime/stack.rs` |
| `fn dup` | `src/semantics/runtime/stack.rs` |
| `fn swap` | `src/semantics/runtime/stack.rs` |
| `fn nip` | `src/semantics/runtime/stack.rs` |
| `fn xdrop` | `src/semantics/runtime/stack.rs` |
| `fn over` | `src/semantics/runtime/stack.rs` |
| `fn pick` | `src/semantics/runtime/stack.rs` |
| `fn pick_n` | `src/semantics/runtime/stack.rs` |
| `fn tuck` | `src/semantics/runtime/stack.rs` |
| `fn rot` | `src/semantics/runtime/stack.rs` |
| `fn roll` | `src/semantics/runtime/stack.rs` |
| `fn reverse3` | `src/semantics/runtime/stack.rs` |
| `fn reverse4` | `src/semantics/runtime/stack.rs` |
| `fn reverse_n` | `src/semantics/runtime/stack.rs` |
| `fn depth` | `src/semantics/runtime/stack.rs` |
| `fn clear` | `src/semantics/runtime/stack.rs` |
| `const MAX_SCRIPT_SIZE` | `src/vm/limits.rs` |
| `const DEFAULT_MAX_STACK_DEPTH` | `src/vm/limits.rs` |
| `const DEFAULT_MAX_INVOCATION_DEPTH` | `src/vm/limits.rs` |
| `const MAX_ITEM_SIZE` | `src/vm/limits.rs` |
| `enum OpCode` | `src/vm/opcode.rs` |
| `const ALL` | `src/vm/opcode.rs` |
| `fn from_u8` | `src/vm/opcode.rs` |
| `const fn` | `src/vm/opcode.rs` |
| `const fn` | `src/vm/opcode.rs` |
| `const fn` | `src/vm/opcode.rs` |
| `const fn` | `src/vm/opcode.rs` |

## Module and Re-Export Signals

| Signal |
| --- |
| `src/abi/mod.rs: mod callback_codec` |
| `src/abi/mod.rs: mod execution` |
| `src/abi/mod.rs: mod fast_codec` |
| `src/abi/mod.rs: mod result_codec` |
| `src/abi/mod.rs: mod stack_value` |
| `src/abi/mod.rs: pub use execution::{BackendKind, ExecutionResult, VmState}` |
| `src/abi/mod.rs: pub use stack_value::{     byte_sequence_bytes, byte_sequence_len, concat_byte_sequences, default_value_for_type_tag,     encode_integer, new_array_default_value_for_type_tag, normalize_stack_item_type_tag,     pop_byte_arg, slice_byte_sequence, stack_value_as_bool, stack_value_as_bytes,     stack_value_as_fixed_bytes, stack_value_as_i64, stack_value_as_string, stack_value_as_u32,     stack_value_as_u8, stack_value_into_items, StackValue, COMPACT_TAG_ARRAY,     COMPACT_TAG_BIG_INTEGER, COMPACT_TAG_BOOLEAN, COMPACT_TAG_BUFFER, COMPACT_TAG_BYTESTRING,     COMPACT_TAG_INTEGER, COMPACT_TAG_INTEROP, COMPACT_TAG_ITERATOR, COMPACT_TAG_MAP,     COMPACT_TAG_NULL, COMPACT_TAG_POINTER, COMPACT_TAG_STRUCT, NEOVM_STACK_ITEM_TYPE_ANY,     NEOVM_STACK_ITEM_TYPE_ARRAY, NEOVM_STACK_ITEM_TYPE_BOOLEAN, NEOVM_STACK_ITEM_TYPE_BUFFER,     NEOVM_STACK_ITEM_TYPE_BYTESTRING, NEOVM_STACK_ITEM_TYPE_INTEGER,     NEOVM_STACK_ITEM_TYPE_INTEROP_INTERFACE, NEOVM_STACK_ITEM_TYPE_MAP,     NEOVM_STACK_ITEM_TYPE_POINTER, NEOVM_STACK_ITEM_TYPE_STRUCT, STACK_VALUE_CODEC_TAG_ARRAY,     STACK_VALUE_CODEC_TAG_BIG_INTEGER, STACK_VALUE_CODEC_TAG_BOOLEAN, STACK_VALUE_CODEC_TAG_BUFFER,     STACK_VALUE_CODEC_TAG_BYTESTRING, STACK_VALUE_CODEC_TAG_INTEGER, STACK_VALUE_CODEC_TAG_INTEROP,     STACK_VALUE_CODEC_TAG_ITERATOR, STACK_VALUE_CODEC_TAG_MAP, STACK_VALUE_CODEC_TAG_NULL,     STACK_VALUE_CODEC_TAG_POINTER, STACK_VALUE_CODEC_TAG_STRUCT, }` |
| `src/host/mod.rs: mod syscall` |
| `src/host/mod.rs: pub use syscall::{interop_hash, syscall_arg_count}` |
| `src/interpreter/executor.rs: mod byte_ops` |
| `src/interpreter/executor.rs: mod compound_ops` |
| `src/interpreter/executor.rs: mod control` |
| `src/interpreter/executor.rs: mod numeric_ops` |
| `src/interpreter/executor.rs: mod push_ops` |
| `src/interpreter/executor.rs: mod result_ops` |
| `src/interpreter/executor.rs: mod slot_ops` |
| `src/interpreter/executor.rs: mod stack_ops` |
| `src/interpreter/helpers/mod.rs: mod bridge` |
| `src/interpreter/helpers/mod.rs: mod retained` |
| `src/interpreter/helpers/mod.rs: mod values` |
| `src/interpreter/mod.rs: mod api` |
| `src/interpreter/mod.rs: mod executor` |
| `src/interpreter/mod.rs: mod helpers` |
| `src/interpreter/mod.rs: mod opcodes` |
| `src/interpreter/mod.rs: mod runtime_types` |
| `src/interpreter/mod.rs: mod state` |
| `src/interpreter/mod.rs: pub use api::{     interpret, interpret_with_stack_and_syscalls, interpret_with_stack_and_syscalls_at,     interpret_with_stack_and_syscalls_at_with_initializer,     interpret_with_stack_and_syscalls_at_with_initializer_and_result_limit,     interpret_with_stack_and_syscalls_at_with_result_limit, interpret_with_syscalls,     SyscallProvider, CALLT_MARKER, CALLT_MARKER_HI, INITIALIZER_COMPLETE_MARKER, }` |
| `src/interpreter/mod.rs: pub use state::{last_interpreter_ip, last_result_limit, last_result_stack_len, last_result_stage}` |
| `src/lib.rs: mod abi` |
| `src/lib.rs: mod host` |
| `src/lib.rs: mod interpreter` |
| `src/lib.rs: mod runtime` |
| `src/lib.rs: mod semantics` |
| `src/lib.rs: mod vm` |
| `src/lib.rs: pub use abi::{     byte_sequence_bytes, byte_sequence_len, concat_byte_sequences, default_value_for_type_tag,     encode_integer, new_array_default_value_for_type_tag, normalize_stack_item_type_tag,     pop_byte_arg, slice_byte_sequence, stack_value_as_bool, stack_value_as_bytes,     stack_value_as_fixed_bytes, stack_value_as_i64, stack_value_as_string, stack_value_as_u32,     stack_value_as_u8, stack_value_into_items, BackendKind, ExecutionResult, StackValue, VmState,     COMPACT_TAG_ARRAY, COMPACT_TAG_BIG_INTEGER, COMPACT_TAG_BOOLEAN, COMPACT_TAG_BUFFER,     COMPACT_TAG_BYTESTRING, COMPACT_TAG_INTEGER, COMPACT_TAG_INTEROP, COMPACT_TAG_ITERATOR,     COMPACT_TAG_MAP, COMPACT_TAG_NULL, COMPACT_TAG_POINTER, COMPACT_TAG_STRUCT,     NEOVM_STACK_ITEM_TYPE_ANY, NEOVM_STACK_ITEM_TYPE_ARRAY, NEOVM_STACK_ITEM_TYPE_BOOLEAN,     NEOVM_STACK_ITEM_TYPE_BUFFER, NEOVM_STACK_ITEM_TYPE_BYTESTRING, NEOVM_STACK_ITEM_TYPE_INTEGER,     NEOVM_STACK_ITEM_TYPE_INTEROP_INTERFACE, NEOVM_STACK_ITEM_TYPE_MAP,     NEOVM_STACK_ITEM_TYPE_POINTER, NEOVM_STACK_ITEM_TYPE_STRUCT, STACK_VALUE_CODEC_TAG_ARRAY,     STACK_VALUE_CODEC_TAG_BIG_INTEGER, STACK_VALUE_CODEC_TAG_BOOLEAN, STACK_VALUE_CODEC_TAG_BUFFER,     STACK_VALUE_CODEC_TAG_BYTESTRING, STACK_VALUE_CODEC_TAG_INTEGER, STACK_VALUE_CODEC_TAG_INTEROP,     STACK_VALUE_CODEC_TAG_ITERATOR, STACK_VALUE_CODEC_TAG_MAP, STACK_VALUE_CODEC_TAG_NULL,     STACK_VALUE_CODEC_TAG_POINTER, STACK_VALUE_CODEC_TAG_STRUCT, }` |
| `src/lib.rs: pub use abi::{callback_codec, fast_codec, result_codec}` |
| `src/lib.rs: pub use host::{interop_hash, syscall_arg_count}` |
| `src/lib.rs: pub use interpreter::{     interpret, interpret_with_stack_and_syscalls, interpret_with_stack_and_syscalls_at,     interpret_with_stack_and_syscalls_at_with_initializer,     interpret_with_stack_and_syscalls_at_with_initializer_and_result_limit,     interpret_with_stack_and_syscalls_at_with_result_limit, interpret_with_syscalls,     last_interpreter_ip, last_result_limit, last_result_stack_len, last_result_stage,     SyscallProvider, CALLT_MARKER, CALLT_MARKER_HI, INITIALIZER_COMPLETE_MARKER, }` |
| `src/lib.rs: pub use runtime::VmContext` |
| `src/lib.rs: pub use vm::{     OpCode, DEFAULT_MAX_INVOCATION_DEPTH, DEFAULT_MAX_STACK_DEPTH, MAX_ITEM_SIZE, MAX_SCRIPT_SIZE, }` |
| `src/semantics/mod.rs: mod arithmetic` |
| `src/semantics/mod.rs: mod collections` |
| `src/semantics/mod.rs: mod comparison` |
| `src/semantics/mod.rs: mod conversion` |
| `src/semantics/mod.rs: mod runtime` |
| `src/semantics/runtime/mod.rs: mod arithmetic` |
| `src/semantics/runtime/mod.rs: mod byte_ops` |
| `src/semantics/runtime/mod.rs: mod collections` |
| `src/semantics/runtime/mod.rs: mod comparison` |
| `src/semantics/runtime/mod.rs: mod conversion` |
| `src/semantics/runtime/mod.rs: mod stack` |
| `src/vm/mod.rs: mod limits` |
| `src/vm/mod.rs: mod opcode` |
| `src/vm/mod.rs: pub use limits::{     DEFAULT_MAX_INVOCATION_DEPTH, DEFAULT_MAX_STACK_DEPTH, MAX_ITEM_SIZE, MAX_SCRIPT_SIZE, }` |
| `src/vm/mod.rs: pub use opcode::OpCode` |

## Test Evidence

| Test | File |
| --- | --- |
| `roundtrip_integer` | `src/abi/fast_codec.rs` |
| `roundtrip_bytestring` | `src/abi/fast_codec.rs` |
| `roundtrip_buffer` | `src/abi/fast_codec.rs` |
| `roundtrip_buffer_empty` | `src/abi/fast_codec.rs` |
| `roundtrip_array` | `src/abi/fast_codec.rs` |
| `roundtrip_map` | `src/abi/fast_codec.rs` |
| `decode_rejects_excessive_nesting` | `src/abi/fast_codec.rs` |
| `decode_rejects_excessive_collection_length` | `src/abi/fast_codec.rs` |
| `bytes_use_little_endian_twos_complement` | `src/abi/stack_value.rs` |
| `integer_encoding_uses_minimal_little_endian_twos_complement` | `src/abi/stack_value.rs` |
| `stack_values_expose_stable_runtime_type_tags` | `src/abi/stack_value.rs` |
| `byte_string_conversion_bytes_follow_shared_stack_value_rules` | `src/abi/stack_value.rs` |
| `conversion_values_follow_neovm_primitive_rules` | `src/abi/stack_value.rs` |
| `neovm_type_tags_normalize_to_compact_runtime_tags` | `src/abi/stack_value.rs` |
| `default_values_follow_shared_type_tag_rules` | `src/abi/stack_value.rs` |
| `new_array_default_values_follow_neovm_rules` | `src/abi/stack_value.rs` |
| `byte_arg_pop_accepts_only_bytestring_and_buffer` | `src/abi/stack_value.rs` |
| `byte_sequence_helpers_accept_only_bytestring_and_buffer` | `src/abi/stack_value.rs` |
| `stack_value_extractors_cover_native_contract_result_shapes` | `src/abi/stack_value.rs` |
| `concat_byte_sequences_preserves_left_sequence_type` | `src/abi/stack_value.rs` |
| `slice_byte_sequence_preserves_source_type_and_checks_bounds` | `src/abi/stack_value.rs` |
| `known_syscall_hashes_match_sha256_prefix` | `src/host/syscall.rs` |
| `unknown_syscall_hashes_still_use_sha256_prefix` | `src/host/syscall.rs` |
| `known_syscall_argument_counts_match_hashes` | `src/host/syscall.rs` |
| `test_try_catch_syscall_exception` | `src/interpreter/api.rs` |
| `decode_retained_prefix` | `src/interpreter/helpers/retained.rs` |
| `retained_prefix_codec_round_trips_nested_compound_ids` | `src/interpreter/helpers/retained.rs` |
| `retained_prefix_codec_rejects_oversized_top_level_stack` | `src/interpreter/helpers/retained.rs` |
| `retained_prefix_codec_round_trips_block_like_struct_argument_list` | `src/interpreter/helpers/retained.rs` |
| `init_slot_preserves_argument_order` | `src/runtime/mod.rs` |
| `static_access_faults_out_of_range` | `src/runtime/mod.rs` |
| `arithmetic_semantics_cover_riscv_runtime_integer_ops` | `tests/abi_opcode_semantics.rs` |
| `arithmetic_semantics_return_opcode_fault_messages` | `tests/abi_opcode_semantics.rs` |
| `comparison_and_conversion_semantics_use_shared_stack_value_rules` | `tests/abi_opcode_semantics.rs` |
| `collection_semantics_cover_value_construction_and_queries` | `tests/abi_opcode_semantics.rs` |
| `collection_semantics_cover_mutation_and_stack_shaping` | `tests/abi_opcode_semantics.rs` |
| `callback_stack_result_codec_is_shared_at_vm_boundary` | `tests/boundary_codecs.rs` |
| `execution_result_codec_is_shared_at_vm_boundary` | `tests/boundary_codecs.rs` |
| `opcode_table_matches_neo_node_3_9_2_vm_package_snapshot` | `tests/canonical_opcode_vectors.rs` |
| `legacy_and_non_canonical_opcode_gaps_are_rejected` | `tests/canonical_opcode_vectors.rs` |
| `executes_basic_arithmetic_script` | `tests/interpreter_smoke.rs` |
| `executes_local_slot_round_trip` | `tests/interpreter_smoke.rs` |
| `delegates_syscall_to_host_provider` | `tests/interpreter_smoke.rs` |
| `catches_syscall_faults_with_try` | `tests/interpreter_smoke.rs` |
| `interpreter_matches_neo_vm_3_9_conformance_vectors` | `tests/neo_vm_3_9_conformance.rs` |
| `official_neo_vm_3_9_vmut_suite_matches_shared_interpreter` | `tests/official_neo_vm_3_9_vmut.rs` |
| `shared_vm_context_owns_common_state_slots_and_results` | `tests/runtime_opcode_ops.rs` |
| `shared_stack_and_byte_opcode_apis_do_not_require_context_methods` | `tests/runtime_opcode_ops.rs` |
| `runtime_arithmetic_ops_own_stack_pop_push_shape` | `tests/runtime_opcode_ops.rs` |
| `runtime_collection_ops_preserve_in_place_mutation_shape` | `tests/runtime_opcode_ops.rs` |
| `runtime_conversion_and_comparison_ops_share_vm_rules` | `tests/runtime_opcode_ops.rs` |
| `runtime_ops_report_faults_through_adapter` | `tests/runtime_opcode_ops.rs` |
| `opcode_slot_assignments_are_canonical` | `tests/shared_semantics.rs` |
| `opcode_metadata_handles_fixed_and_prefixed_operands` | `tests/shared_semantics.rs` |
| `stack_value_integer_and_boolean_semantics_are_shared` | `tests/shared_semantics.rs` |
| `syscall_hash_and_argument_counts_are_shared` | `tests/shared_semantics.rs` |
| `execution_result_keeps_fault_metadata_optional` | `tests/shared_semantics.rs` |
| `interpreter_sources_stay_small_enough_to_review` | `tests/source_layout.rs` |
| `interpreter_opcode_aliases_come_from_canonical_opcode_enum` | `tests/source_layout.rs` |
| `opcode_byte_lookup_is_generated_from_the_canonical_enum` | `tests/source_layout.rs` |
| `interpreter_uses_central_stack_item_type_tags` | `tests/source_layout.rs` |
| `internal_codecs_use_shared_stack_value_codec_tags` | `tests/source_layout.rs` |
| `known_syscall_hashes_match_neo_sha256_little_endian_rule` | `tests/syscall_vectors.rs` |
| `unknown_syscall_hashes_request_full_stack_forwarding` | `tests/syscall_vectors.rs` |
| `stack_values_round_trip_through_serde` | `tests/wire_semantics.rs` |
| `execution_result_accepts_backward_compatible_minimal_json` | `tests/wire_semantics.rs` |

## Dependency Boundary

| Dependency | Kind |
| --- | --- |
| `num-bigint` | runtime |
| `num-traits` | runtime |
| `serde` | runtime |
| `sha2` | runtime |
| `criterion` | test |
| `serde_json` | test |

## Suggested Reading Path

1. Read `src/lib.rs`: crate root, public exports, and top-level documentation.
2. Read `src/abi/stack_value.rs`: wire format, stack value, or host/guest boundary type.
3. Read `src/interpreter/state.rs`: VM interpreter and opcode semantics.
4. Read `src/interpreter/helpers/values.rs`: VM interpreter and opcode semantics.
5. Read `src/runtime/mod.rs`: execution runtime, state transition, or gas behavior.
6. Read `src/semantics/arithmetic.rs`: implementation detail or helper module.

## Change Safety Checklist

- Keep the stated responsibility boundary intact: Decode canonical opcodes, Execute stack and state semantics, Expose reusable runtime APIs.
- Update the workflow and dataflow diagrams when adding or removing major execution steps.
- Add or update tests in the files listed under Test Evidence when public API or state-transition behavior changes.
- Re-run `python tools/docs/generate_crate_visual_docs.py` from the Neo N4 repository root after source layout changes.
