use std::{fs, path::Path};

const MAX_INTERPRETER_SOURCE_LINES: usize = 1_000;
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
