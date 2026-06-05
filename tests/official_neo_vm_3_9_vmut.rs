use neo_vm_rs::{
    OpCode, StackValue, SyscallProvider, VmState, encode_integer, interpret_with_stack_and_syscalls,
};
use num_bigint::BigInt;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

const FIXTURE_ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/official-neo-vm-3.9.0/Tests"
);

#[test]
fn official_neo_vm_3_9_vmut_suite_matches_shared_interpreter() {
    let root = Path::new(FIXTURE_ROOT);
    assert!(
        root.exists(),
        "official Neo.VM 3.9.0 VMUT fixture directory is missing: {}",
        root.display()
    );

    let mut files = Vec::new();
    collect_json_files(root, &mut files).expect("fixture traversal should succeed");
    files.sort();

    let mut checked_cases = 0usize;
    let mut skipped_break_cases = 0usize;

    for file in &files {
        let text = fs::read_to_string(file)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", file.display()));
        let suite: Value = serde_json::from_str(&text)
            .unwrap_or_else(|err| panic!("invalid JSON in {}: {err}", file.display()));

        let tests = suite["tests"]
            .as_array()
            .unwrap_or_else(|| panic!("{} missing tests array", file.display()));

        for test in tests {
            let steps = test["steps"]
                .as_array()
                .unwrap_or_else(|| panic!("{} test missing steps array", case_name(file, test)));
            let Some(last_step) = steps.last() else {
                continue;
            };
            let state = last_step["result"]["state"].as_str().unwrap_or("");
            if state == "BREAK" {
                skipped_break_cases += 1;
                continue;
            }
            if state != "HALT" && state != "FAULT" {
                panic!(
                    "{} has unsupported final state {state}",
                    case_name(file, test)
                );
            }

            let script = parse_script(&test["script"]).unwrap_or_else(|err| {
                panic!("{} script parse failed: {err}", case_name(file, test))
            });
            let expected_state = if state == "HALT" {
                VmState::Halt
            } else {
                VmState::Fault
            };

            let mut syscalls = OfficialVmTestSyscalls::default();
            let result = match interpret_with_stack_and_syscalls(&script, Vec::new(), &mut syscalls)
            {
                Ok(result) => result,
                Err(err) if expected_state == VmState::Fault => {
                    let _ = err;
                    checked_cases += 1;
                    continue;
                }
                Err(err) => panic!("{} interpreter error: {err}", case_name(file, test)),
            };

            assert_eq!(result.state, expected_state, "{}", case_name(file, test));

            // Match Neo.VM's VMJsonTestBase: once the engine reaches FAULT the
            // upstream harness validates only state/exception and returns before
            // checking invocation or result stack fields from JSON.
            if expected_state == VmState::Fault {
                checked_cases += 1;
                continue;
            }

            if let Some(expected_stack) = last_step["result"]["resultStack"].as_array() {
                let expected = expected_stack
                    .iter()
                    .map(parse_expected_item)
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap_or_else(|err| {
                        panic!(
                            "{} expected stack parse failed: {err}",
                            case_name(file, test)
                        )
                    });
                let actual = result
                    .stack
                    .iter()
                    .rev()
                    .map(canonical_actual_item)
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap_or_else(|err| {
                        panic!(
                            "{} actual stack normalization failed: {err}",
                            case_name(file, test)
                        )
                    });
                assert_eq!(actual, expected, "{}", case_name(file, test));
            }

            checked_cases += 1;
        }
    }

    assert_eq!(files.len(), 161, "official VMUT fixture file count drifted");
    assert_eq!(
        checked_cases, 723,
        "official executable VMUT case count drifted"
    );
    assert_eq!(
        skipped_break_cases, 16,
        "official debugger-only BREAK case count drifted"
    );
}

#[derive(Default)]
struct OfficialVmTestSyscalls {
    next_interop_handle: u64,
}

impl SyscallProvider for OfficialVmTestSyscalls {
    fn syscall(&mut self, api: u32, _ip: usize, stack: &mut Vec<StackValue>) -> Result<(), String> {
        match api {
            0x7777_7777 => {
                let handle = self.next_interop_handle;
                self.next_interop_handle += 1;
                stack.push(StackValue::Interop(handle));
                Ok(())
            }
            0xadde_adde => Err("error".to_string()),
            other => Err(format!("unsupported syscall 0x{other:08x}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CanonicalItem {
    Null,
    Pointer(i64),
    Boolean(bool),
    Integer(String),
    ByteString(Vec<u8>),
    Buffer(Vec<u8>),
    Interop,
    Array(Vec<CanonicalItem>),
    Struct(Vec<CanonicalItem>),
    Map(BTreeMap<String, CanonicalItem>),
}

fn collect_json_files(root: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_json_files(&path, files)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            files.push(path);
        }
    }
    Ok(())
}

fn parse_script(value: &Value) -> Result<Vec<u8>, String> {
    if let Some(hex) = value.as_str() {
        return decode_hex(hex);
    }

    let entries = value
        .as_array()
        .ok_or_else(|| "script must be a hex string or token array".to_string())?;
    let mut script = Vec::new();
    for entry in entries {
        let token = entry
            .as_str()
            .ok_or_else(|| format!("script token must be a string: {entry}"))?;
        if token.starts_with("0x") {
            script.extend(decode_hex(token)?);
        } else {
            let opcode = opcode_by_name(token)
                .ok_or_else(|| format!("unknown NeoVM opcode token {token}"))?;
            script.push(opcode.byte());
        }
    }
    Ok(script)
}

fn opcode_by_name(name: &str) -> Option<OpCode> {
    OpCode::ALL
        .iter()
        .copied()
        .find(|opcode| opcode.name().eq_ignore_ascii_case(name))
}

fn parse_expected_item(value: &Value) -> Result<CanonicalItem, String> {
    let item_type = value["type"]
        .as_str()
        .ok_or_else(|| format!("stack item missing type: {value}"))?
        .to_ascii_lowercase();

    match item_type.as_str() {
        "null" => Ok(CanonicalItem::Null),
        "pointer" => Ok(CanonicalItem::Pointer(json_i64(&value["value"])?)),
        "boolean" => Ok(CanonicalItem::Boolean(
            value["value"]
                .as_bool()
                .ok_or_else(|| format!("boolean item has non-bool value: {value}"))?,
        )),
        "integer" => Ok(CanonicalItem::Integer(json_decimal_string(
            &value["value"],
        )?)),
        "string" => Ok(CanonicalItem::ByteString(
            value["value"]
                .as_str()
                .ok_or_else(|| format!("string item has non-string value: {value}"))?
                .as_bytes()
                .to_vec(),
        )),
        "bytestring" => Ok(CanonicalItem::ByteString(decode_hex(json_str_or_empty(
            &value["value"],
        )?)?)),
        "buffer" => Ok(CanonicalItem::Buffer(decode_hex(json_str_or_empty(
            &value["value"],
        )?)?)),
        "interop" => Ok(CanonicalItem::Interop),
        "array" => parse_expected_sequence(&value["value"]).map(CanonicalItem::Array),
        "struct" => parse_expected_sequence(&value["value"]).map(CanonicalItem::Struct),
        "map" => parse_expected_map(&value["value"]).map(CanonicalItem::Map),
        other => Err(format!("unsupported stack item type {other}: {value}")),
    }
}

fn parse_expected_sequence(value: &Value) -> Result<Vec<CanonicalItem>, String> {
    value
        .as_array()
        .ok_or_else(|| format!("compound item value is not an array: {value}"))?
        .iter()
        .map(parse_expected_item)
        .collect()
}

fn parse_expected_map(value: &Value) -> Result<BTreeMap<String, CanonicalItem>, String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("map item value is not an object: {value}"))?;
    object
        .iter()
        .map(|(key, item)| Ok((normalize_hex_key(key), parse_expected_item(item)?)))
        .collect()
}

fn canonical_actual_item(value: &StackValue) -> Result<CanonicalItem, String> {
    match value {
        StackValue::Null => Ok(CanonicalItem::Null),
        StackValue::Pointer(position) => Ok(CanonicalItem::Pointer(*position)),
        StackValue::Boolean(value) => Ok(CanonicalItem::Boolean(*value)),
        StackValue::Integer(value) => Ok(CanonicalItem::Integer(value.to_string())),
        StackValue::BigInteger(bytes) => Ok(CanonicalItem::Integer(
            BigInt::from_signed_bytes_le(bytes).to_string(),
        )),
        StackValue::ByteString(bytes) => Ok(CanonicalItem::ByteString(bytes.clone())),
        StackValue::Buffer(bytes) => Ok(CanonicalItem::Buffer(bytes.clone())),
        StackValue::Interop(_) | StackValue::Iterator(_) => Ok(CanonicalItem::Interop),
        StackValue::Array(items) => items
            .iter()
            .map(canonical_actual_item)
            .collect::<Result<Vec<_>, _>>()
            .map(CanonicalItem::Array),
        StackValue::Struct(items) => items
            .iter()
            .map(canonical_actual_item)
            .collect::<Result<Vec<_>, _>>()
            .map(CanonicalItem::Struct),
        StackValue::Map(entries) => entries
            .iter()
            .map(|(key, item)| {
                Ok((
                    bytes_to_hex(&stack_item_span(key)?),
                    canonical_actual_item(item)?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()
            .map(CanonicalItem::Map),
    }
}

fn stack_item_span(value: &StackValue) -> Result<Vec<u8>, String> {
    match value {
        StackValue::Integer(value) => Ok(encode_integer(*value)),
        StackValue::BigInteger(bytes)
        | StackValue::ByteString(bytes)
        | StackValue::Buffer(bytes) => Ok(bytes.clone()),
        StackValue::Boolean(value) => Ok(vec![u8::from(*value)]),
        StackValue::Null
        | StackValue::Array(_)
        | StackValue::Struct(_)
        | StackValue::Map(_)
        | StackValue::Interop(_)
        | StackValue::Iterator(_)
        | StackValue::Pointer(_) => Err(format!("stack item has no NeoVM span: {value:?}")),
    }
}

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    let hex = value.strip_prefix("0x").unwrap_or(value);
    if hex.is_empty() {
        return Ok(Vec::new());
    }
    if !hex.len().is_multiple_of(2) {
        return Err(format!("hex string has odd length: {value}"));
    }
    (0..hex.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&hex[index..index + 2], 16)
                .map_err(|err| format!("invalid hex byte in {value} at {index}: {err}"))
        })
        .collect()
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    let mut hex = String::from("0x");
    for byte in bytes {
        use std::fmt::Write;
        write!(&mut hex, "{byte:02x}").expect("writing to String cannot fail");
    }
    hex
}

fn normalize_hex_key(key: &str) -> String {
    if key.is_empty() {
        String::new()
    } else if key.starts_with("0x") {
        key.to_ascii_lowercase()
    } else {
        format!("0x{}", key.to_ascii_lowercase())
    }
}

fn json_decimal_string(value: &Value) -> Result<String, String> {
    match value {
        Value::Number(number) => Ok(number.to_string()),
        Value::String(string) => Ok(string.clone()),
        _ => Err(format!(
            "expected integer number or decimal string: {value}"
        )),
    }
}

fn json_i64(value: &Value) -> Result<i64, String> {
    match value {
        Value::Number(number) => number
            .as_i64()
            .ok_or_else(|| format!("expected i64-compatible number: {value}")),
        Value::String(string) => string
            .parse()
            .map_err(|err| format!("invalid i64 string {string}: {err}")),
        _ => Err(format!("expected i64 number or string: {value}")),
    }
}

fn json_str_or_empty(value: &Value) -> Result<&str, String> {
    if value.is_null() {
        return Ok("");
    }
    value
        .as_str()
        .ok_or_else(|| format!("expected string value: {value}"))
}

fn case_name(file: &Path, test: &Value) -> String {
    let name = test["name"].as_str().unwrap_or("<unnamed>");
    format!("{}::{name}", file.display())
}
