use criterion::{criterion_group, criterion_main, Criterion};
use neo_vm_rs::{interop_hash, syscall_arg_count, OpCode, StackValue};
use std::hint::black_box;

fn opcode_decode(c: &mut Criterion) {
    let bytes: Vec<u8> = OpCode::ALL.iter().map(|opcode| opcode.byte()).collect();
    c.bench_function("opcode_decode_all", |b| {
        b.iter(|| {
            for byte in &bytes {
                black_box(OpCode::try_from(black_box(*byte)).expect("valid opcode"));
            }
        })
    });
}

fn opcode_metadata(c: &mut Criterion) {
    c.bench_function("opcode_metadata_all", |b| {
        b.iter(|| {
            for opcode in OpCode::ALL {
                black_box(opcode.name());
                black_box(opcode.operand_size());
                black_box(opcode.operand_prefix());
            }
        })
    });
}

fn syscall_helpers(c: &mut Criterion) {
    let names = [
        "System.Contract.Call",
        "System.Contract.CallNative",
        "System.Runtime.Platform",
        "System.Runtime.CheckWitness",
        "System.Storage.Get",
        "System.Storage.Put",
        "System.Crypto.CheckSig",
        "System.Iterator.Next",
    ];

    c.bench_function("syscall_hash_and_arg_count", |b| {
        b.iter(|| {
            for name in names {
                let hash = interop_hash(black_box(name));
                black_box(syscall_arg_count(hash));
            }
        })
    });
}

fn stack_value_conversions(c: &mut Criterion) {
    let values = [
        StackValue::ByteString(vec![0xff]),
        StackValue::ByteString(vec![0x00, 0x80]),
        StackValue::ByteString(vec![0x01, 0x00, 0x00, 0x00]),
        StackValue::Boolean(true),
        StackValue::Integer(42),
    ];

    c.bench_function("stack_value_bool_and_int", |b| {
        b.iter(|| {
            for value in &values {
                black_box(value.to_bool());
                black_box(value.to_i128());
            }
        })
    });
}

criterion_group!(
    benches,
    opcode_decode,
    opcode_metadata,
    syscall_helpers,
    stack_value_conversions
);
criterion_main!(benches);
