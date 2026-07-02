//! C# `Neo.SmartContract.IInteroperable` equivalent.
//!
//! Types that can be converted to and from [`StackValue`] for storage
//! round-tripping through the VM stack. This is the canonical Rust
//! representation of the C# `IInteroperable` interface.

use alloc::boxed::Box;
use alloc::string::String;

use crate::StackValue;

/// Trait for types that can be serialized to/from NeoVM stack values.
///
/// This mirrors the C# `IInteroperable` interface exactly:
/// - `from_stack_value` ↔ `FromStackItem(StackItem)`
/// - `to_stack_value` ↔ `ToStackItem(IReferenceCounter?)`
/// - `clone_box` ↔ `Clone()`
/// - `from_replica` ↔ `FromReplica(IInteroperable)`
// `from_*` here take `&mut self` deliberately: they POPULATE self (deserialize
// into an existing instance), mirroring C# `IInteroperable.FromStackItem` /
// `FromReplica`, so clippy's "from_* takes no self" convention does not apply.
#[allow(clippy::wrong_self_convention)]
pub trait Interoperable: core::fmt::Debug + Send + Sync {
    /// Populate this instance from a [`StackValue`].
    ///
    /// Corresponds to C# `IInteroperable.FromStackItem`.
    fn from_stack_value(&mut self, value: StackValue) -> Result<(), InteroperableError>;

    /// Convert this instance to a [`StackValue`].
    ///
    /// Corresponds to C# `IInteroperable.ToStackItem`.
    fn to_stack_value(&self) -> Result<StackValue, InteroperableError>;

    /// Create a boxed clone of this interoperable instance.
    ///
    /// Corresponds to C# `IInteroperable.Clone`.
    fn clone_box(&self) -> Box<dyn Interoperable>;

    /// Populate this instance by cloning from another interoperable value.
    ///
    /// Default implementation round-trips through `StackValue`.
    /// Corresponds to C# `IInteroperable.FromReplica`.
    fn from_replica(&mut self, replica: &dyn Interoperable) -> Result<(), InteroperableError> {
        self.from_stack_value(replica.to_stack_value()?)
    }
}

/// Error type for [`Interoperable`] conversions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InteroperableError {
    /// The stack value type is not compatible with the target type.
    InvalidType(String),
    /// The stack value data is malformed or out of range.
    InvalidData(String),
    /// The conversion is not supported (e.g., write-only types like Signer).
    NotSupported(String),
}

impl core::fmt::Display for InteroperableError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidType(msg) => write!(f, "Invalid type: {msg}"),
            Self::InvalidData(msg) => write!(f, "Invalid data: {msg}"),
            Self::NotSupported(msg) => write!(f, "Not supported: {msg}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InteroperableError {}

/// Implement [`Interoperable`] for a type that already has
/// `to_stack_value(&self) -> StackValue` and
/// `from_stack_value(StackValue) -> Result<Self, E>`.
///
/// The error type `E` must implement `Display` so it can be converted
/// to [`InteroperableError::InvalidData`].
#[macro_export]
macro_rules! impl_interoperable {
    ($ty:ty) => {
        impl $crate::Interoperable for $ty {
            fn from_stack_value(
                &mut self,
                value: $crate::StackValue,
            ) -> ::core::result::Result<(), $crate::InteroperableError> {
                *self = <Self as ::core::convert::TryFrom<$crate::StackValue>>::try_from(value)
                    .map_err(|e| {
                        $crate::InteroperableError::InvalidData(
                            ::alloc::string::ToString::to_string(&e),
                        )
                    })?;
                Ok(())
            }

            fn to_stack_value(
                &self,
            ) -> ::core::result::Result<$crate::StackValue, $crate::InteroperableError> {
                Ok(<Self as ::core::convert::Into<$crate::StackValue>>::into(
                    self.clone(),
                ))
            }

            fn clone_box(&self) -> ::alloc::boxed::Box<dyn $crate::Interoperable> {
                ::alloc::boxed::Box::new(self.clone())
            }
        }
    };
}
