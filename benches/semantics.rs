use criterion::{Criterion, criterion_group, criterion_main};
use neo_vm_rs::{
    OpCode, StackValue, VmState, fast_codec, interop_hash, interpret, syscall_arg_count,
};
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

fn stack_codec(c: &mut Criterion) {
    let stack = vec![
        StackValue::Integer(42),
        StackValue::ByteString(vec![0xAA; 32]),
        StackValue::Boolean(true),
        StackValue::Array(vec![
            StackValue::Integer(1),
            StackValue::Integer(2),
            StackValue::ByteString(vec![0xBB; 16]),
        ]),
        StackValue::Map(vec![(
            StackValue::ByteString(b"key".to_vec()),
            StackValue::ByteString(vec![0xCC; 24]),
        )]),
    ];
    let encoded = fast_codec::encode_stack(&stack);

    c.bench_function("fast_codec_encode_stack_mixed", |b| {
        b.iter(|| {
            black_box(fast_codec::encode_stack(black_box(&stack)));
        })
    });

    c.bench_function("fast_codec_decode_stack_mixed", |b| {
        b.iter(|| {
            black_box(fast_codec::decode_stack(black_box(&encoded)).expect("valid stack"));
        })
    });
}

fn interpreter_hot_paths(c: &mut Criterion) {
    let arithmetic_script = [
        0x57, 0x01, 0x00, // INITSLOT locals=1, args=0
        0x10, // PUSH0
        0x70, // STLOC0
        0x68, // LDLOC0
        0x11, // PUSH1
        0x9e, // ADD
        0x70, // STLOC0
        0x68, // LDLOC0
        0x11, // PUSH1
        0x9e, // ADD
        0x70, // STLOC0
        0x68, // LDLOC0
        0x11, // PUSH1
        0x9e, // ADD
        0x70, // STLOC0
        0x68, // LDLOC0
        0x40, // RET
    ];

    c.bench_function("interpret_arithmetic_slots", |b| {
        b.iter(|| {
            let result = interpret(black_box(&arithmetic_script)).expect("script should execute");
            black_box(result.state == VmState::Halt);
        })
    });
}

criterion_group!(
    benches,
    opcode_decode,
    opcode_metadata,
    syscall_helpers,
    stack_value_conversions,
    stack_codec,
    interpreter_hot_paths
);
criterion_main!(benches);
