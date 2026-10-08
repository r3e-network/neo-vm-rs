use neo_vm_rs::reference::{SharedHeap, SharedValue};
use neo_vm_rs::{
    OpCode, ReferenceSyscallProvider, SharedVmException, StackItemType, StackValue,
    interpret_with_stack_and_syscalls_at_reference as run,
};

fn int(n: i64) -> SharedValue {
    SharedValue::primitive(StackValue::Integer(n)).unwrap()
}
#[derive(Default)]
struct NoHost;
impl ReferenceSyscallProvider for NoHost {
    fn syscall(
        &mut self,
        _: u32,
        _: usize,
        _: usize,
        _: &mut Vec<SharedValue>,
    ) -> Result<(), SharedVmException> {
        Err(SharedVmException::Fault("unexpected host call".into()))
    }
}
fn script(code: &[OpCode], args: Vec<SharedValue>, heap: &SharedHeap) -> Vec<SharedValue> {
    run(
        &code.iter().map(|x| *x as u8).collect::<Vec<_>>(),
        args,
        0,
        heap.clone(),
        1,
        &mut NoHost,
    )
    .unwrap()
    .stack
}

#[test]
fn aliases_cycles_and_gc_are_flat_and_preserve_mutations() {
    let heap = SharedHeap::new();
    let a = heap.array(vec![]).unwrap();
    a.append_raw(&a).unwrap();
    assert!(a.item(&int(0)).unwrap().unwrap().same_reference(&a));
    assert!(a.snapshot(2048).is_err());
    assert_eq!(heap.collect(), 0);
    assert_eq!(heap.subitem_count(), 1);
    drop(a);
    assert_eq!(heap.collect(), 1);
    assert_eq!(heap.object_count(), 0);
    assert_eq!(heap.subitem_count(), 0);
}
#[test]
fn clone_struct_only_copies_struct_children_and_bounds_cycles() {
    let heap = SharedHeap::new();
    let a = heap.array(vec![int(1)]).unwrap();
    let b = heap.buffer(vec![1]).unwrap();
    let nested = heap.structure(vec![a.clone(), b.clone()]).unwrap();
    let outer = heap.structure(vec![nested.clone()]).unwrap();
    let copied = outer.clone_struct(2048).unwrap();
    let child = copied.item(&int(0)).unwrap().unwrap();
    assert!(!child.same_reference(&nested));
    assert!(child.item(&int(0)).unwrap().unwrap().same_reference(&a));
    assert!(child.item(&int(1)).unwrap().unwrap().same_reference(&b));
    let cycle = heap.structure(vec![]).unwrap();
    cycle.append_raw(&cycle).unwrap();
    assert_eq!(
        cycle.clone_struct(2048).unwrap_err(),
        "Beyond struct subitem clone limits!"
    );
}
#[test]
fn map_keys_are_typed_ordered_and_integer_width_independent() {
    let heap = SharedHeap::new();
    let yes = SharedValue::primitive(StackValue::Boolean(true)).unwrap();
    let bytes = SharedValue::primitive(StackValue::ByteString(vec![1])).unwrap();
    let map = heap
        .map(vec![
            (yes.clone(), int(1)),
            (int(1), int(2)),
            (bytes.clone(), int(3)),
        ])
        .unwrap();
    map.set_item_raw(
        &SharedValue::primitive(StackValue::BigInteger(vec![1, 0])).unwrap(),
        &int(9),
    )
    .unwrap();
    assert_eq!(map.len().unwrap(), 3);
    assert_eq!(
        map.item(&int(1)).unwrap().unwrap().primitive_ref(),
        Some(&StackValue::Integer(9))
    );
    assert_eq!(map.map_items().unwrap()[0].0, yes);
    assert_eq!(map.map_items().unwrap()[2].0, bytes);
    assert!(
        map.set_item_raw(&heap.buffer(vec![1]).unwrap(), &int(0))
            .is_err()
    );
    assert!(
        map.set_item_raw(
            &SharedValue::primitive(StackValue::ByteString(vec![0; 65])).unwrap(),
            &int(0)
        )
        .is_err()
    );
}
#[test]
fn foreign_heap_edges_are_rejected_without_mutation() {
    let heap = SharedHeap::new();
    let a = heap.array(vec![]).unwrap();
    let b = SharedHeap::new().array(vec![]).unwrap();
    assert!(a.append_raw(&b).is_err());
    assert_eq!(a.len().unwrap(), 0);
    assert!(heap.array(vec![b]).is_err());
}
#[test]
fn snapshots_preserve_types_alias_values_and_bounded_depth() {
    let heap = SharedHeap::new();
    let b = heap.buffer(vec![7]).unwrap();
    let a = heap.structure(vec![b.clone(), b]).unwrap();
    assert_eq!(
        a.snapshot(3).unwrap(),
        StackValue::Struct(vec![
            StackValue::Buffer(vec![7]),
            StackValue::Buffer(vec![7])
        ])
    );
    assert!(a.snapshot(2).is_err());
    let mut deep = int(0);
    for _ in 0..1500 {
        deep = heap.array(vec![deep]).unwrap();
    }
    let snapshot = deep.snapshot(1501).unwrap();
    assert!(matches!(snapshot, StackValue::Array(_)));
    assert!(deep.snapshot(1500).is_err());
}
#[test]
fn cached_reference_counts_follow_mutation_collection_and_frame_drop() {
    let heap = SharedHeap::new();
    let mut parent = heap.register_frame();
    parent.set(3);
    let a = heap.array(vec![int(1)]).unwrap();
    assert_eq!(heap.reference_count(), 4);
    a.append_raw(&int(2)).unwrap();
    assert_eq!(heap.reference_count(), 5);
    {
        let mut child = heap.register_frame();
        child.set(4);
        assert_eq!(heap.reference_count(), 9);
        child.set(1);
        assert_eq!(heap.reference_count(), 6);
    }
    a.clear().unwrap();
    assert_eq!(heap.reference_count(), 3);
    drop(parent);
    assert_eq!(heap.reference_count(), 0);
    drop(a);
    heap.collect();
    assert_eq!(heap.reference_count(), 0);
}
#[test]
fn native_struct_comparison_uses_reference_children_and_comparison_budgets() {
    let heap = SharedHeap::new();
    let a = heap.array(vec![int(1)]).unwrap();
    let b = heap.array(vec![int(1)]).unwrap();
    let x = heap.structure(vec![a.clone()]).unwrap();
    let y = heap.structure(vec![a]).unwrap();
    let z = heap.structure(vec![b]).unwrap();
    assert!(x.vm_equal(&y, 2048, 65535).unwrap());
    assert!(!x.vm_equal(&z, 2048, 65535).unwrap());
    assert!(x.vm_equal(&y, 1, 65535).is_err());
    let empty = SharedValue::primitive(StackValue::ByteString(vec![])).unwrap();
    assert!(
        heap.structure(vec![empty.clone(), empty])
            .unwrap()
            .vm_equal(
                &heap
                    .structure(vec![
                        SharedValue::primitive(StackValue::ByteString(vec![]))
                            .unwrap();
                        2
                    ])
                    .unwrap(),
                2048,
                2
            )
            .is_err()
    );
}
#[test]
fn buffer_setitem_and_memcpy_mutate_shared_target() {
    let heap = SharedHeap::new();
    let b = heap.buffer(vec![0; 3]).unwrap();
    let alias = b.clone();
    script(
        &[OpCode::PUSH0, OpCode::PUSH7, OpCode::SETITEM],
        vec![b.clone()],
        &heap,
    );
    assert_eq!(alias.buffer_bytes().unwrap(), vec![7, 0, 0]);
    script(
        &[OpCode::MEMCPY],
        vec![b.clone(), int(1), b.clone(), int(0), int(2)],
        &heap,
    );
    assert_eq!(alias.buffer_bytes().unwrap(), vec![7, 7, 0]);
}
#[test]
fn convert_preserves_identity_or_creates_only_a_new_outer_shell() {
    let heap = SharedHeap::new();
    let child = heap.array(vec![int(1)]).unwrap();
    let value = heap.structure(vec![child.clone()]).unwrap();
    let result = run(
        &[OpCode::CONVERT as u8, StackItemType::Array as u8],
        vec![value.clone()],
        0,
        heap.clone(),
        1,
        &mut NoHost,
    )
    .unwrap()
    .stack
    .remove(0);
    assert!(!result.same_reference(&value));
    assert!(
        result
            .item(&int(0))
            .unwrap()
            .unwrap()
            .same_reference(&child)
    );
    let buf = heap.buffer(vec![1]).unwrap();
    let out = run(
        &[OpCode::CONVERT as u8, StackItemType::Buffer as u8],
        vec![buf.clone()],
        0,
        heap.clone(),
        1,
        &mut NoHost,
    )
    .unwrap();
    assert!(out.stack[0].same_reference(&buf));
}
#[test]
fn newarray_boolean_default_is_native_false_and_bad_tag_faults() {
    let heap = SharedHeap::new();
    let out = run(
        &[
            OpCode::PUSH1 as u8,
            OpCode::NEWARRAY_T as u8,
            StackItemType::Boolean as u8,
        ],
        vec![],
        0,
        heap.clone(),
        1,
        &mut NoHost,
    )
    .unwrap();
    assert_eq!(
        out.stack[0].item(&int(0)).unwrap().unwrap().primitive_ref(),
        Some(&StackValue::Boolean(false))
    );
    assert!(
        run(
            &[OpCode::PUSH1 as u8, OpCode::NEWARRAY_T as u8, 0xff],
            vec![],
            0,
            heap,
            1,
            &mut NoHost
        )
        .is_err()
    );
}

#[test]
fn implicit_integer_arguments_reject_buffer_without_changing_explicit_conversion() {
    let heap = SharedHeap::new();
    for opcode in [
        OpCode::NEWARRAY,
        OpCode::NEWSTRUCT,
        OpCode::PACK,
        OpCode::PACKSTRUCT,
        OpCode::PACKMAP,
        OpCode::NEWBUFFER,
    ] {
        let buffer = heap.buffer(vec![1]).unwrap();
        assert!(
            run(
                &[opcode as u8],
                vec![int(7), buffer],
                0,
                heap.clone(),
                1,
                &mut NoHost
            )
            .is_err(),
            "{opcode:?}"
        );
    }
    let buffer = heap.buffer(vec![1]).unwrap();
    let result = run(
        &[OpCode::CONVERT as u8, StackItemType::Integer as u8],
        vec![buffer],
        0,
        heap,
        1,
        &mut NoHost,
    )
    .unwrap();
    assert_eq!(
        result.stack[0].primitive_ref(),
        Some(&StackValue::Integer(1))
    );
}

#[test]
fn pointer_same_type_conversion_preserves_the_loaded_script_identity() {
    let result = run(
        &[
            OpCode::PUSHA as u8,
            9,
            0,
            0,
            0,
            OpCode::DUP as u8,
            OpCode::CONVERT as u8,
            StackItemType::Pointer as u8,
            OpCode::EQUAL as u8,
            OpCode::RET as u8,
        ],
        vec![],
        0,
        SharedHeap::new(),
        1,
        &mut NoHost,
    )
    .unwrap();
    assert_eq!(
        result.stack[0].primitive_ref(),
        Some(&StackValue::Boolean(true))
    );
}
