use std::collections::HashSet;

use neo_vm_rs::{interpret, StackValue, VmState};
use serde::Deserialize;

const FIXTURE: &str = include_str!("fixtures/neo_vm_3_9_conformance.json");

#[derive(Debug, Deserialize)]
struct Suite {
    neo_node_tag: String,
    neo_package_version: String,
    neo_vm_package_version: String,
    source: String,
    vectors: Vec<Vector>,
}

#[derive(Debug, Deserialize)]
struct Vector {
    name: String,
    description: String,
    script_hex: String,
    expected_state: VmState,
    expected_stack: Vec<StackValue>,
}

#[test]
fn interpreter_matches_neo_vm_3_9_conformance_vectors() {
    let suite: Suite = serde_json::from_str(FIXTURE).expect("fixture should be valid JSON");

    assert_eq!(suite.neo_node_tag, "v3.9.2");
    assert_eq!(suite.neo_package_version, "Neo 3.9.1");
    assert_eq!(suite.neo_vm_package_version, "Neo.VM 3.9.0");
    assert!(suite.source.contains("Neo.VM 3.9.0"));
    assert_eq!(suite.vectors.len(), 20);

    let mut names = HashSet::new();
    for vector in suite.vectors {
        assert!(names.insert(vector.name.clone()), "duplicate vector name");
        assert!(!vector.description.trim().is_empty(), "{}", vector.name);

        let script = decode_hex(&vector.script_hex)
            .unwrap_or_else(|err| panic!("{} has invalid script hex: {err}", vector.name));
        let result = interpret(&script)
            .unwrap_or_else(|err| panic!("{} failed to execute: {err}", vector.name));

        assert_eq!(result.state, vector.expected_state, "{}", vector.name);
        assert_eq!(result.stack, vector.expected_stack, "{}", vector.name);
    }
}

fn decode_hex(hex: &str) -> Result<Vec<u8>, String> {
    if !hex.len().is_multiple_of(2) {
        return Err("hex string has odd length".to_string());
    }

    (0..hex.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&hex[index..index + 2], 16)
                .map_err(|err| format!("invalid byte at offset {index}: {err}"))
        })
        .collect()
}
