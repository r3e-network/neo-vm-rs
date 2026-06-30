use std::{fs, path::Path};

const MAX_INTERPRETER_SOURCE_LINES: usize = 900;
const STACK_ITEM_TAG_MATCH_PATTERNS: &[&str] = &[
    "0x00 =>", "0x10 =>", "0x20 =>", "0x21 =>", "0x28 =>", "0x30 =>", "0x40 =>", "0x41 =>",
    "0x48 =>", "0x60 =>", "0x20 |", "0x21 |", "0x28 |", "0x30 |", "0x40 |", "0x41 |", "0x48 |",
    "0x60 |", "| 0x20", "| 0x21", "| 0x28", "| 0x30", "| 0x40", "| 0x41", "| 0x48", "| 0x60",
];

#[test]
fn interpreter_sources_stay_small_enough_to_review() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/interpreter");
    let mut oversized = Vec::new();

    collect_oversized_sources(&root, &mut oversized);

    assert!(
        oversized.is_empty(),
        "interpreter source files exceed their review-size limits: {oversized:?}"
    );
}

#[test]
fn interpreter_executor_uses_single_directory_module_root() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/interpreter");

    assert!(
        !root.join("executor.rs").exists(),
        "executor should not be split between executor.rs and executor/; keep the module root at executor/mod.rs"
    );
    assert!(
        root.join("executor").join("mod.rs").exists(),
        "executor module root should live next to its focused opcode-group submodules"
    );
}

#[test]
fn runtime_opcode_adapters_live_outside_pure_semantics_tree() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let semantics_mod = read_workspace_source("src/semantics/mod.rs");

    assert!(
        !root.join("src/semantics/runtime").exists(),
        "runtime stack adapters should live under src/runtime/ops, not under pure semantics"
    );
    assert!(
        !semantics_mod.contains("pub mod runtime"),
        "semantics should expose pure value rules only; stack adapters belong to runtime::ops"
    );
    assert!(
        root.join("src/runtime/ops/arithmetic.rs").exists()
            && root.join("src/runtime/ops/collections.rs").exists()
            && root.join("src/runtime/ops/stack.rs").exists(),
        "runtime opcode adapters should be grouped under src/runtime/ops"
    );
}

#[test]
fn runtime_opcode_adapters_expose_only_opcode_entrypoints() {
    let runtime_stack = read_workspace_source("src/runtime/ops/stack.rs");
    let runtime_comparison = read_workspace_source("src/runtime/ops/comparison.rs");
    let runtime_conversion = read_workspace_source("src/runtime/ops/conversion.rs");
    let runtime_mod = read_workspace_source("src/runtime/mod.rs");

    for helper in [
        "pub fn pick_n",
        "pub fn pop_bool",
        "pub fn pop_cmp_eq",
        "pub fn pop_cmp_ne",
        "pub fn pop_cmp_gt",
        "pub fn pop_cmp_ge",
        "pub fn pop_cmp_lt",
        "pub fn pop_cmp_le",
        "pub fn push_bigint",
        "pub fn push_default",
    ] {
        assert!(
            !runtime_stack.contains(helper)
                && !runtime_comparison.contains(helper)
                && !runtime_conversion.contains(helper),
            "runtime::ops should not expose speculative non-opcode helper {helper}"
        );
    }

    assert!(
        !runtime_mod.contains("fn pop_bool_value"),
        "RuntimeStack should not expose unused boolean-pop sugar beside shared comparison semantics"
    );
}

#[test]
fn byte_splice_semantics_are_not_reimplemented_per_layer() {
    let abi_source = read_stack_value_source();
    let splice_semantics = read_workspace_source("src/semantics/splice.rs");
    let runtime_bytes = read_workspace_source("src/runtime/ops/bytes.rs");
    let interpreter_byte_ops = read_workspace_source("src/interpreter/executor/byte_ops.rs");
    let interpreter_values = read_workspace_source("src/interpreter/helpers/values.rs");

    assert!(
        abi_source.contains("pub fn stack_value_span_bytes"),
        "ABI StackValue should expose the canonical NeoVM GetSpan byte semantics"
    );
    assert!(
        splice_semantics.contains("stack_value_span_bytes"),
        "splice semantics should own the canonical opcode-level GetSpan byte rules"
    );
    assert!(
        runtime_bytes.contains("splice_rules::") && interpreter_byte_ops.contains("splice_rules::"),
        "runtime and interpreter byte opcode adapters should both reuse semantics::splice"
    );
    assert!(
        interpreter_values.contains("stack_value_span_bytes"),
        "interpreter byte helpers should reuse ABI span semantics instead of duplicating primitive matches"
    );
    assert!(
        !abi_source.contains("concat_byte_sequences")
            && !abi_source.contains("slice_byte_sequence"),
        "ABI must not keep legacy type-preserving byte CAT/SUBSTR helpers beside NeoVM GetSpan splice semantics"
    );
    assert!(
        !interpreter_values.contains("Some(StackValue::ByteString(value)) => Ok(value)")
            && !interpreter_values.contains("Some(StackValue::Buffer(_, value)) => Ok(value)"),
        "interpreter byte helpers should not keep a second byte-conversion match table"
    );
    for duplicate in [
        "result_bytes.extend_from_slice",
        "bytes[..count].to_vec()",
        "bytes[index..end].to_vec()",
    ] {
        assert!(
            !interpreter_byte_ops.contains(duplicate),
            "interpreter byte_ops should not reimplement splice byte manipulation: {duplicate}"
        );
    }
}

#[test]
fn truthiness_semantics_delegate_to_stack_value() {
    let comparison = read_workspace_source("src/semantics/comparison.rs");

    assert!(
        comparison.contains("value.to_bool()"),
        "boolean_value should delegate to StackValue::to_bool instead of duplicating truthiness rules"
    );
}

#[test]
fn interpreter_struct_equality_reuses_runtime_value_graph_comparison() {
    let values = read_workspace_source("src/interpreter/helpers/values.rs");

    assert!(
        values.contains("structurally_equal(left, right)"),
        "interpreter Struct equality should reuse the shared runtime value graph comparison"
    );
    assert!(
        !values.contains("fn struct_equal"),
        "interpreter helpers should not duplicate recursive Struct equality"
    );
}

#[test]
fn numeric_stack_shapes_are_shared_between_runtime_and_interpreter() {
    let runtime_ops_mod = read_workspace_source("src/runtime/ops/mod.rs");
    let runtime_arithmetic = read_workspace_source("src/runtime/ops/arithmetic.rs");
    let runtime_comparison = read_workspace_source("src/runtime/ops/comparison.rs");
    let interpreter_numeric = read_workspace_source("src/interpreter/executor/numeric_ops.rs");

    assert!(
        runtime_ops_mod.contains("stack_shape::")
            && runtime_arithmetic.contains("runtime_binary_value_op!")
            && runtime_comparison.contains("runtime_binary_bool_op!")
            && interpreter_numeric.contains("stack_shape::"),
        "runtime and interpreter numeric opcode adapters should share stack_shape helpers"
    );
    assert!(
        runtime_arithmetic.contains("runtime_ternary_bool_op!(within, rules::within_values);")
            && interpreter_numeric.contains(
                "stack_shape::ternary_bool(&mut value_stack, arithmetic_rules::within_values)"
            ),
        "runtime and interpreter WITHIN adapters should share the ternary bool stack shape"
    );

    for duplicate in [
        "fn unary_value(",
        "fn binary_value(",
        "fn ternary_value(",
        "fn binary_bool(",
        "fn bool_binary(",
        "match rules::within_values",
    ] {
        assert!(
            !interpreter_numeric.contains(duplicate),
            "interpreter numeric_ops should not keep private stack-shape helper {duplicate}"
        );
    }
    assert!(
        !runtime_arithmetic.contains("match rules::within_values"),
        "runtime arithmetic should not hand-roll the WITHIN stack shape"
    );
}

#[test]
fn runtime_numeric_stack_shape_adapters_are_generated() {
    let runtime_ops_mod = read_workspace_source("src/runtime/ops/mod.rs");
    let runtime_arithmetic = read_workspace_source("src/runtime/ops/arithmetic.rs");
    let runtime_comparison = read_workspace_source("src/runtime/ops/comparison.rs");

    for macro_name in [
        "macro_rules! runtime_unary_value_op",
        "macro_rules! runtime_binary_value_op",
        "macro_rules! runtime_ternary_value_op",
        "macro_rules! runtime_unary_bool_op",
        "macro_rules! runtime_binary_bool_op",
        "macro_rules! runtime_binary_bool_value_op",
        "macro_rules! runtime_ternary_bool_op",
    ] {
        assert!(
            runtime_ops_mod.contains(macro_name),
            "runtime stack-shape adapter boilerplate should be generated by {macro_name}"
        );
    }

    for duplicate in [
        "stack_shape::unary_value(stack",
        "stack_shape::binary_value(stack",
        "stack_shape::ternary_value(stack",
        "stack_shape::unary_bool(stack",
        "stack_shape::binary_bool(stack",
        "stack_shape::ternary_bool(stack",
    ] {
        assert!(
            !runtime_arithmetic.contains(duplicate) && !runtime_comparison.contains(duplicate),
            "runtime opcode adapters should not repeat stack-shape wrapper body {duplicate}"
        );
    }
}

#[test]
fn numeric_stack_shape_helpers_live_with_semantics_not_runtime_adapters() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let semantics_mod = read_workspace_source("src/semantics/mod.rs");
    let runtime_ops_mod = read_workspace_source("src/runtime/ops/mod.rs");
    let runtime_arithmetic = read_workspace_source("src/runtime/ops/arithmetic.rs");
    let interpreter_numeric = read_workspace_source("src/interpreter/executor/numeric_ops.rs");

    assert!(
        semantics_mod.contains("pub(crate) mod stack_shape;"),
        "shared value-stack opcode shapes should live under semantics::stack_shape"
    );
    assert!(
        !root.join("src/runtime/ops/value_stack.rs").exists(),
        "runtime::ops should not own the shared value-stack opcode shape helpers"
    );
    assert!(
        runtime_ops_mod.contains("impl<R: RuntimeStack + ?Sized> ValueStack for R"),
        "runtime::ops should only adapt RuntimeStack into the shared ValueStack abstraction"
    );
    assert!(
        runtime_arithmetic.contains("runtime_binary_value_op!(add, rules::add_values);"),
        "runtime numeric adapters should call semantics::stack_shape helpers"
    );
    assert!(
        !interpreter_numeric.contains("runtime::ops::value_stack"),
        "interpreter numeric execution should not depend on runtime adapter internals"
    );
    assert!(
        interpreter_numeric.contains("semantics::{")
            && interpreter_numeric.contains("stack_shape::{self, ValueStack}"),
        "interpreter numeric execution should import shared stack shapes from semantics"
    );
}

#[test]
fn arithmetic_semantics_expose_stack_value_rules_not_parallel_i64_api() {
    let arithmetic = read_workspace_source("src/semantics/arithmetic.rs");

    for helper in [
        "pub fn add_i64(",
        "pub fn sub_i64(",
        "pub fn mul_i64(",
        "pub fn div_i64(",
        "pub fn modulo_i64(",
        "pub fn negate_i64(",
        "pub fn abs_i64(",
        "pub fn sign_i64(",
        "pub fn max_i64(",
        "pub fn min_i64(",
        "pub fn pow_i64(",
        "pub fn sqrt_i64(",
        "pub fn modmul_i64(",
        "pub fn modpow_i64(",
        "pub fn shl_i64(",
        "pub fn shr_i64(",
        "pub fn bitwise_and_i64(",
        "pub fn bitwise_or_i64(",
        "pub fn bitwise_xor_i64(",
        "pub fn bitwise_not_i64(",
        "pub fn inc_i64(",
        "pub fn dec_i64(",
        "pub fn within_i64(",
    ] {
        assert!(
            !arithmetic.contains(helper),
            "arithmetic semantics should expose canonical StackValue rules only, not parallel i64 helper {helper}"
        );
    }
}

#[test]
fn comparison_semantics_expose_stack_value_rules_not_parallel_i64_api() {
    let comparison = read_workspace_source("src/semantics/comparison.rs");

    for helper in [
        "pub fn less_than_i64(",
        "pub fn less_or_equal_i64(",
        "pub fn greater_than_i64(",
        "pub fn greater_or_equal_i64(",
        "pub fn num_equal_i64(",
        "pub fn num_not_equal_i64(",
    ] {
        assert!(
            !comparison.contains(helper),
            "comparison semantics should expose canonical StackValue rules only, not parallel i64 helper {helper}"
        );
    }
}

#[test]
fn comparison_semantics_do_not_expose_ambiguous_boolean_aliases() {
    let comparison = read_workspace_source("src/semantics/comparison.rs");

    for helper in ["pub fn bool_not(", "pub fn nz("] {
        assert!(
            !comparison.contains(helper),
            "comparison semantics should expose opcode-specific not_value/nz_value or boolean_value, not ambiguous alias {helper}"
        );
    }
}

#[test]
fn comparison_semantics_keep_one_numeric_operand_decoder() {
    let comparison = read_workspace_source("src/semantics/comparison.rs");

    let decode_count = comparison
        .matches("numeric::decode_signed_le_bytes_bigint(value)")
        .count();
    assert_eq!(
        decode_count, 1,
        "comparison semantics should decode primitive numeric operands through one helper, not parallel match tables"
    );
}

#[test]
fn vm_context_exposes_one_canonical_boundary_name_per_operation() {
    let runtime_mod = read_workspace_source("src/runtime/mod.rs");

    for alias in ["pub fn from_abi_stack(", "pub fn to_execution_result("] {
        assert!(
            !runtime_mod.contains(alias),
            "VmContext should expose canonical from_stack/into_execution_result only, not compatibility alias {alias}"
        );
    }
}

#[test]
fn stack_opcode_shapes_are_shared_between_runtime_and_interpreter() {
    let stack_semantics = read_workspace_source("src/semantics/stack.rs");
    let runtime_stack = read_workspace_source("src/runtime/ops/stack.rs");
    let interpreter_stack = read_workspace_source("src/interpreter/executor/stack_ops.rs");

    assert!(
        stack_semantics.contains("pub(crate) fn dup")
            && stack_semantics.contains("pub(crate) fn roll"),
        "semantics::stack should own shared VM stack transformations"
    );
    assert!(
        runtime_stack.contains("stack_rules::") && interpreter_stack.contains("stack_rules::"),
        "runtime and interpreter stack opcode adapters should both reuse semantics::stack"
    );

    for duplicate in [
        "stack.swap(last, last - 1)",
        "stack.insert(stack.len() - 2",
        "stack[start..].reverse()",
        "let idx = stack.len() - 1 - n",
    ] {
        assert!(
            !interpreter_stack.contains(duplicate),
            "interpreter stack_ops should not reimplement stack manipulation: {duplicate}"
        );
    }
}

#[test]
fn interpreter_fault_results_are_centralized() {
    let executor = read_workspace_source("src/interpreter/executor/mod.rs");
    let result_ops = read_workspace_source("src/interpreter/executor/result_ops.rs");

    assert!(
        result_ops.contains("pub(super) fn fault_result("),
        "result_ops should own interpreter Fault ExecutionResult construction"
    );
    assert!(
        !executor.contains("ExecutionResult {"),
        "interpreter executor loop should not duplicate Fault ExecutionResult literals"
    );
    assert!(
        !executor.contains("fast_codec::encode_stack(&to_abi_stack(&cloned))"),
        "fault locals encoding should be centralized with the fault result constructor"
    );
}

#[test]
fn collection_constructor_rules_are_not_reimplemented_per_layer() {
    let runtime_collections = read_workspace_source("src/runtime/ops/collections.rs");
    let interpreter_byte_ops = read_workspace_source("src/interpreter/executor/byte_ops.rs");
    let interpreter_compound_ops =
        read_workspace_source("src/interpreter/executor/compound_ops.rs");
    let collection_semantics = read_workspace_source("src/semantics/collections.rs");

    assert!(
        runtime_collections.contains("rules::new_array(0)")
            && runtime_collections.contains("rules::new_struct(0)")
            && runtime_collections.contains("rules::new_map()"),
        "runtime collection constructors should use semantics::collections instead of constructing ABI values directly"
    );
    assert!(
        interpreter_byte_ops.contains("collection_rules::new_buffer(count)")
            && !interpreter_byte_ops.contains("1_048_576"),
        "interpreter NEWBUFFER should reuse collection constructor limits instead of carrying a private size check"
    );
    assert!(
        interpreter_compound_ops.contains("new_array_default_value_for_neovm_type_tag(kind)")
            && !interpreter_compound_ops
                .contains("NEOVM_STACK_ITEM_TYPE_INTEGER => StackValue::Integer(0)")
            && !interpreter_compound_ops
                .contains("NEOVM_STACK_ITEM_TYPE_BYTESTRING => StackValue::ByteString"),
        "interpreter NEWARRAY_T should reuse the shared default-value rule"
    );
    assert!(
        collection_semantics.contains("pub(crate) fn non_negative_count"),
        "collection semantics should expose the shared non-negative count rule"
    );
    assert!(
        runtime_collections.contains("rules::non_negative_count")
            && !runtime_collections.contains("fn positive_count"),
        "runtime collection adapters should not keep a private count validator"
    );
    assert!(
        interpreter_compound_ops.contains("collection_rules::non_negative_count")
            && !interpreter_compound_ops.contains("if count < 0"),
        "interpreter compound collection opcodes should reuse the shared non-negative count rule"
    );
}

#[test]
fn runtime_byte_result_faulting_is_centralized() {
    let runtime_bytes = read_workspace_source("src/runtime/ops/bytes.rs");
    let value_result_ops = runtime_bytes
        .split("pub fn memcpy")
        .next()
        .expect("bytes adapter should define memcpy after value-returning ops");

    assert!(
        value_result_ops.contains("push_value_result"),
        "runtime byte adapters should use the common Result<StackValue> push/fault helper"
    );
    assert!(
        !value_result_ops.contains("Err(message) => runtime.fault(&message)"),
        "CAT/SUBSTR/LEFT/RIGHT should not repeat Result-to-fault match bodies"
    );
}

#[test]
fn runtime_collection_pack_shapes_are_shared() {
    let runtime_collections = read_workspace_source("src/runtime/ops/collections.rs");

    assert!(
        runtime_collections.contains("fn pop_non_negative_count")
            && runtime_collections.contains("fn pop_values"),
        "PACK, PACKSTRUCT, and PACKMAP should share count validation and value popping helpers"
    );
    assert_eq!(
        runtime_collections
            .matches("rules::non_negative_count")
            .count(),
        1,
        "runtime collection pack opcodes should not repeat non-negative count validation"
    );
}

#[test]
fn runtime_collection_result_adapters_are_centralized() {
    let runtime_mod = read_workspace_source("src/runtime/mod.rs");
    let runtime_collections = read_workspace_source("src/runtime/ops/collections.rs");

    for helper in [
        "pub(crate) fn push_i64_result",
        "pub(crate) fn push_bool_result",
        "pub(crate) fn push_values_result",
    ] {
        assert!(
            runtime_mod.contains(helper),
            "runtime should centralize scalar/vector Result push adapters through {helper}"
        );
    }
    assert!(
        runtime_collections.contains("fn apply_top_mut"),
        "mutable collection opcodes should share one top-value mutation adapter"
    );
    assert_eq!(
        runtime_collections.matches("top_value_mut()").count(),
        1,
        "runtime collection adapters should not repeat top-value mutation dispatch"
    );
    for duplicate in [
        "match rules::size",
        "match rules::has_key",
        "for value in values",
        "if let Err(message)",
    ] {
        assert!(
            !runtime_collections.contains(duplicate),
            "runtime collection adapters should not repeat Result/fault plumbing: {duplicate}"
        );
    }
}

#[test]
fn interpreter_map_key_lookup_reuses_collection_semantics() {
    let interpreter_compound_ops =
        read_workspace_source("src/interpreter/executor/compound_ops.rs");
    let collection_semantics = read_workspace_source("src/semantics/collections.rs");

    assert!(
        collection_semantics.contains("pub fn map_entry_index"),
        "collection semantics should own primitive map-key lookup"
    );
    assert!(
        interpreter_compound_ops.contains("collection_rules::map_entry_index"),
        "interpreter map opcodes should reuse shared primitive map-key lookup"
    );
    for duplicate in [
        ".find(|(candidate, _)| primitive_key_equals",
        ".position(|(candidate, _)| primitive_key_equals",
    ] {
        assert!(
            !interpreter_compound_ops.contains(duplicate),
            "interpreter compound_ops should not reimplement map-key lookup: {duplicate}"
        );
    }
}

#[test]
fn interpreter_collection_mutations_commit_through_one_helper() {
    let interpreter_compound_ops =
        read_workspace_source("src/interpreter/executor/compound_ops.rs");

    assert!(
        interpreter_compound_ops.contains("fn commit_collection_update("),
        "compound_ops should centralize consumed-mutation tracking and alias propagation"
    );
    assert_eq!(
        interpreter_compound_ops
            .matches("remember_consumed_mutation(")
            .count(),
        1,
        "collection mutation opcodes should not repeat consumed-mutation tracking"
    );
    assert_eq!(
        interpreter_compound_ops
            .matches("propagate_update(")
            .count(),
        1,
        "collection mutation opcodes should not repeat alias propagation"
    );
}

#[test]
fn interpreter_opcode_aliases_come_from_canonical_opcode_enum() {
    let source = read_workspace_source("src/interpreter/opcodes.rs");

    assert!(
        source.contains("use crate::OpCode;"),
        "interpreter opcode aliases must derive from the canonical OpCode enum"
    );
    assert!(
        source.contains("OpCode::$name.byte()") && source.contains("opcode_alias!(PUSHINT8)"),
        "opcode aliases should be typed enum aliases, not an independent byte table"
    );
    assert!(
        !source.contains("= 0x"),
        "interpreter opcode aliases must not duplicate hard-coded opcode bytes"
    );
}

#[test]
fn opcode_byte_lookup_is_generated_from_the_canonical_enum() {
    let source = read_workspace_source("src/vm/opcode.rs");

    assert!(
        source.contains("const LOOKUP: [Option<Self>; 256]"),
        "OpCode byte decoding should use a generated lookup table"
    );
    assert!(
        source.contains("Self::LOOKUP[value as usize].ok_or(value)"),
        "TryFrom<u8> should read from the generated opcode lookup table"
    );
    assert!(
        !source.contains("0x00 => Self::PUSHINT8"),
        "TryFrom<u8> must not duplicate the opcode byte assignment table"
    );
}

#[test]
fn opcode_metadata_comes_from_one_declaration_table() {
    let source = read_workspace_source("src/vm/opcode.rs");

    assert!(
        source.contains("macro_rules! define_opcodes"),
        "opcode enum, ALL order, names, and operand metadata should be generated from one table"
    );
    assert!(
        source.contains("define_opcodes! {"),
        "opcode.rs should keep a single canonical opcode declaration table"
    );
    assert!(
        !source.contains("Self::PUSHINT8,\r\n") && !source.contains("Self::PUSHINT8,\n"),
        "OpCode::ALL must not be a second hand-maintained opcode list"
    );
    assert!(
        !source.contains("Self::PUSHINT8 => \"PUSHINT8\""),
        "opcode names should come from the declaration identifier, not a second match table"
    );
}

#[test]
fn interpreter_uses_central_stack_item_type_tags() {
    let sources = [
        read_workspace_source("src/interpreter/executor/compound_ops.rs"),
        read_workspace_source("src/interpreter/helpers/values.rs"),
    ];

    for source in sources {
        for pattern in STACK_ITEM_TAG_MATCH_PATTERNS {
            assert!(
                !source.contains(pattern),
                "interpreter code should use NEOVM_STACK_ITEM_TYPE_* constants instead of literal pattern {pattern}"
            );
        }
    }
}

#[test]
fn interpreter_type_checks_reuse_shared_semantics() {
    let compound_ops = read_workspace_source("src/interpreter/executor/compound_ops.rs");
    let values = read_workspace_source("src/interpreter/helpers/values.rs");

    assert!(
        values.contains("pub(crate) fn is_type(")
            && values.contains("conversion_rules::is_type(value, kind)"),
        "interpreter ISTYPE should route through a helper backed by semantics::conversion"
    );
    assert!(
        values.contains("pub(crate) fn is_null(")
            && values.contains("comparison_rules::is_null(value)"),
        "interpreter ISNULL should route through a helper backed by semantics::comparison"
    );
    assert!(
        compound_ops.contains("let result = is_type(kind, &item)?;")
            && compound_ops.contains("StackValue::Boolean(is_null(&item))"),
        "compound_ops should call shared type-check helpers instead of matching StackValue variants directly"
    );

    for duplicate in [
        "matches!(item, StackValue::Pointer(_))",
        "matches!(item, StackValue::Boolean(_))",
        "matches!(item, StackValue::Integer(_) | StackValue::BigInteger(_))",
        "matches!(item, StackValue::ByteString(_))",
        "matches!(item, StackValue::Buffer(_, _))",
        "matches!(item, StackValue::Array(_, _))",
        "matches!(item, StackValue::Struct(_, _))",
        "matches!(item, StackValue::Map(_, _))",
        "matches!(item, StackValue::Interop(_))",
        "matches!(item, StackValue::Null)",
    ] {
        assert!(
            !compound_ops.contains(duplicate),
            "compound_ops should not keep a private type-check match: {duplicate}"
        );
    }
}

#[test]
fn internal_codecs_use_shared_stack_value_codec_tags() {
    let fast_codec = read_workspace_source("src/abi/fast_codec.rs");
    let helpers_mod = read_workspace_source("src/interpreter/helpers/mod.rs");
    let retained_codec = read_workspace_source("src/interpreter/helpers/retained.rs");

    assert!(
        fast_codec.contains("STACK_VALUE_CODEC_TAG_INTEGER"),
        "fast codec should import the shared stack-value codec tag constants"
    );
    assert!(
        !fast_codec.contains("const TAG_"),
        "fast codec must not duplicate compact type tag byte values"
    );
    assert!(
        !helpers_mod.contains("const RETAINED_TAG_"),
        "retained-prefix codec must not define a second compact type tag table"
    );
    assert!(
        !retained_codec.contains("RETAINED_TAG_")
            && retained_codec.contains("STACK_VALUE_CODEC_TAG_INTEGER"),
        "retained-prefix codec should use STACK_VALUE_CODEC_TAG_* directly"
    );
}

#[test]
fn syscall_metadata_comes_from_one_declaration_table() {
    let syscall = read_workspace_source("src/host/syscall.rs");

    assert!(
        syscall.contains("macro_rules! define_syscalls"),
        "syscall hashes and argument counts should be generated from one metadata declaration"
    );
    assert!(
        syscall.contains("define_syscalls! {"),
        "syscall.rs should keep one canonical syscall metadata table"
    );
    assert!(
        !syscall.contains("0x525b_7d62 => 4")
            && !syscall.contains("0x677b_f71a => usize::MAX")
            && !syscall.contains("0x8cec_27f8 => 1"),
        "syscall_arg_count must not duplicate hash literals from known_interop_hash"
    );
    assert!(
        !syscall.contains("\"System.Contract.Call\" => Some(0x525b_7d62)")
            && !syscall.contains("\"System.Runtime.CheckWitness\" => Some(0x8cec_27f8)"),
        "known_interop_hash must not be a second handwritten hash table"
    );
}

#[test]
fn pending_exception_state_is_shared_between_runtime_and_interpreter() {
    let runtime_mod = read_workspace_source("src/runtime/mod.rs");
    let runtime_exception = read_workspace_source("src/runtime/pending_exception.rs");
    let interpreter_state = read_workspace_source("src/interpreter/state.rs");

    assert!(
        runtime_mod.contains("pub(crate) use pending_exception::")
            && runtime_mod.contains("PendingException"),
        "runtime should expose the shared pending-exception representation crate-internally"
    );
    assert!(
        runtime_exception.contains("pub(crate) enum PendingException<V>"),
        "pending_exception.rs should own the single generic PendingException type"
    );
    assert!(
        interpreter_state.contains(
            "pub(super) type PendingException = crate::runtime::PendingException<StackValue>;"
        ),
        "interpreter state should alias the runtime PendingException instead of redefining it"
    );
    assert!(
        !interpreter_state.contains("enum PendingException"),
        "interpreter state must not keep a duplicate PendingException definition"
    );
}

#[test]
fn interpreter_numeric_helpers_call_canonical_rules_directly() {
    let values = read_workspace_source("src/interpreter/helpers/values.rs");
    let push_ops = read_workspace_source("src/interpreter/executor/push_ops.rs");
    let stack_value = read_stack_value_source();

    for duplicate in [
        "fn decode_signed_le_bytes(",
        "fn encode_integer(",
        "fn trim_le_bytes_slice(",
    ] {
        assert!(
            !values.contains(duplicate),
            "interpreter helpers should not keep forwarding numeric helper {duplicate}"
        );
    }

    assert!(
        values.contains("numeric::decode_signed_le_bytes_i64")
            && values.contains("crate::abi::encode_integer"),
        "interpreter value helpers should call canonical numeric and ABI integer rules directly"
    );
    assert_eq!(
        values
            .matches("numeric::decode_signed_le_bytes_i64")
            .count(),
        1,
        "interpreter value helpers should keep one primitive i64 decoder table shared by integer and shift-count pops"
    );
    assert!(
        push_ops.contains("numeric::trim_le_bytes_slice"),
        "PUSHINT128/PUSHINT256 should trim through canonical numeric rules directly"
    );
    assert!(
        stack_value.contains("numeric::decode_signed_le_bytes_i128")
            && !stack_value.contains("fn little_endian_twos_complement_i128"),
        "ABI StackValue should not keep a private two's-complement decoder beside semantics::numeric"
    );
}

#[test]
fn host_call_retained_state_is_centralized() {
    let bridge = read_workspace_source("src/interpreter/helpers/bridge.rs");

    assert!(
        bridge.contains("struct RetainedHostState"),
        "bridge host calls should capture retained interpreter state through one shared state object"
    );
    assert!(
        bridge.contains("fn retain_values("),
        "bridge host calls should use one retained-value capture helper instead of repeating cfg/buffer blocks"
    );
    assert!(
        bridge.contains("fn restore_non_stack(")
            && bridge.contains("fn restore_non_stack_best_effort("),
        "shared retained state should own strict and best-effort non-stack restoration"
    );

    for duplicate in [
        "encode_retained_prefix_to_slice(locals",
        "encode_retained_prefix_to_slice(args",
        "encode_retained_prefix_to_slice(static_fields",
        "encode_retained_prefix_to_slice(consumed_mutations",
        "cfg!(target_arch = \"riscv32\") && !locals.is_empty()",
        "cfg!(target_arch = \"riscv32\") && !args.is_empty()",
        "cfg!(target_arch = \"riscv32\") && !static_fields.is_empty()",
        "cfg!(target_arch = \"riscv32\") && !consumed_mutations.is_empty()",
    ] {
        assert!(
            !bridge.contains(duplicate),
            "bridge host calls should not duplicate retained-state capture: {duplicate}"
        );
    }
}

#[test]
fn abi_codecs_share_binary_cursor() {
    let abi_mod = read_workspace_source("src/abi/mod.rs");
    let cursor = read_workspace_source("src/abi/cursor.rs");
    let result_codec = read_workspace_source("src/abi/result_codec.rs");
    let callback_codec = read_workspace_source("src/abi/callback_codec.rs");

    assert!(
        abi_mod.contains("mod cursor;"),
        "ABI should keep shared binary cursor utilities in src/abi/cursor.rs"
    );
    assert!(
        cursor.contains("pub(super) struct Cursor<'a>")
            && cursor.contains("pub(super) fn read_u64(")
            && cursor.contains("pub(super) fn expect_eof("),
        "shared ABI cursor should own primitive little-endian reads and EOF checks"
    );

    for (name, source) in [
        ("result_codec", result_codec.as_str()),
        ("callback_codec", callback_codec.as_str()),
    ] {
        assert!(
            source.contains("use super::cursor::Cursor;"),
            "{name} should import the shared ABI cursor"
        );
        assert!(
            !source.contains("struct Cursor<'a>"),
            "{name} should not define a private cursor copy"
        );
        assert!(
            !source.contains("fn read_u32(&mut self)")
                && !source.contains("fn read_u64(&mut self)")
                && !source.contains("fn read_exact(&mut self"),
            "{name} should not duplicate cursor primitive readers"
        );
    }
}

#[test]
fn fast_codec_decoder_uses_local_primitive_read_helpers() {
    let fast_codec = read_workspace_source("src/abi/fast_codec.rs");

    assert!(
        fast_codec.contains("fn read_exact<'a>(")
            && fast_codec.contains("fn read_u32(")
            && fast_codec.contains("fn read_u64(")
            && fast_codec.contains("fn read_len_prefixed_bytes("),
        "fast_codec should centralize decoder bounds checks and little-endian primitive reads"
    );

    let decoder = fast_codec
        .split("fn decode_value_depth")
        .nth(1)
        .expect("fast_codec should define decode_value_depth");
    for duplicate in [
        "bytes.len() < pos + 4",
        "bytes.len() < pos + 8",
        "bytes[pos + 7]",
        "u32::from_le_bytes([bytes[pos]",
    ] {
        assert!(
            !decoder.contains(duplicate),
            "fast_codec decode_value_depth should not duplicate primitive read logic: {duplicate}"
        );
    }
}

#[test]
fn fast_codec_encoder_uses_one_stack_value_match() {
    let fast_codec = read_workspace_source("src/abi/fast_codec.rs");

    assert!(
        fast_codec.contains("trait FastEncodeSink")
            && fast_codec.contains("fn encode_value_into<S: FastEncodeSink>")
            && fast_codec.contains("struct VecSink")
            && fast_codec.contains("struct SliceSink"),
        "fast_codec should encode StackValue through one shared sink abstraction for Vec and slice outputs"
    );
    assert!(
        !fast_codec.contains("fn encode_value(value: &StackValue, buf: &mut Vec<u8>)"),
        "Vec encoding should not keep a private StackValue match beside slice encoding"
    );
    assert!(
        !fast_codec.contains("fn encode_value_to_slice("),
        "slice encoding should not keep a private StackValue match beside Vec encoding"
    );

    let match_count = fast_codec.matches("match value {").count();
    assert_eq!(
        match_count, 1,
        "fast_codec should have exactly one StackValue encoding match table"
    );
}

fn collect_oversized_sources(dir: &Path, oversized: &mut Vec<(String, usize)>) {
    for entry in fs::read_dir(dir).expect("interpreter directory should be readable") {
        let entry = entry.expect("interpreter entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_oversized_sources(&path, oversized);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }

        let source = fs::read_to_string(&path).expect("source file should be UTF-8");
        let line_count = source.lines().count();
        let limit = MAX_INTERPRETER_SOURCE_LINES;

        if line_count > limit {
            oversized.push((
                path.strip_prefix(Path::new(env!("CARGO_MANIFEST_DIR")))
                    .unwrap_or(&path)
                    .display()
                    .to_string(),
                line_count,
            ));
        }
    }
}

fn read_workspace_source(relative_path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path))
        .unwrap_or_else(|error| panic!("source file {relative_path} should be readable: {error}"))
}

/// Read the combined source of the `stack_value` directory module.
///
/// `stack_value` was split from a single file into a directory module
/// (`mod.rs` plus `tags.rs` and `ops.rs`); concatenating the submodules lets
/// the structural assertions keep matching content regardless of which
/// submodule now owns it.
fn read_stack_value_source() -> String {
    let mut combined = String::new();
    for submodule in ["src/abi/stack_value/mod.rs", "src/abi/stack_value/ops.rs"] {
        combined.push_str(&read_workspace_source(submodule));
        combined.push('\n');
    }
    combined
}
