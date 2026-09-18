use std::{collections::HashSet, path::Path, process::Command};

use neo_vm_rs::{VmState, interpret};

#[path = "neo_vm_3_9_conformance/suite.rs"]
mod suite;
#[path = "neo_vm_3_9_conformance/vector.rs"]
mod vector;

#[test]
#[ignore = "requires the built .NET 10 NeoVm39ConformanceGenerator; see tools/NeoVm39ConformanceGenerator/DIFFERENTIAL.md"]
fn random_add_scripts_match_live_csharp_neovm() {
    let dll = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "tools/NeoVm39ConformanceGenerator/bin/Debug/net10.0/NeoVm39ConformanceGenerator.dll",
    );
    assert!(
        dll.is_file(),
        "build the C# oracle first: {}",
        dll.display()
    );
    let mut scripts = HashSet::new();
    let mut checked = 0;
    for seed in [1u32, 42, 20260918, u32::MAX] {
        let output = Command::new("dotnet")
            .arg(&dll)
            .args(["--random-add", &seed.to_string(), "1024"])
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
        assert_eq!(suite.neo_package_version, "Neo 3.10.1");
        assert_eq!(suite.neo_node_tag, "v3.10.1");
        assert!(suite.source.contains("Neo.VM 3.10.1"));
        assert_eq!(suite.vectors.len(), 1024);
        for (index, vector) in suite.vectors.into_iter().enumerate() {
            assert_eq!(vector.name, format!("random_add_{seed}_{index}"));
            assert!(!vector.description.is_empty());
            let hex = &vector.script_hex;
            assert!(hex.is_ascii() && hex.len().is_multiple_of(2));
            let script: Vec<u8> = (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("script hex"))
                .collect();
            assert!((6..=96).contains(&script.len()));
            assert_eq!(&script[..1], &[0x00]);
            assert_eq!(script.last(), Some(&0x40));
            let body = &script[..script.len() - 1];
            assert_eq!((body.len() + 1) % 3, 0);
            for term in body[2..].chunks_exact(3) {
                assert_eq!(term[0], 0x00);
                assert_eq!(term[2], 0x9e);
            }
            assert_eq!(vector.expected_state, VmState::Halt);
            let result = interpret(&script)
                .unwrap_or_else(|error| panic!("seed={seed} index={index} script={hex}: {error}"));
            assert_eq!(
                result.state, vector.expected_state,
                "seed={seed} index={index} script={hex}"
            );
            assert_eq!(
                result.stack, vector.expected_stack,
                "seed={seed} index={index} script={hex}"
            );
            scripts.insert(script);
            checked += 1;
        }
    }
    assert_eq!(checked, 4096);
    assert!(scripts.len() > 4000, "random corpus lost diversity");
    println!(
        "C# Neo.VM 3.10.1 / neo-vm-rs: {checked} comparisons, {} unique scripts",
        scripts.len()
    );
}
