use crate::StackValue as AbiStackValue;
use alloc::vec::Vec;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum StackValue {
    Integer(i64),
    BigInteger(Vec<u8>),
    ByteString(Vec<u8>),
    Boolean(bool),
    Pointer(usize),
    Array(u64, Vec<StackValue>),
    Struct(u64, Vec<StackValue>),
    Map(u64, Vec<(StackValue, StackValue)>),
    Buffer(u64, Vec<u8>),
    Interop(u64),
    Iterator(u64),
    Null,
}

impl crate::runtime::PendingExceptionValue for StackValue {
    fn from_exception_message(message: alloc::string::String) -> Self {
        Self::ByteString(message.into_bytes())
    }
}

#[derive(Default)]
pub(crate) struct CompoundIds {
    next: u64,
}

impl CompoundIds {
    fn alloc(&mut self) -> u64 {
        let id = self.next;
        self.next += 1;
        id
    }

    pub(crate) fn array(&mut self, items: Vec<StackValue>) -> StackValue {
        StackValue::Array(self.alloc(), items)
    }

    pub(crate) fn r#struct(&mut self, items: Vec<StackValue>) -> StackValue {
        StackValue::Struct(self.alloc(), items)
    }

    pub(crate) fn map(&mut self, items: Vec<(StackValue, StackValue)>) -> StackValue {
        StackValue::Map(self.alloc(), items)
    }

    pub(crate) fn buffer(&mut self, bytes: Vec<u8>) -> StackValue {
        StackValue::Buffer(self.alloc(), bytes)
    }

    pub(crate) fn clone_struct_for_storage(&mut self, value: &StackValue) -> StackValue {
        match value {
            StackValue::Struct(_, _) => self.deep_clone(value),
            _ => value.clone(),
        }
    }

    pub(crate) fn deep_clone(&mut self, value: &StackValue) -> StackValue {
        match value {
            StackValue::Integer(value) => StackValue::Integer(*value),
            StackValue::BigInteger(value) => StackValue::BigInteger(value.clone()),
            StackValue::ByteString(value) => StackValue::ByteString(value.clone()),
            StackValue::Boolean(value) => StackValue::Boolean(*value),
            StackValue::Pointer(value) => StackValue::Pointer(*value),
            StackValue::Array(_, items) => {
                let mut cloned = Vec::with_capacity(items.len());
                for item in items {
                    cloned.push(self.deep_clone(item));
                }
                self.array(cloned)
            }
            StackValue::Struct(_, items) => {
                let mut cloned = Vec::with_capacity(items.len());
                for item in items {
                    cloned.push(self.deep_clone(item));
                }
                self.r#struct(cloned)
            }
            StackValue::Map(_, items) => {
                let mut cloned = Vec::with_capacity(items.len());
                for (key, value) in items {
                    cloned.push((self.deep_clone(key), self.deep_clone(value)));
                }
                self.map(cloned)
            }
            StackValue::Buffer(_, bytes) => self.buffer(bytes.clone()),
            StackValue::Interop(handle) => StackValue::Interop(*handle),
            StackValue::Iterator(handle) => StackValue::Iterator(*handle),
            StackValue::Null => StackValue::Null,
        }
    }

    pub(crate) fn import_abi(&mut self, value: AbiStackValue) -> StackValue {
        match value {
            AbiStackValue::Integer(value) => StackValue::Integer(value),
            AbiStackValue::BigInteger(value) => StackValue::BigInteger(value),
            AbiStackValue::ByteString(value) => StackValue::ByteString(value),
            AbiStackValue::Boolean(value) => StackValue::Boolean(value),
            AbiStackValue::Pointer(value) => StackValue::Pointer(value as usize),
            AbiStackValue::Array(items) => {
                let len = items.len();
                let mut imported = Vec::with_capacity(len);
                for item in items {
                    imported.push(self.import_abi(item));
                }
                self.array(imported)
            }
            AbiStackValue::Struct(items) => {
                let len = items.len();
                let mut imported = Vec::with_capacity(len);
                for item in items {
                    imported.push(self.import_abi(item));
                }
                self.r#struct(imported)
            }
            AbiStackValue::Map(items) => {
                let len = items.len();
                let mut imported = Vec::with_capacity(len);
                for (key, value) in items {
                    imported.push((self.import_abi(key), self.import_abi(value)));
                }
                self.map(imported)
            }
            AbiStackValue::Buffer(value) => self.buffer(value),
            AbiStackValue::Interop(handle) => StackValue::Interop(handle),
            AbiStackValue::Iterator(handle) => StackValue::Iterator(handle),
            AbiStackValue::Null => StackValue::Null,
        }
    }
}

#[inline]
pub(crate) fn to_abi_stack(stack: &[StackValue]) -> Vec<AbiStackValue> {
    let mut abi = Vec::with_capacity(stack.len().max(8));
    for item in stack {
        abi.push(to_abi_value(item));
    }
    abi
}

#[inline]
pub(crate) fn to_abi_value(value: &StackValue) -> AbiStackValue {
    match value {
        StackValue::Integer(value) => AbiStackValue::Integer(*value),
        StackValue::BigInteger(value) => AbiStackValue::BigInteger(clone_bytes(value)),
        StackValue::ByteString(value) => AbiStackValue::ByteString(clone_bytes(value)),
        StackValue::Boolean(value) => AbiStackValue::Boolean(*value),
        StackValue::Pointer(value) => AbiStackValue::Pointer(*value as i64),
        StackValue::Array(_, items) => {
            let mut converted = Vec::with_capacity(items.len());
            for item in items {
                converted.push(to_abi_value(item));
            }
            AbiStackValue::Array(converted)
        }
        StackValue::Struct(_, items) => {
            let mut converted = Vec::with_capacity(items.len());
            for item in items {
                converted.push(to_abi_value(item));
            }
            AbiStackValue::Struct(converted)
        }
        StackValue::Map(_, items) => {
            let mut converted = Vec::with_capacity(items.len());
            for (key, value) in items {
                converted.push((to_abi_value(key), to_abi_value(value)));
            }
            AbiStackValue::Map(converted)
        }
        StackValue::Buffer(_, bytes) => AbiStackValue::Buffer(clone_bytes(bytes)),
        StackValue::Interop(handle) => AbiStackValue::Interop(*handle),
        StackValue::Iterator(handle) => AbiStackValue::Iterator(*handle),
        StackValue::Null => AbiStackValue::Null,
    }
}

#[inline]
pub(crate) fn into_abi_value(value: StackValue) -> AbiStackValue {
    match value {
        StackValue::Integer(value) => AbiStackValue::Integer(value),
        StackValue::BigInteger(value) => AbiStackValue::BigInteger(value),
        StackValue::ByteString(value) => AbiStackValue::ByteString(value),
        StackValue::Boolean(value) => AbiStackValue::Boolean(value),
        StackValue::Pointer(value) => AbiStackValue::Pointer(value as i64),
        StackValue::Array(_, items) => {
            AbiStackValue::Array(items.into_iter().map(into_abi_value).collect())
        }
        StackValue::Struct(_, items) => {
            AbiStackValue::Struct(items.into_iter().map(into_abi_value).collect())
        }
        StackValue::Map(_, items) => AbiStackValue::Map(
            items
                .into_iter()
                .map(|(key, value)| (into_abi_value(key), into_abi_value(value)))
                .collect(),
        ),
        StackValue::Buffer(_, bytes) => AbiStackValue::Buffer(bytes),
        StackValue::Interop(handle) => AbiStackValue::Interop(handle),
        StackValue::Iterator(handle) => AbiStackValue::Iterator(handle),
        StackValue::Null => AbiStackValue::Null,
    }
}

pub(crate) fn structurally_equal(left: &StackValue, right: &StackValue) -> bool {
    match (left, right) {
        (StackValue::Integer(l), StackValue::Integer(r)) => l == r,
        (StackValue::BigInteger(l), StackValue::BigInteger(r)) => l == r,
        (StackValue::ByteString(l), StackValue::ByteString(r)) => l == r,
        (StackValue::Boolean(l), StackValue::Boolean(r)) => l == r,
        (StackValue::Pointer(l), StackValue::Pointer(r)) => l == r,
        (StackValue::Null, StackValue::Null) => true,
        (StackValue::Interop(l), StackValue::Interop(r)) => l == r,
        (StackValue::Iterator(l), StackValue::Iterator(r)) => l == r,
        (StackValue::Buffer(_, l), StackValue::Buffer(_, r)) => l == r,
        (StackValue::Array(_, l), StackValue::Array(_, r))
        | (StackValue::Struct(_, l), StackValue::Struct(_, r)) => {
            l.len() == r.len()
                && l.iter()
                    .zip(r.iter())
                    .all(|(l, r)| structurally_equal(l, r))
        }
        (StackValue::Map(_, l), StackValue::Map(_, r)) => {
            l.len() == r.len()
                && l.iter().zip(r.iter()).all(|((lk, lv), (rk, rv))| {
                    structurally_equal(lk, rk) && structurally_equal(lv, rv)
                })
        }
        _ => false,
    }
}

/// Computes the C# Neo.VM `ReferenceCounter` count for the current VM roots:
/// one reference per eval-stack entry and per slot (locals / arguments / static
/// fields), plus one per compound-to-child edge (Array/Struct items; Map keys
/// AND values, i.e. `Count * 2`). This mirrors `_referencesCount`
/// (`AddStackReference` + `AddReference`) in
/// `neo_csharp_vm/src/Neo.VM/ReferenceCounter.cs`. Each compound's child edges
/// are counted once (dedup by id), which both matches C# (a compound's children
/// are linked once when built) and makes the walk terminate on shared/cyclic
/// compounds. The interpreter faults when this exceeds `MaxStackSize` (2048),
/// matching C# `CheckZeroReferred() > Limits.MaxStackSize` — unlike the old
/// check, which only counted top-level eval-stack entries.
pub(crate) fn count_references(
    stack: &[StackValue],
    locals: &[StackValue],
    args: &[StackValue],
    static_fields: &[StackValue],
) -> usize {
    // Root references: AddStackReference is called once per stack entry and slot.
    let mut total = stack.len() + locals.len() + args.len() + static_fields.len();
    let mut visited: alloc::collections::BTreeSet<u64> = alloc::collections::BTreeSet::new();
    for roots in [stack, locals, args, static_fields] {
        for value in roots {
            total += count_child_edges(value, &mut visited);
        }
    }
    total
}

fn count_child_edges(value: &StackValue, visited: &mut alloc::collections::BTreeSet<u64>) -> usize {
    match value {
        StackValue::Array(id, items) | StackValue::Struct(id, items) => {
            if !visited.insert(*id) {
                return 0;
            }
            // C# Array/Struct SubItemsCount == items.len(): one AddReference per child.
            let mut edges = items.len();
            for child in items {
                edges += count_child_edges(child, visited);
            }
            edges
        }
        StackValue::Map(id, entries) => {
            if !visited.insert(*id) {
                return 0;
            }
            // C# Map.SubItemsCount == Count * 2 (each key and each value is a reference).
            let mut edges = entries.len() * 2;
            for (key, val) in entries {
                edges += count_child_edges(key, visited);
                edges += count_child_edges(val, visited);
            }
            edges
        }
        // Buffer is reference-tracked but has no StackItem children.
        _ => 0,
    }
}

#[cfg(test)]
mod reference_count_tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn nested_array_counts_each_child_edge() {
        // One array of 5 ints on the stack: 1 stack ref + 5 child edges = 6.
        let arr = StackValue::Array(1, vec![StackValue::Integer(0); 5]);
        assert_eq!(count_references(&[arr], &[], &[], &[]), 6);
    }

    #[test]
    fn map_counts_keys_and_values() {
        // One map of 2 entries: 1 stack ref + 2*2 (key+value) = 5.
        let map = StackValue::Map(
            1,
            vec![
                (StackValue::Integer(1), StackValue::Integer(10)),
                (StackValue::Integer(2), StackValue::Integer(20)),
            ],
        );
        assert_eq!(count_references(&[map], &[], &[], &[]), 5);
    }

    #[test]
    fn slots_are_counted_as_references() {
        let stack = vec![StackValue::Integer(0)];
        let locals = vec![StackValue::Integer(1), StackValue::Integer(2)];
        let statics = vec![StackValue::Integer(3)];
        // 1 stack + 2 locals + 1 static = 4.
        assert_eq!(count_references(&stack, &locals, &[], &statics), 4);
    }

    #[test]
    fn shared_compound_children_counted_once() {
        // inner (id 9) is a child of both A (id 1) and B (id 2), and both A and B
        // are on the stack. C# counts: 2 stack (A,B) + 1 (A->inner) + 1 (B->inner)
        // + inner's children (2) = 6. The inner's children must NOT be double-counted.
        let inner = || StackValue::Array(9, vec![StackValue::Integer(0); 2]);
        let a = StackValue::Array(1, vec![inner()]);
        let b = StackValue::Array(2, vec![inner()]);
        assert_eq!(count_references(&[a, b], &[], &[], &[]), 6);
    }

    #[test]
    fn deeply_nested_array_exceeds_limit() {
        // A single root array whose total child edges exceed MaxStackSize must be
        // counted as over-limit even though the eval stack holds just one item.
        let items: vec::Vec<StackValue> = (0..3000).map(StackValue::Integer).collect();
        let arr = StackValue::Array(1, items);
        assert!(count_references(&[arr], &[], &[], &[]) > 2048);
    }
}

#[inline]
fn clone_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    out.extend_from_slice(bytes);
    out
}

#[inline]
pub(crate) fn compound_id(value: &StackValue) -> Option<u64> {
    match value {
        StackValue::Array(id, _)
        | StackValue::Struct(id, _)
        | StackValue::Map(id, _)
        | StackValue::Buffer(id, _) => Some(*id),
        _ => None,
    }
}

pub(crate) fn find_affected_indices(target_id: u64, stack: &[StackValue]) -> Vec<usize> {
    let mut indices = Vec::with_capacity(stack.len().min(8));
    for (idx, value) in stack.iter().enumerate() {
        if contains_compound_id(value, target_id) {
            indices.push(idx);
        }
    }
    indices
}

fn contains_compound_id(value: &StackValue, target_id: u64) -> bool {
    if compound_id(value) == Some(target_id) {
        return true;
    }
    match value {
        StackValue::Array(_, items) | StackValue::Struct(_, items) => items
            .iter()
            .any(|item| contains_compound_id(item, target_id)),
        StackValue::Map(_, items) => items
            .iter()
            .any(|(k, v)| contains_compound_id(k, target_id) || contains_compound_id(v, target_id)),
        _ => false,
    }
}

pub(crate) fn propagate_update(
    updated: &StackValue,
    stack: &mut [StackValue],
    locals: &mut [StackValue],
    args: &mut [StackValue],
    static_fields: &mut [StackValue],
    affected_stack_indices: Option<&[usize]>,
) {
    match affected_stack_indices {
        Some(indices) if !indices.is_empty() => {
            for &idx in indices {
                if idx < stack.len() {
                    replace_alias(&mut stack[idx], updated);
                }
            }
        }
        Some(_) => {
            // Empty indices - skip stack iteration
        }
        None => {
            for value in stack {
                replace_alias(value, updated);
            }
        }
    }
    for value in locals {
        replace_alias(value, updated);
    }
    for value in args {
        replace_alias(value, updated);
    }
    for value in static_fields {
        replace_alias(value, updated);
    }
}

pub(crate) fn propagate_aliases_from_sources(targets: &mut [StackValue], sources: &[StackValue]) {
    for source in sources {
        propagate_alias_from_source(targets, source);
    }
}

fn propagate_alias_from_source(targets: &mut [StackValue], source: &StackValue) {
    if compound_id(source).is_some() {
        for target in targets.iter_mut() {
            replace_alias(target, source);
        }
    }

    match source {
        StackValue::Array(_, items) | StackValue::Struct(_, items) => {
            for item in items {
                propagate_alias_from_source(targets, item);
            }
        }
        StackValue::Map(_, items) => {
            for (key, value) in items {
                propagate_alias_from_source(targets, key);
                propagate_alias_from_source(targets, value);
            }
        }
        StackValue::Buffer(_, _)
        | StackValue::Integer(_)
        | StackValue::BigInteger(_)
        | StackValue::ByteString(_)
        | StackValue::Boolean(_)
        | StackValue::Pointer(_)
        | StackValue::Interop(_)
        | StackValue::Iterator(_)
        | StackValue::Null => {}
    }
}

fn replace_alias(target: &mut StackValue, updated: &StackValue) {
    let target_id = compound_id(target);
    if target_id.is_some() && target_id == compound_id(updated) {
        *target = updated.clone();
        return;
    }

    match target {
        StackValue::Array(_, items) | StackValue::Struct(_, items) => {
            for item in items {
                replace_alias(item, updated);
            }
        }
        StackValue::Map(_, items) => {
            for (key, value) in items {
                replace_alias(key, updated);
                replace_alias(value, updated);
            }
        }
        StackValue::Buffer(_, _)
        | StackValue::Integer(_)
        | StackValue::BigInteger(_)
        | StackValue::ByteString(_)
        | StackValue::Boolean(_)
        | StackValue::Pointer(_)
        | StackValue::Interop(_)
        | StackValue::Iterator(_)
        | StackValue::Null => {}
    }
}
