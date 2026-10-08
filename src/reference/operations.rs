use super::{
    SharedValue,
    heap::{Edge, Node},
};
use crate::{StackItemType, StackValue};
use alloc::{
    collections::VecDeque,
    string::{String, ToString},
    vec::Vec,
};
use num_bigint::BigInt;
use num_traits::ToPrimitive;

fn primitive_integer(value: &SharedValue) -> Result<i32, String> {
    let integer = match value.primitive_ref() {
        Some(StackValue::Integer(value)) => BigInt::from(*value),
        Some(StackValue::Boolean(value)) => BigInt::from(u8::from(*value)),
        Some(StackValue::BigInteger(bytes) | StackValue::ByteString(bytes))
            if bytes.len() <= 32 =>
        {
            BigInt::from_signed_bytes_le(bytes)
        }
        _ => return Err("collection index/value must be a primitive integer".to_string()),
    };
    integer
        .to_i32()
        .ok_or_else(|| "collection integer exceeds Int32 range".to_string())
}

fn map_key(value: &SharedValue) -> Result<&StackValue, String> {
    match value.primitive_ref() {
        Some(value @ (StackValue::Integer(_) | StackValue::Boolean(_))) => Ok(value),
        Some(value @ StackValue::BigInteger(bytes)) if bytes.len() <= 32 => Ok(value),
        Some(value @ StackValue::ByteString(bytes)) if bytes.len() <= 64 => Ok(value),
        _ => Err("map key must be a primitive of at most 64 bytes".to_string()),
    }
}

fn integer(value: &StackValue) -> Option<BigInt> {
    match value {
        StackValue::Integer(n) => Some(BigInt::from(*n)),
        StackValue::BigInteger(bytes) => Some(BigInt::from_signed_bytes_le(bytes)),
        _ => None,
    }
}

fn primitive_equal(left: &StackValue, right: &StackValue) -> bool {
    if let (Some(left), Some(right)) = (integer(left), integer(right)) {
        return left == right;
    }
    left == right
}

fn key_equal(left: &Edge, right: &StackValue) -> bool {
    matches!(left, Edge::Primitive(value) if primitive_equal(value,right))
}

impl SharedValue {
    /// Read one edge without traversing the referenced object. None denotes a missing key/index.
    pub fn item(&self, key: &SharedValue) -> Result<Option<Self>, String> {
        let edge = self.with_node(|node| match node {
            Node::Array(items) | Node::Struct(items) => {
                let index = primitive_integer(key)?;
                Ok(usize::try_from(index)
                    .ok()
                    .and_then(|i| items.get(i).cloned()))
            }
            Node::Map(items) => {
                let key = map_key(key)?;
                Ok(items
                    .iter()
                    .find(|(k, _)| key_equal(k, key))
                    .map(|(_, v)| v.clone()))
            }
            Node::Buffer(bytes) => {
                let index = primitive_integer(key)?;
                Ok(usize::try_from(index)
                    .ok()
                    .and_then(|i| bytes.get(i))
                    .map(|n| Edge::Primitive(StackValue::Integer(i64::from(*n)))))
            }
        })?;
        edge.map(|edge| self.heap().expect("object checked").resolve(edge))
            .transpose()
    }

    /// Assign an edge directly. Opcode adapters implement the explicit Struct clone rule first.
    pub fn set_item_raw(&self, key: &SharedValue, value: &Self) -> Result<bool, String> {
        let heap = self
            .heap()
            .ok_or_else(|| "expected mutable collection".to_string())?;
        let edge = heap.edge(value)?;
        self.with_node_mut(|node| match node {
            Node::Array(items) | Node::Struct(items) => {
                let index = primitive_integer(key)?;
                let Some(item) = usize::try_from(index).ok().and_then(|i| items.get_mut(i)) else {
                    return Ok(false);
                };
                *item = edge;
                Ok(true)
            }
            Node::Map(items) => {
                let key = map_key(key)?;
                if let Some((_, old)) = items.iter_mut().find(|(k, _)| key_equal(k, key)) {
                    *old = edge;
                } else {
                    items.push((Edge::Primitive(key.clone()), edge));
                }
                Ok(true)
            }
            Node::Buffer(bytes) => {
                let index = primitive_integer(key)?;
                let Some(byte) = usize::try_from(index).ok().and_then(|i| bytes.get_mut(i)) else {
                    return Ok(false);
                };
                let n = primitive_integer(value)?;
                if !(-128..=255).contains(&n) {
                    return Err("SETITEM byte exceeds range".to_string());
                }
                *byte = n as u8;
                Ok(true)
            }
        })
    }

    pub fn append_raw(&self, value: &Self) -> Result<(), String> {
        let edge = self
            .heap()
            .ok_or_else(|| "expected array".to_string())?
            .edge(value)?;
        self.with_node_mut(|node| match node {
            Node::Array(items) | Node::Struct(items) => {
                items.push(edge);
                Ok(())
            }
            _ => Err("APPEND expects Array or Struct".to_string()),
        })
    }

    pub fn remove_item(&self, key: &SharedValue) -> Result<bool, String> {
        self.with_node_mut(|node| match node {
            Node::Array(items) | Node::Struct(items) => {
                let index = primitive_integer(key)?;
                let Some(index) = usize::try_from(index).ok().filter(|i| *i < items.len()) else {
                    return Ok(false);
                };
                items.remove(index);
                Ok(true)
            }
            Node::Map(items) => {
                let key = map_key(key)?;
                if let Some(index) = items.iter().position(|(k, _)| key_equal(k, key)) {
                    items.remove(index);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            _ => Err("REMOVE expects Array, Struct, or Map".to_string()),
        })
    }

    pub fn clear(&self) -> Result<(), String> {
        self.with_node_mut(|node| match node {
            Node::Array(v) | Node::Struct(v) => {
                v.clear();
                Ok(())
            }
            Node::Map(v) => {
                v.clear();
                Ok(())
            }
            _ => Err("CLEARITEMS expects compound type".to_string()),
        })
    }

    pub fn reverse(&self) -> Result<(), String> {
        self.with_node_mut(|node| match node {
            Node::Array(v) | Node::Struct(v) => {
                v.reverse();
                Ok(())
            }
            Node::Buffer(v) => {
                v.reverse();
                Ok(())
            }
            _ => Err("REVERSEITEMS expects Array, Struct, or Buffer".to_string()),
        })
    }

    pub fn pop(&self) -> Result<Option<Self>, String> {
        let edge = self.with_node_mut(|node| match node {
            Node::Array(v) | Node::Struct(v) => Ok(v.pop()),
            _ => Err("POPITEM expects Array or Struct".to_string()),
        })?;
        edge.map(|edge| self.heap().expect("object checked").resolve(edge))
            .transpose()
    }

    pub fn write_buffer(&self, offset: usize, bytes: &[u8]) -> Result<(), String> {
        self.with_node_mut(|node| match node {
            Node::Buffer(buffer) => {
                let end = offset
                    .checked_add(bytes.len())
                    .ok_or_else(|| "buffer range overflow".to_string())?;
                let target = buffer
                    .get_mut(offset..end)
                    .ok_or_else(|| "buffer range out of bounds".to_string())?;
                target.copy_from_slice(bytes);
                Ok(())
            }
            _ => Err("expected Buffer".to_string()),
        })
    }

    /// Core Struct.Clone: only Struct children are copied; all other object edges remain shared.
    /// There is deliberately no cycle memoization: a Struct cycle exceeds the clone-item limit.
    pub fn clone_struct(&self, max_stack_size: usize) -> Result<Self, String> {
        if self.kind() != StackItemType::Struct {
            return Ok(self.clone());
        }
        let mut remaining = max_stack_size
            .checked_sub(1)
            .ok_or_else(|| "struct clone limit".to_string())?;
        let heap = self.heap().expect("Struct handle");
        let result = heap.structure(Vec::new())?;
        let mut queue = VecDeque::from([(self.clone(), result.clone())]);
        while let Some((source, target)) = queue.pop_front() {
            for item in source.items()? {
                remaining = remaining
                    .checked_sub(1)
                    .ok_or_else(|| "Beyond struct subitem clone limits!".to_string())?;
                if item.kind() == StackItemType::Struct {
                    let copied = heap.structure(Vec::new())?;
                    target.append_raw(&copied)?;
                    queue.push_back((item, copied));
                } else {
                    target.append_raw(&item)?;
                }
            }
        }
        Ok(result)
    }

    /// Native equality: reference equality for mutable non-Struct objects; bounded value equality
    /// for Struct, including reference equality of any Array/Map/Buffer fields it contains.
    pub fn vm_equal(
        &self,
        other: &Self,
        max_items: usize,
        max_comparable_size: usize,
    ) -> Result<bool, String> {
        let mut pending = alloc::vec![(self.clone(), other.clone())];
        let mut items = max_items;
        let mut comparable = max_comparable_size;
        while let Some((left, right)) = pending.pop() {
            items = items.checked_sub(1).ok_or_else(|| {
                "Too many struct items to compare in struct comparison.".to_string()
            })?;
            if let Some(StackValue::ByteString(bytes)) = left.primitive_ref() {
                if comparable == 0 || bytes.len() > comparable {
                    return Err("byte string comparison limit".to_string());
                }
                let Some(StackValue::ByteString(other)) = right.primitive_ref() else {
                    return Ok(false);
                };
                if other.len() > comparable {
                    return Err("byte string comparison limit".to_string());
                }
                comparable -= bytes.len().max(other.len()).max(1);
                if bytes != other {
                    return Ok(false);
                }
                continue;
            }
            comparable = comparable
                .checked_sub(1)
                .ok_or_else(|| "struct comparison limit".to_string())?;
            if left.kind() == StackItemType::Struct {
                if left.same_reference(&right) {
                    continue;
                }
                if right.kind() != StackItemType::Struct {
                    return Ok(false);
                }
                let a = left.items()?;
                let b = right.items()?;
                if a.len() != b.len() {
                    return Ok(false);
                }
                pending.extend(a.into_iter().zip(b));
            } else if let (Some(a), Some(b)) = (left.primitive_ref(), right.primitive_ref()) {
                if !primitive_equal(a, b) {
                    return Ok(false);
                }
            } else if left.kind() == StackItemType::Pointer
                && right.kind() == StackItemType::Pointer
            {
                if left != right {
                    return Ok(false);
                }
            } else if !left.same_reference(&right) {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
