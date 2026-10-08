use super::{SharedScriptIdentity, SharedValue, Value};
use crate::{StackItemType, StackValue};
use alloc::{
    rc::{Rc, Weak},
    string::{String, ToString},
    vec::Vec,
};
use core::cell::RefCell;

/// The object arena shared by nested interpreters and their native/ordinary callbacks.
#[derive(Clone, Default)]
pub struct SharedHeap(pub(super) Rc<RefCell<Arena>>);

#[derive(Default)]
pub(super) struct Arena {
    pub slots: Vec<Option<Slot>>,
    free: Vec<usize>,
    pub subitems: usize,
    frame_roots: usize,
}
pub(super) struct Slot {
    pub node: Node,
    pub read_only: bool,
    root: Weak<Root>,
}
pub(super) struct Root {
    pub id: usize,
    pub kind: StackItemType,
    pub arena: Rc<RefCell<Arena>>,
}

#[derive(Clone)]
pub(super) enum Edge {
    Primitive(StackValue),
    Pointer(SharedScriptIdentity, usize),
    Object(usize),
}

pub(super) enum Node {
    Array(Vec<Edge>),
    Struct(Vec<Edge>),
    Map(Vec<(Edge, Edge)>),
    Buffer(Vec<u8>),
}

impl Node {
    pub fn kind(&self) -> StackItemType {
        match self {
            Self::Array(_) => StackItemType::Array,
            Self::Struct(_) => StackItemType::Struct,
            Self::Map(_) => StackItemType::Map,
            Self::Buffer(_) => StackItemType::Buffer,
        }
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Array(v) | Self::Struct(v) => v.len(),
            Self::Map(v) => v.len(),
            Self::Buffer(v) => v.len(),
        }
    }
    pub fn subitems(&self) -> usize {
        match self {
            Self::Array(v) | Self::Struct(v) => v.len(),
            Self::Map(v) => v.len() * 2,
            Self::Buffer(_) => 0,
        }
    }
    fn children(&self) -> Vec<&Edge> {
        match self {
            Self::Array(v) | Self::Struct(v) => v.iter().collect(),
            Self::Map(v) => v.iter().flat_map(|(k, v)| [k, v]).collect(),
            Self::Buffer(_) => Vec::new(),
        }
    }
}

impl SharedHeap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate_value(&self, value: &SharedValue) -> Result<(), String> {
        self.edge(value).map(|_| ())
    }

    pub(super) fn edge(&self, value: &SharedValue) -> Result<Edge, String> {
        match &value.0 {
            Value::Primitive(value) => Ok(Edge::Primitive(value.clone())),
            Value::Pointer(script, position) => Ok(Edge::Pointer(script.clone(), *position)),
            Value::Object(root) if Rc::ptr_eq(&self.0, &root.arena) => Ok(Edge::Object(root.id)),
            Value::Object(_) => Err("compound values belong to different engine heaps".to_string()),
        }
    }

    pub(super) fn resolve(&self, edge: Edge) -> Result<SharedValue, String> {
        match edge {
            Edge::Primitive(value) => SharedValue::primitive(value),
            Edge::Pointer(script, position) => Ok(SharedValue::pointer(script, position)),
            Edge::Object(id) => {
                let mut arena = self.0.borrow_mut();
                let slot = arena
                    .slots
                    .get_mut(id)
                    .and_then(Option::as_mut)
                    .ok_or_else(|| "dangling object reference".to_string())?;
                let root = if let Some(root) = slot.root.upgrade() {
                    root
                } else {
                    let root = Rc::new(Root {
                        id,
                        kind: slot.node.kind(),
                        arena: self.0.clone(),
                    });
                    slot.root = Rc::downgrade(&root);
                    root
                };
                Ok(SharedValue(Value::Object(root)))
            }
        }
    }

    fn allocate(&self, node: Node) -> Result<SharedValue, String> {
        let mut arena = self.0.borrow_mut();
        let id = arena.free.pop().unwrap_or(arena.slots.len());
        let root = Rc::new(Root {
            id,
            kind: node.kind(),
            arena: self.0.clone(),
        });
        arena.subitems += node.subitems();
        let slot = Some(Slot {
            node,
            read_only: false,
            root: Rc::downgrade(&root),
        });
        if id == arena.slots.len() {
            arena.slots.push(slot);
        } else {
            arena.slots[id] = slot;
        }
        Ok(SharedValue(Value::Object(root)))
    }

    pub fn array(&self, items: Vec<SharedValue>) -> Result<SharedValue, String> {
        self.allocate(Node::Array(
            items
                .iter()
                .map(|v| self.edge(v))
                .collect::<Result<_, _>>()?,
        ))
    }
    pub fn structure(&self, items: Vec<SharedValue>) -> Result<SharedValue, String> {
        self.allocate(Node::Struct(
            items
                .iter()
                .map(|v| self.edge(v))
                .collect::<Result<_, _>>()?,
        ))
    }
    pub fn map(&self, items: Vec<(SharedValue, SharedValue)>) -> Result<SharedValue, String> {
        let result = self.allocate(Node::Map(Vec::new()))?;
        for (key, value) in items {
            result.set_item_raw(&key, &value)?;
        }
        Ok(result)
    }
    pub fn buffer(&self, bytes: Vec<u8>) -> Result<SharedValue, String> {
        self.allocate(Node::Buffer(bytes))
    }

    /// New values crossing the old value ABI are imported explicitly, never during a reference call.
    pub fn import_abi(&self, value: StackValue) -> Result<SharedValue, String> {
        enum Work {
            Visit(StackValue),
            Finish(StackItemType, usize),
        }
        let mut work = alloc::vec![Work::Visit(value)];
        let mut values = Vec::new();
        let mut remaining = 2048usize;
        while let Some(step) = work.pop() {
            match step {
                Work::Visit(value) => {
                    remaining = remaining
                        .checked_sub(1)
                        .ok_or_else(|| "shared value import exceeds item limit".to_string())?;
                    let (kind, children) = match value {
                        StackValue::Array(v) => (StackItemType::Array, v),
                        StackValue::Struct(v) => (StackItemType::Struct, v),
                        StackValue::Map(v) => (
                            StackItemType::Map,
                            v.into_iter().flat_map(|(k, v)| [k, v]).collect(),
                        ),
                        StackValue::Buffer(bytes) => {
                            values.push(self.buffer(bytes)?);
                            continue;
                        }
                        primitive => {
                            values.push(SharedValue::primitive(primitive)?);
                            continue;
                        }
                    };
                    if children.len() > remaining {
                        return Err("shared value import exceeds item limit".to_string());
                    }
                    work.push(Work::Finish(kind, children.len()));
                    work.extend(children.into_iter().rev().map(Work::Visit));
                }
                Work::Finish(kind, count) => {
                    let children = values.split_off(values.len() - count);
                    let value = match kind {
                        StackItemType::Array => self.array(children)?,
                        StackItemType::Struct => self.structure(children)?,
                        StackItemType::Map => {
                            let mut iter = children.into_iter();
                            let mut pairs = Vec::new();
                            while let Some(k) = iter.next() {
                                pairs.push((k, iter.next().expect("map pair")));
                            }
                            self.map(pairs)?
                        }
                        _ => unreachable!("compound import type"),
                    };
                    values.push(value);
                }
            }
        }
        values
            .pop()
            .ok_or_else(|| "empty shared import".to_string())
    }

    /// Collect unreachable cycles. This is an explicit pressure/phase-boundary operation, not a
    /// per-instruction graph copy. External handles are roots; internal edges hold no Rc roots.
    pub fn collect(&self) -> usize {
        let mut arena = self.0.borrow_mut();
        let mut reachable = alloc::vec![false; arena.slots.len()];
        let mut pending = arena
            .slots
            .iter()
            .enumerate()
            .filter_map(|(id, slot)| {
                slot.as_ref()
                    .filter(|s| s.root.strong_count() != 0)
                    .map(|_| id)
            })
            .collect::<Vec<_>>();
        while let Some(id) = pending.pop() {
            if reachable[id] {
                continue;
            }
            reachable[id] = true;
            if let Some(slot) = &arena.slots[id] {
                for edge in slot.node.children() {
                    if let Edge::Object(child) = edge {
                        pending.push(*child);
                    }
                }
            }
        }
        let mut removed = 0;
        for (id, live) in reachable.into_iter().enumerate() {
            if !live {
                if let Some(slot) = arena.slots[id].take() {
                    arena.subitems -= slot.node.subitems();
                    arena.free.push(id);
                    removed += 1;
                }
            }
        }
        removed
    }

    pub fn object_count(&self) -> usize {
        let arena = self.0.borrow();
        arena.slots.len() - arena.free.len()
    }

    /// Native ReferenceCounter counts Array/Struct elements and both Map keys and values.
    pub fn subitem_count(&self) -> usize {
        self.0.borrow().subitems
    }
    pub fn reference_count(&self) -> usize {
        let arena = self.0.borrow();
        arena.subitems + arena.frame_roots
    }
    pub fn register_frame(&self) -> SharedFrameRoots {
        SharedFrameRoots {
            heap: self.clone(),
            count: 0,
        }
    }
}

/// A scoped count of real VM evaluation/slot references for one nested interpreter.
/// Host temporary handles only retain GC roots and are deliberately excluded from this count.
pub struct SharedFrameRoots {
    heap: SharedHeap,
    count: usize,
}
impl SharedFrameRoots {
    pub fn set(&mut self, count: usize) {
        let mut arena = self.heap.0.borrow_mut();
        arena.frame_roots = arena.frame_roots - self.count + count;
        self.count = count;
    }
}
impl Drop for SharedFrameRoots {
    fn drop(&mut self) {
        self.heap.0.borrow_mut().frame_roots -= self.count;
    }
}
