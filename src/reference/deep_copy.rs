use super::{SharedHeap, SharedValue, Value};
use crate::{StackItemType, StackValue};
use alloc::{collections::BTreeMap, string::String, vec::Vec};

impl SharedValue {
    /// Whether this collection rejects mutation, as in Neo's immutable notification snapshots.
    pub fn is_read_only(&self) -> bool {
        match &self.0 {
            Value::Object(root) => {
                root.arena.borrow().slots[root.id]
                    .as_ref()
                    .expect("live root keeps node alive")
                    .read_only
            }
            _ => false,
        }
    }

    fn make_read_only(&self) {
        if let Value::Object(root) = &self.0 {
            root.arena.borrow_mut().slots[root.id]
                .as_mut()
                .expect("live root keeps node alive")
                .read_only = true;
        }
    }
}

impl SharedHeap {
    /// Neo StackItem.DeepCopy: preserve graph aliases/cycles, copy all compounds, and optionally
    /// freeze collections and turn Buffers into ByteStrings. Existing readonly state is retained.
    pub fn deep_copy(
        &self,
        value: &SharedValue,
        as_immutable: bool,
    ) -> Result<SharedValue, String> {
        self.validate_value(value)?;
        let mut copies = BTreeMap::new();
        let mut pending = Vec::new();
        let root = self.copy_shell(value, as_immutable, &mut copies, &mut pending)?;
        while let Some((source, target)) = pending.pop() {
            if source.kind() == StackItemType::Map {
                for (key, value) in source.map_items()? {
                    let copied =
                        self.copy_shell(&value, as_immutable, &mut copies, &mut pending)?;
                    target.set_item_raw(&key, &copied)?;
                }
            } else {
                for value in source.items()? {
                    let copied =
                        self.copy_shell(&value, as_immutable, &mut copies, &mut pending)?;
                    target.append_raw(&copied)?;
                }
            }
            if as_immutable || source.is_read_only() {
                target.make_read_only();
            }
        }
        Ok(root)
    }

    fn copy_shell(
        &self,
        value: &SharedValue,
        as_immutable: bool,
        copies: &mut BTreeMap<usize, SharedValue>,
        pending: &mut Vec<(SharedValue, SharedValue)>,
    ) -> Result<SharedValue, String> {
        let Some(id) = value.object_id() else {
            return Ok(value.clone());
        };
        if let Some(copy) = copies.get(&id) {
            return Ok(copy.clone());
        }
        let copy = match value.kind() {
            StackItemType::Array => self.array(Vec::new())?,
            StackItemType::Struct => self.structure(Vec::new())?,
            StackItemType::Map => self.map(Vec::new())?,
            StackItemType::Buffer if as_immutable => {
                SharedValue::primitive(StackValue::ByteString(value.buffer_bytes()?))?
            }
            StackItemType::Buffer => self.buffer(value.buffer_bytes()?)?,
            _ => unreachable!("arena object type"),
        };
        copies.insert(id, copy.clone());
        if value.kind() != StackItemType::Buffer {
            pending.push((value.clone(), copy.clone()));
        }
        Ok(copy)
    }
}
