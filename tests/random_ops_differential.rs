use std::{collections::HashSet, path::Path, process::Command};

use neo_vm_rs::{VmState, interpret};

#[path = "neo_vm_3_9_conformance/suite.rs"]
mod suite;
#[path = "neo_vm_3_9_conformance/vector.rs"]
mod vector;

#[test]
#[ignore = "requires the built .NET 10 NeoVm39ConformanceGenerator; see tools/NeoVm39ConformanceGenerator/DIFFERENTIAL.md"]
fn random_ops_scripts_match_live_csharp_neovm() {
    let dll = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "tools/NeoVm39ConformanceGenerator/bin/Debug/net10.0/NeoVm39ConformanceGenerator.dll",
    );
    assert!(
        dll.is_file(),
        "build the C# oracle first: {}",
        dll.display()
    );

    let known_families: HashSet<&str> = [
        "negate",
        "abs",
        "sub",
        "div",
        "mod",
        "cat",
        "left",
        "right",
        "substr",
        "fault_empty_add",
        "fault_empty_abs",
        "fault_underflow_sub",
    ]
    .into_iter()
    .collect();
    // Family -> minimum terminal state it must reach (structural sanity gate).
    let expected_state: std::collections::HashMap<&str, VmState> = [
        ("negate", VmState::Halt),
        ("abs", VmState::Halt),
        ("sub", VmState::Halt),
        ("cat", VmState::Halt),
        ("fault_empty_add", VmState::Fault),
        ("fault_empty_abs", VmState::Fault),
        ("fault_underflow_sub", VmState::Fault),
        // div/mod/left/right/substr reach Halt or Fault depending on generated operands.
    ]
    .into_iter()
    .collect();

    let mut seen_families: HashSet<String> = HashSet::new();
    let mut checked = 0usize;
    let mut fault_cases = 0usize;

    for seed in [1u32, 42, 20260918, u32::MAX] {
        let output = Command::new("dotnet")
            .arg(&dll)
            .args(["--random-ops", &seed.to_string(), "1024"])
            .output()
            .expect("launch C# NeoVM oracle");
        assert!(
            output.status.success(),
            "oracle failed for seed={seed}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let suite: suite::Suite =
            serde_json::from_slice(&output.stdout).expect("oracle must emit valid JSON");
        assert_eq!(suite.neo_vm_package_version, "Neo.VM 3.10.1");
        assert_eq!(suite.vectors.len(), 1024);

        for (index, vector) in suite.vectors.into_iter().enumerate() {
            let suffix = format!("_{seed}_{index}");
            let fam = vector
                .name
                .strip_prefix("ops_")
                .and_then(|rest| rest.strip_suffix(&suffix))
                .unwrap_or_else(|| {
                    panic!("seed={seed} index={index} name {:?} malformed", vector.name)
                });
            assert!(
                known_families.contains(fam),
                "seed={seed} index={index} unknown family {fam}"
            );
            assert_eq!(vector.name, format!("ops_{fam}_{seed}_{index}"));
            seen_families.insert(fam.to_string());
            if fam.starts_with("fault_") {
                assert_eq!(
                    vector.expected_state,
                    VmState::Fault,
                    "seed={seed} index={index} {fam}"
                );
            }
            if let Some(&must) = expected_state.get(fam) {
                if must == VmState::Fault {
                    assert_eq!(
                        vector.expected_state,
                        VmState::Fault,
                        "seed={seed} index={index} {fam}"
                    );
                }
            }

            let hex = &vector.script_hex;
            assert!(hex.is_ascii() && hex.len().is_multiple_of(2));
            let script: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("script hex"))
                .collect();
            assert_eq!(
                script.last(),
                Some(&0x40),
                "seed={seed} index={index} {fam}"
            );

            match interpret(&script) {
                Ok(outcome) => {
                    assert_eq!(
                        outcome.state, vector.expected_state,
                        "seed={seed} index={index} fam={fam} script={hex}"
                    );
                    if outcome.state == VmState::Halt {
                        assert_eq!(
                            outcome.stack, vector.expected_stack,
                            "seed={seed} index={index} fam={fam} script={hex}"
                        );
                    } else {
                        fault_cases += 1;
                    }
                }
                Err(_) => {
                    assert_eq!(
                        vector.expected_state,
                        VmState::Fault,
                        "Rust faulted but oracle expected Halt for seed={seed} index={index} fam={fam} script={hex}"
                    );
                    fault_cases += 1;
                }
            }
            checked += 1;
        }
    }

    assert_eq!(checked, 4096);
    assert_eq!(
        seen_families.len(),
        known_families.len(),
        "missing families"
    );
    assert!(
        fault_cases > 500,
        "expected many FAULT cases, got {fault_cases}"
    );
    println!(
        "C# Neo.VM 3.10.1 / neo-vm-rs: {checked} comparisons ({fault_cases} FAULT), families={}",
        seen_families.len()
    );
}
