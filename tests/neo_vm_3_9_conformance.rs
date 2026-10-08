use std::collections::HashSet;

use neo_vm_rs::reference::{SharedHeap, SharedValue};
use neo_vm_rs::{
    ReferenceSyscallProvider, SharedVmException, interpret,
    interpret_with_stack_and_syscalls_at_reference,
};

#[path = "neo_vm_3_9_conformance/suite.rs"]
mod suite;
#[path = "neo_vm_3_9_conformance/vector.rs"]
mod vector;

use suite::Suite;

const FIXTURE: &str = include_str!("fixtures/neo_vm_3_9_conformance.json");

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

struct NoReferenceSyscalls;
impl ReferenceSyscallProvider for NoReferenceSyscalls {
    fn syscall(
        &mut self,
        api: u32,
        _ip: usize,
        _depth: usize,
        _stack: &mut Vec<SharedValue>,
    ) -> Result<(), SharedVmException> {
        Err(SharedVmException::Fault(format!(
            "unexpected syscall {api:08x}"
        )))
    }
}

#[test]
fn reference_interpreter_matches_all_pinned_conformance_vectors() {
    let suite: Suite = serde_json::from_str(FIXTURE).expect("fixture should be valid JSON");
    assert_eq!(suite.vectors.len(), 20);
    for vector in suite.vectors {
        let script = decode_hex(&vector.script_hex).unwrap();
        let result = interpret_with_stack_and_syscalls_at_reference(
            &script,
            vec![],
            0,
            SharedHeap::new(),
            1,
            &mut NoReferenceSyscalls,
        );
        match result {
            Ok(result) => {
                assert_eq!(result.state, vector.expected_state, "{}", vector.name);
                let actual = result
                    .stack
                    .iter()
                    .map(|value| value.snapshot(2048).unwrap())
                    .collect::<Vec<_>>();
                assert_eq!(actual, vector.expected_stack, "{}", vector.name);
            }
            Err(error) => assert_eq!(
                vector.expected_state,
                neo_vm_rs::VmState::Fault,
                "{}: {error}",
                vector.name
            ),
        }
    }
}
