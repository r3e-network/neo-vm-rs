use super::SharedValue;
use crate::{StackItemType, StackValue};
use alloc::{
    collections::BTreeSet,
    string::{String, ToString},
    vec::Vec,
};

enum Work {
    Visit(SharedValue),
    Finish(StackItemType, usize, usize),
}

impl SharedValue {
    /// Explicit acyclic value snapshot. Shared aliases are copied by value; active cycles fail.
    /// Execution/call boundaries never use this projection.
    pub fn snapshot(&self, max_items: usize) -> Result<StackValue, String> {
        let mut work = alloc::vec![Work::Visit(self.clone())];
        let mut active = BTreeSet::new();
        let mut output = Vec::new();
        let mut remaining = max_items;
        while let Some(step) = work.pop() {
            match step {
                Work::Visit(value) => {
                    remaining = remaining
                        .checked_sub(1)
                        .ok_or_else(|| "shared snapshot item limit".to_string())?;
                    if let Some((_, position)) = value.pointer_info() {
                        output.push(StackValue::Pointer(position as i64));
                        continue;
                    }
                    if let Some(primitive) = value.primitive_ref() {
                        output.push(primitive.clone());
                        continue;
                    }
                    let id = value.object_id().expect("object");
                    if !active.insert(id) {
                        return Err("cyclic value cannot be serialized".to_string());
                    }
                    let kind = value.kind();
                    if kind == StackItemType::Buffer {
                        output.push(StackValue::Buffer(value.buffer_bytes()?));
                        active.remove(&id);
                        continue;
                    }
                    let values = if kind == StackItemType::Map {
                        value
                            .map_items()?
                            .into_iter()
                            .flat_map(|(k, v)| [k, v])
                            .collect::<Vec<_>>()
                    } else {
                        value.items()?
                    };
                    if values.len() > remaining {
                        return Err("shared snapshot item limit".to_string());
                    }
                    work.push(Work::Finish(kind, values.len(), id));
                    work.extend(values.into_iter().rev().map(Work::Visit));
                }
                Work::Finish(kind, count, id) => {
                    let values = output.split_off(output.len() - count);
                    output.push(match kind {
                        StackItemType::Array => StackValue::Array(values),
                        StackItemType::Struct => StackValue::Struct(values),
                        StackItemType::Map => {
                            let mut iter = values.into_iter();
                            let mut pairs = Vec::new();
                            while let Some(k) = iter.next() {
                                pairs.push((k, iter.next().expect("map pair")))
                            }
                            StackValue::Map(pairs)
                        }
                        _ => unreachable!("compound type"),
                    });
                    active.remove(&id);
                }
            }
        }
        output
            .pop()
            .ok_or_else(|| "empty shared snapshot".to_string())
    }
}
