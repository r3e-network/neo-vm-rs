use alloc::{format, string::String};

use crate::StackValue;

#[derive(Debug, Clone)]
pub(super) enum PendingException {
    Message(String),
    ThrownValue(StackValue),
}

impl PendingException {
    pub(super) fn message(message: String) -> Self {
        Self::Message(message)
    }

    pub(super) fn thrown_value(value: StackValue) -> Self {
        Self::ThrownValue(value)
    }

    pub(super) fn into_catch_item(self) -> StackValue {
        match self {
            Self::Message(message) => StackValue::ByteString(message.into_bytes()),
            Self::ThrownValue(value) => value,
        }
    }

    pub(super) fn fault_message(&self) -> String {
        match self {
            Self::Message(message) => message.clone(),
            Self::ThrownValue(value) => format!("exception: {:?}", value),
        }
    }
}
