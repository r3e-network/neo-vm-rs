//! Shared execution limits used by NeoVM consumers.

/// Maximum NeoVM script size accepted by local execution and proof input code.
pub const MAX_SCRIPT_SIZE: usize = 1024 * 1024;

/// Default maximum evaluation stack depth.
pub const DEFAULT_MAX_STACK_DEPTH: usize = 2048;

/// Default maximum invocation depth.
pub const DEFAULT_MAX_INVOCATION_DEPTH: usize = 1024;

/// Maximum size for buffers and compound values used by bounded execution.
pub const MAX_ITEM_SIZE: usize = 1024 * 1024;
