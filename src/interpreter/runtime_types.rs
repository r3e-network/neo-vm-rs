use crate::StackValue;
use alloc::vec::Vec;

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

pub(crate) fn count_references(
    stack: &[StackValue],
    locals: &[StackValue],
    args: &[StackValue],
    static_fields: &[StackValue],
) -> usize {
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
            let mut edges = entries.len() * 2;
            for (key, val) in entries {
                edges += count_child_edges(key, visited);
                edges += count_child_edges(val, visited);
            }
            edges
        }
        _ => 0,
    }
}

#[cfg(test)]
mod reference_count_tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn nested_array_counts_each_child_edge() {
        let arr = StackValue::Array(1, vec![StackValue::Integer(0); 5]);
        assert_eq!(count_references(&[arr], &[], &[], &[]), 6);
    }

    #[test]
    fn map_counts_keys_and_values() {
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
        assert_eq!(count_references(&stack, &locals, &[], &statics), 4);
    }

    #[test]
    fn shared_compound_children_counted_once() {
        let inner = || StackValue::Array(9, vec![StackValue::Integer(0); 2]);
        let a = StackValue::Array(1, vec![inner()]);
        let b = StackValue::Array(2, vec![inner()]);
        assert_eq!(count_references(&[a, b], &[], &[], &[]), 6);
    }

    #[test]
    fn deeply_nested_array_exceeds_limit() {
        let items: vec::Vec<StackValue> = (0..3000).map(StackValue::Integer).collect();
        let arr = StackValue::Array(1, items);
        assert!(count_references(&[arr], &[], &[], &[]) > 2048);
    }
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
        Some(_) => {}
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
