//! Opt-in reference-preserving values for one NeoVM engine family.
//!
//! Compound nodes live in one flat arena. Internal edges contain object identifiers, not Rc
//! pointers, so cycles are representable without Rc cycles. Only explicit snapshots, collection
//! cloning, comparison, and garbage collection traverse a graph; stack/call copies retain handles.

mod heap;
mod operations;
mod snapshot;

pub use crate::interpreter::SharedVmException;
pub use heap::{SharedFrameRoots, SharedHeap};

use crate::{StackItemType, StackValue};
use alloc::{
    rc::Rc,
    string::{String, ToString},
    vec::Vec,
};
use core::fmt;
use heap::Root;

/// Identity of one loaded Script object; internal CALL/initializer frames reuse it.
#[derive(Clone, Debug)]
pub struct SharedScriptIdentity(Rc<()>);
impl SharedScriptIdentity {
    pub(crate) fn new() -> Self {
        Self(Rc::new(()))
    }
    pub fn same_script(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

/// A primitive, scoped script pointer, or handle into a shared engine-family object arena.
#[derive(Clone)]
pub struct SharedValue(Value);

#[derive(Clone)]
enum Value {
    Primitive(StackValue),
    Pointer(SharedScriptIdentity, usize),
    Object(Rc<Root>),
}

impl fmt::Debug for SharedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Value::Primitive(value) => value.fmt(f),
            Value::Pointer(_, position) => write!(f, "Pointer@{position}"),
            Value::Object(root) => write!(f, "{:?}#{}", root.kind, root.id),
        }
    }
}

impl PartialEq for SharedValue {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Value::Primitive(left), Value::Primitive(right)) => left == right,
            (Value::Pointer(ls, lp), Value::Pointer(rs, rp)) => lp == rp && ls.same_script(rs),
            (Value::Object(_), Value::Object(_)) => self.same_reference(other),
            _ => false,
        }
    }
}

impl SharedValue {
    /// Import a primitive without assigning an object identity.
    pub fn primitive(value: StackValue) -> Result<Self, String> {
        if matches!(value, StackValue::Pointer(_)) {
            return Err("a pointer requires its loaded Script identity".to_string());
        }
        if matches!(
            value,
            StackValue::Array(_)
                | StackValue::Struct(_)
                | StackValue::Map(_)
                | StackValue::Buffer(_)
        ) {
            return Err("compound values require a shared heap".to_string());
        }
        Ok(Self(Value::Primitive(value)))
    }

    pub fn pointer(script: SharedScriptIdentity, position: usize) -> Self {
        Self(Value::Pointer(script, position))
    }
    pub fn pointer_info(&self) -> Option<(SharedScriptIdentity, usize)> {
        match &self.0 {
            Value::Pointer(script, position) => Some((script.clone(), *position)),
            _ => None,
        }
    }

    pub fn primitive_ref(&self) -> Option<&StackValue> {
        match &self.0 {
            Value::Primitive(value) => Some(value),
            _ => None,
        }
    }

    pub fn kind(&self) -> StackItemType {
        match &self.0 {
            Value::Object(root) => root.kind,
            Value::Pointer(_, _) => StackItemType::Pointer,
            Value::Primitive(value) => match value {
                StackValue::Integer(_) | StackValue::BigInteger(_) => StackItemType::Integer,
                StackValue::ByteString(_) => StackItemType::ByteString,
                StackValue::Boolean(_) => StackItemType::Boolean,
                StackValue::Pointer(_) => StackItemType::Pointer,
                StackValue::Interop(_) | StackValue::Iterator(_) => StackItemType::InteropInterface,
                StackValue::Null => StackItemType::Any,
                _ => unreachable!("primitive constructor validates the type"),
            },
        }
    }

    pub fn object_id(&self) -> Option<usize> {
        match &self.0 {
            Value::Object(root) => Some(root.id),
            _ => None,
        }
    }

    pub fn same_reference(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Value::Object(left), Value::Object(right)) => {
                left.id == right.id && Rc::ptr_eq(&left.arena, &right.arena)
            }
            _ => false,
        }
    }

    pub fn heap(&self) -> Option<SharedHeap> {
        match &self.0 {
            Value::Object(root) => Some(SharedHeap(root.arena.clone())),
            _ => None,
        }
    }

    fn with_node<T>(
        &self,
        action: impl FnOnce(&heap::Node) -> Result<T, String>,
    ) -> Result<T, String> {
        let Value::Object(root) = &self.0 else {
            return Err("expected compound value".to_string());
        };
        let arena = root.arena.borrow();
        action(
            &arena.slots[root.id]
                .as_ref()
                .expect("live root keeps node alive")
                .node,
        )
    }

    fn with_node_mut<T>(
        &self,
        action: impl FnOnce(&mut heap::Node) -> Result<T, String>,
    ) -> Result<T, String> {
        let Value::Object(root) = &self.0 else {
            return Err("expected compound value".to_string());
        };
        let mut arena = root.arena.borrow_mut();
        let node = &mut arena.slots[root.id]
            .as_mut()
            .expect("live root keeps node alive")
            .node;
        let before = node.subitems();
        let result = action(node);
        let after = node.subitems();
        arena.subitems = arena.subitems - before + after;
        result
    }

    /// Buffer/Array/Struct/Map length; primitive lengths follow their native byte span.
    pub fn len(&self) -> Result<usize, String> {
        if let Some(value) = self.primitive_ref() {
            return crate::stack_value_span_bytes(value)
                .map(|v| v.len())
                .ok_or_else(|| "value has no size".to_string());
        }
        self.with_node(|node| Ok(node.len()))
    }

    pub fn is_empty(&self) -> Result<bool, String> {
        self.len().map(|len| len == 0)
    }

    pub fn items(&self) -> Result<Vec<SharedValue>, String> {
        let edges = self.with_node(|node| match node {
            heap::Node::Array(items) | heap::Node::Struct(items) => Ok(items.clone()),
            _ => Err("expected Array or Struct".to_string()),
        })?;
        let heap = self.heap().expect("object checked");
        edges.into_iter().map(|edge| heap.resolve(edge)).collect()
    }

    pub fn map_items(&self) -> Result<Vec<(SharedValue, SharedValue)>, String> {
        let pairs = self.with_node(|node| match node {
            heap::Node::Map(items) => Ok(items.clone()),
            _ => Err("expected Map".to_string()),
        })?;
        let heap = self.heap().expect("object checked");
        pairs
            .into_iter()
            .map(|(key, value)| Ok((heap.resolve(key)?, heap.resolve(value)?)))
            .collect()
    }

    pub fn buffer_bytes(&self) -> Result<Vec<u8>, String> {
        self.with_node(|node| match node {
            heap::Node::Buffer(bytes) => Ok(bytes.clone()),
            _ => Err("expected Buffer".to_string()),
        })
    }

    /// A type/span-only projection for non-collection opcode helpers. It never walks children.
    pub(crate) fn type_view(&self) -> StackValue {
        match &self.0 {
            Value::Primitive(value) => value.clone(),
            Value::Pointer(_, position) => StackValue::Pointer(*position as i64),
            Value::Object(root) => match root.kind {
                StackItemType::Array => StackValue::Array(Vec::new()),
                StackItemType::Struct => StackValue::Struct(Vec::new()),
                StackItemType::Map => StackValue::Map(Vec::new()),
                StackItemType::Buffer => {
                    StackValue::Buffer(self.buffer_bytes().expect("Buffer handle"))
                }
                _ => unreachable!("arena has only compound types"),
            },
        }
    }
}
