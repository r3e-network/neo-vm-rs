use alloc::{format, string::String};

// Shared exception-carrier vocabulary for BOTH engines. Its full surface is used
// across the crate, but each single-engine build (interpreter- or runtime-only)
// exercises only a subset — the `#[allow(dead_code)]`s below mark the members that
// are engine-specific rather than genuinely dead. The default (both engines) uses
// every member, so a real dead member would still surface on the canonical build.
#[derive(Debug, Clone)]
pub(crate) enum PendingException<V> {
    // Constructed only by the `interpreter` (host-error strings); the `runtime`
    // carries `ThrownValue` exclusively but still matches this in its renderers.
    #[allow(dead_code)]
    Message(String),
    ThrownValue(V),
}

pub(crate) trait PendingExceptionValue {
    fn from_exception_message(message: String) -> Self;
}

impl PendingExceptionValue for crate::StackValue {
    fn from_exception_message(message: String) -> Self {
        Self::ByteString(message.into_bytes())
    }
}

impl<V> PendingException<V> {
    // interpreter-only constructor
    #[allow(dead_code)]
    pub(crate) fn message(message: String) -> Self {
        Self::Message(message)
    }

    pub(crate) fn thrown_value(value: V) -> Self {
        Self::ThrownValue(value)
    }

    pub(crate) fn into_catch_item(self) -> V
    where
        V: PendingExceptionValue,
    {
        match self {
            Self::Message(message) => V::from_exception_message(message),
            Self::ThrownValue(value) => value,
        }
    }
}

impl<V: core::fmt::Debug> PendingException<V> {
    // runtime-only fault renderer (borrows)
    #[allow(dead_code)]
    pub(crate) fn fault_message(&self) -> String {
        match self {
            Self::Message(message) => message.clone(),
            Self::ThrownValue(value) => format!("exception: {:?}", value),
        }
    }

    // interpreter-only fault renderer (consumes)
    #[allow(dead_code)]
    pub(crate) fn into_fault_message(self) -> String {
        match self {
            Self::Message(message) => message,
            Self::ThrownValue(value) => format!("THROW: {:?}", value),
        }
    }
}
