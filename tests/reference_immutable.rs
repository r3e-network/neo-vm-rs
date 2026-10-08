use neo_vm_rs::reference::{SharedHeap, SharedValue};
use neo_vm_rs::{
    OpCode, ReferenceSyscallProvider, SharedVmException, StackValue,
    interpret_with_stack_and_syscalls_at_reference as run,
};

fn integer(n: i64) -> SharedValue {
    SharedValue::primitive(StackValue::Integer(n)).unwrap()
}
struct Host;
impl ReferenceSyscallProvider for Host {
    fn syscall(
        &mut self,
        _: u32,
        _: usize,
        _: usize,
        _: &mut Vec<SharedValue>,
    ) -> Result<(), SharedVmException> {
        panic!("no syscall expected")
    }
}

#[test]
fn immutable_graph_preserves_aliases_cycles_and_separates_source() {
    let heap = SharedHeap::new();
    let child = heap.array(vec![integer(7)]).unwrap();
    let source = heap.array(vec![child.clone(), child.clone()]).unwrap();
    source.append_raw(&source).unwrap();
    let copied = heap.deep_copy(&source, true).unwrap();
    assert!(!copied.same_reference(&source));
    let items = copied.items().unwrap();
    assert!(items[0].same_reference(&items[1]));
    assert!(items[2].same_reference(&copied));
    assert!(copied.is_read_only() && items[0].is_read_only());
    child.set_item_raw(&integer(0), &integer(9)).unwrap();
    assert_eq!(items[0].item(&integer(0)).unwrap(), Some(integer(7)));
    assert!(copied.clear().is_err());
    assert!(items[0].append_raw(&integer(1)).is_err());
}

#[test]
fn immutable_copy_keeps_map_alias_and_freezes_values_but_not_original() {
    let heap = SharedHeap::new();
    let source = heap.map(vec![]).unwrap();
    source.set_item_raw(&integer(1), &source).unwrap();
    let copied = heap.deep_copy(&source, true).unwrap();
    assert!(
        copied
            .item(&integer(1))
            .unwrap()
            .unwrap()
            .same_reference(&copied)
    );
    assert!(copied.set_item_raw(&integer(2), &integer(3)).is_err());
    assert!(copied.remove_item(&integer(1)).is_err());
    assert!(copied.clear().is_err());
    source.set_item_raw(&integer(2), &integer(3)).unwrap();
    assert_eq!(copied.len().unwrap(), 1);
}

#[test]
fn buffers_copy_or_become_bytes_without_aliasing_original() {
    let heap = SharedHeap::new();
    let source = heap.buffer(vec![1, 2]).unwrap();
    let mutable = heap.deep_copy(&source, false).unwrap();
    let immutable = heap.deep_copy(&source, true).unwrap();
    source.write_buffer(0, &[9]).unwrap();
    assert_eq!(mutable.buffer_bytes().unwrap(), vec![1, 2]);
    assert_eq!(
        immutable.primitive_ref(),
        Some(&StackValue::ByteString(vec![1, 2]))
    );
}

#[test]
fn readonly_survives_deep_copy_false_but_struct_clone_is_mutable() {
    let heap = SharedHeap::new();
    let array = heap.array(vec![integer(1)]).unwrap();
    let nested = heap.structure(vec![integer(2)]).unwrap();
    let source = heap.structure(vec![array, nested]).unwrap();
    let frozen = heap.deep_copy(&source, true).unwrap();
    assert!(heap.deep_copy(&frozen, false).unwrap().is_read_only());
    let cloned = frozen.clone_struct(2048).unwrap();
    assert!(!cloned.is_read_only());
    let cloned_items = cloned.items().unwrap();
    assert!(cloned_items[0].is_read_only());
    assert!(!cloned_items[1].is_read_only());
    cloned_items[1].append_raw(&integer(3)).unwrap();
    cloned.append_raw(&integer(4)).unwrap();
}

#[test]
fn reference_opcodes_observe_immutable_guards() {
    let heap = SharedHeap::new();
    let mutable = heap.array(vec![integer(7)]).unwrap();
    let frozen = heap.deep_copy(&mutable, true).unwrap();
    for (opcode, extra) in [
        (OpCode::SETITEM, vec![integer(0), integer(3)]),
        (OpCode::APPEND, vec![integer(3)]),
        (OpCode::REMOVE, vec![integer(0)]),
        (OpCode::CLEARITEMS, vec![]),
        (OpCode::REVERSEITEMS, vec![]),
        (OpCode::POPITEM, vec![]),
    ] {
        let mut initial = vec![frozen.clone()];
        initial.extend(extra);
        let result = run(&[opcode as u8], initial, 0, heap.clone(), 1, &mut Host);
        assert!(
            matches!(result, Err(SharedVmException::Fault(_))),
            "{opcode:?}"
        );
    }
    let result = run(
        &[OpCode::PUSH0 as u8, OpCode::PICKITEM as u8],
        vec![frozen],
        0,
        heap,
        1,
        &mut Host,
    )
    .unwrap();
    assert_eq!(result.stack, vec![integer(7)]);
}

#[test]
fn copied_cycles_are_collectable_and_cross_heap_inputs_are_rejected() {
    let heap = SharedHeap::new();
    {
        let source = heap.array(vec![]).unwrap();
        source.append_raw(&source).unwrap();
        let copied = heap.deep_copy(&source, true).unwrap();
        assert!(SharedHeap::new().deep_copy(&copied, true).is_err());
    }
    heap.collect();
    assert_eq!(heap.reference_count(), 0);
}
