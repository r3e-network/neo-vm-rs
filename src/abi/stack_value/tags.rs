//! Tag constants for the canonical NeoVM stack value types.

/// Compact integer tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_INTEGER: u8 = 0;
/// Compact boolean tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_BOOLEAN: u8 = 1;
/// Compact byte string tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_BYTESTRING: u8 = 2;
/// Compact big integer tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_BIG_INTEGER: u8 = 3;
/// Compact array tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_ARRAY: u8 = 4;
/// Compact struct tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_STRUCT: u8 = 5;
/// Compact map tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_MAP: u8 = 6;
/// Compact null tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_NULL: u8 = 7;
/// Compact interop handle tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_INTEROP: u8 = 8;
/// Compact iterator handle tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_ITERATOR: u8 = 9;
/// Compact buffer tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_BUFFER: u8 = 10;
/// Compact pointer tag used by generated RISC-V runtime helpers.
pub const COMPACT_TAG_POINTER: u8 = 11;

/// Shared binary codec tag for integer stack values.
pub const STACK_VALUE_CODEC_TAG_INTEGER: u8 = 0x01;
/// Shared binary codec tag for big integer stack values.
pub const STACK_VALUE_CODEC_TAG_BIG_INTEGER: u8 = 0x02;
/// Shared binary codec tag for byte string stack values.
pub const STACK_VALUE_CODEC_TAG_BYTESTRING: u8 = 0x03;
/// Shared binary codec tag for boolean stack values.
pub const STACK_VALUE_CODEC_TAG_BOOLEAN: u8 = 0x04;
/// Shared binary codec tag for array stack values.
pub const STACK_VALUE_CODEC_TAG_ARRAY: u8 = 0x05;
/// Shared binary codec tag for struct stack values.
pub const STACK_VALUE_CODEC_TAG_STRUCT: u8 = 0x06;
/// Shared binary codec tag for map stack values.
pub const STACK_VALUE_CODEC_TAG_MAP: u8 = 0x07;
/// Shared binary codec tag for interop stack values.
pub const STACK_VALUE_CODEC_TAG_INTEROP: u8 = 0x08;
/// Shared binary codec tag for iterator stack values.
pub const STACK_VALUE_CODEC_TAG_ITERATOR: u8 = 0x09;
/// Shared binary codec tag for null stack values.
pub const STACK_VALUE_CODEC_TAG_NULL: u8 = 0x0A;
/// Shared binary codec tag for pointer stack values.
pub const STACK_VALUE_CODEC_TAG_POINTER: u8 = 0x0B;
/// Shared binary codec tag for buffer stack values.
pub const STACK_VALUE_CODEC_TAG_BUFFER: u8 = 0x0C;

/// NeoVM `StackItemType.Any`.
pub const NEOVM_STACK_ITEM_TYPE_ANY: u8 = 0x00;
/// NeoVM `StackItemType.Pointer`.
pub const NEOVM_STACK_ITEM_TYPE_POINTER: u8 = 0x10;
/// NeoVM `StackItemType.Boolean`.
pub const NEOVM_STACK_ITEM_TYPE_BOOLEAN: u8 = 0x20;
/// NeoVM `StackItemType.Integer`.
pub const NEOVM_STACK_ITEM_TYPE_INTEGER: u8 = 0x21;
/// NeoVM `StackItemType.ByteString`.
pub const NEOVM_STACK_ITEM_TYPE_BYTESTRING: u8 = 0x28;
/// NeoVM `StackItemType.Buffer`.
pub const NEOVM_STACK_ITEM_TYPE_BUFFER: u8 = 0x30;
/// NeoVM `StackItemType.Array`.
pub const NEOVM_STACK_ITEM_TYPE_ARRAY: u8 = 0x40;
/// NeoVM `StackItemType.Struct`.
pub const NEOVM_STACK_ITEM_TYPE_STRUCT: u8 = 0x41;
/// NeoVM `StackItemType.Map`.
pub const NEOVM_STACK_ITEM_TYPE_MAP: u8 = 0x48;
/// NeoVM `StackItemType.InteropInterface`.
pub const NEOVM_STACK_ITEM_TYPE_INTEROP_INTERFACE: u8 = 0x60;
